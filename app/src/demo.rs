//! Demo mode: curated historical severe-weather volumes loaded in place of
//! live data, so rendering can be evaluated against fixed reference scenes.
//! See docs/superpowers/specs/2026-07-23-demo-mode-design.md.

use nexrad_data::aws::archive::Identifier;
use std::path::PathBuf;

/// A curated historical event. `volume_name` is the exact archive object name
/// (verified to exist in the `unidata-nexrad-level2` bucket); the S3 key is
/// derived from it by `nexrad_data::aws::archive::download_file`.
#[derive(Debug)]
pub struct DemoEvent {
    pub key: &'static str,
    pub label: &'static str,
    pub site: &'static str,
    pub volume_name: &'static str,
}

/// The curated demo scenes.
pub const EVENTS: &[DemoEvent] = &[
    DemoEvent {
        key: "moore2013",
        label: "Moore, OK EF5 tornado — 2013-05-20 20:16 UTC",
        site: "KTLX",
        volume_name: "KTLX20130520_201643_V06.gz",
    },
    DemoEvent {
        key: "harvey2017",
        label: "Hurricane Harvey landfall — 2017-08-26 03:04 UTC",
        site: "KCRP",
        volume_name: "KCRP20170826_030439_V06",
    },
];

/// A parsed `--demo` request: either a curated event or a local volume file.
#[derive(Debug)]
pub enum DemoRequest {
    Event(&'static DemoEvent),
    File(PathBuf),
}

impl DemoRequest {
    /// Human-readable label for the status line.
    pub fn label(&self) -> String {
        match self {
            DemoRequest::Event(ev) => ev.label.to_string(),
            DemoRequest::File(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
        }
    }

    /// The archive volume file name (basename for local files).
    pub fn volume_name(&self) -> String {
        match self {
            DemoRequest::Event(ev) => ev.volume_name.to_string(),
            DemoRequest::File(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
        }
    }

    /// The radar site the scene belongs to, derived from the canonical
    /// volume file name for local files (e.g. `KTLX20130520_...` -> `KTLX`).
    pub fn site_id(&self) -> Result<String, String> {
        match self {
            DemoRequest::Event(ev) => Ok(ev.site.to_string()),
            DemoRequest::File(_) => {
                let name = self.volume_name();
                let identifier = Identifier::new(name.clone());
                // Validate that the filename is in canonical NEXRAD format
                // by checking if we can parse the date_time
                identifier.date_time().ok_or_else(|| {
                    format!(
                        "cannot derive radar site from file name {name:?} — expected \
                         a canonical volume name like KTLX20130520_201643_V06"
                    )
                })?;
                // If we got here, the filename is valid; extract the site
                identifier.site().map(str::to_string).ok_or(format!(
                    "cannot derive radar site from file name {name:?} — expected \
                         a canonical volume name like KTLX20130520_201643_V06"
                ))
            }
        }
    }
}

/// Scan CLI args for `--demo <event|path>`. Returns `Ok(None)` when the flag
/// is absent; `Err` carries a user-facing message (unknown key errors list the
/// available events).
pub fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Option<DemoRequest>, String> {
    while let Some(arg) = args.next() {
        if arg != "--demo" {
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("--demo requires a value\n{}", available_events()))?;
        if let Some(ev) = EVENTS.iter().find(|ev| ev.key == value) {
            return Ok(Some(DemoRequest::Event(ev)));
        }
        let path = PathBuf::from(&value);
        if path.is_file() {
            return Ok(Some(DemoRequest::File(path)));
        }
        return Err(format!(
            "unknown demo event or file {value:?}\n{}",
            available_events()
        ));
    }
    Ok(None)
}

fn available_events() -> String {
    let mut s = String::from("available demo events:\n");
    for ev in EVENTS {
        s.push_str(&format!("  {:<12} {}\n", ev.key, ev.label));
    }
    s.push_str("  <path>       any local Level II archive volume file");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_finds_known_event() {
        let req = parse_args(["--demo".to_string(), "moore2013".to_string()].into_iter())
            .unwrap()
            .unwrap();
        match req {
            DemoRequest::Event(ev) => {
                assert_eq!(ev.site, "KTLX");
                assert_eq!(ev.volume_name, "KTLX20130520_201643_V06.gz");
            }
            other => panic!("expected Event, got {other:?}"),
        }
    }

    #[test]
    fn parse_args_without_flag_is_none() {
        assert!(parse_args(std::iter::empty()).unwrap().is_none());
    }

    #[test]
    fn parse_args_unknown_key_lists_events() {
        let err = parse_args(["--demo".to_string(), "nope".to_string()].into_iter()).unwrap_err();
        assert!(
            err.contains("moore2013"),
            "error should list events, got: {err}"
        );
        assert!(err.contains("harvey2017"));
    }

    #[test]
    fn parse_args_missing_value_errors() {
        assert!(parse_args(["--demo".to_string()].into_iter()).is_err());
    }

    #[test]
    fn parse_args_existing_path_becomes_file_request() {
        let path = std::env::temp_dir().join("KTLX20130520_201643_V06.gz");
        std::fs::write(&path, b"stub").unwrap();
        let req = parse_args(["--demo".to_string(), path.display().to_string()].into_iter())
            .unwrap()
            .unwrap();
        match &req {
            DemoRequest::File(p) => assert_eq!(p, &path),
            other => panic!("expected File, got {other:?}"),
        }
        // Site is derived from the canonical volume filename.
        assert_eq!(req.site_id().unwrap(), "KTLX");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn site_id_for_events_and_bad_filenames() {
        let ev = DemoRequest::Event(&EVENTS[0]);
        assert_eq!(ev.site_id().unwrap(), "KTLX");
        let bad = DemoRequest::File(std::path::PathBuf::from("/tmp/not-a-volume.bin"));
        assert!(bad.site_id().is_err());
    }
}
