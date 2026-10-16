//! Per-storm a-deck download scheduler for the spaghetti-models overlay.
//!
//! Ply `net` ignores `get` for an id that still has an entry and keeps a
//! finished result (including an `Err`) for 60 frames; it also has no request
//! timeout (`docs/reviews/windows-stability-review.md` C2/C3). Each storm
//! uses one stable id, and the scheduler calls `cancel()` after every result
//! and on an app-side deadline, which frees the id for the retry. Failures
//! back off exponentially, and each storm's own failure streak independently
//! decides whether its own failures ask for a toast. Each storm has its own
//! slot, so a hung download never blocks another storm or any other NHC
//! data.
//!
//! The 60s per-attempt deadline is measured against a monotonic clock
//! (`std::time::Instant`), not the wall-clock `now: f64` passed into
//! `tick`. `now` is still used for backoff/`next_due` scheduling, where a
//! wrong value is self-correcting and low-stakes; but a wall clock that can
//! jump (sleep/resume, NTP) must never be trusted to measure "how long has
//! this request been in flight" (review S6). `tick` also polls the
//! transport before checking that deadline, so a response that finished
//! this frame is always processed, never thrown away because the deadline
//! also elapsed this frame.
//!
//! `NetPort` is the transport boundary: the planned app-owned HTTP module
//! can replace `PlyNet` without touching the parser, renderer, or scheduler.

use crate::adeck::adeck_url;
use std::collections::BTreeMap;
use std::time::Instant;

/// Refresh period while the overlay is on. A-decks fill in over hours; the
/// research asks for no faster than every 10-15 min.
pub const REFRESH_SECS: f64 = 900.0;
/// App-side deadline per attempt, measured against a monotonic clock (see
/// module docs). Ply net sets no HTTP timeout (review C3); a 350 KB file
/// normally lands in a few seconds.
pub const ATTEMPT_TIMEOUT_SECS: f64 = 60.0;
/// First retry delay; doubles per consecutive failure.
pub const BACKOFF_BASE_SECS: f64 = 30.0;
/// Cap for exponential backoff after a real (non-404) transport or parse
/// failure. Lower than `REFRESH_SECS`: a storm that keeps failing shouldn't
/// wait as long between checks as a healthy one's normal refresh (L4).
pub const BACKOFF_CAP_SECS: f64 = 300.0;
/// Flat retry delay for a storm with no a-deck published yet (HTTP 404).
/// Deliberately not exponential: a brand-new invest isn't going to get an
/// a-deck sooner by backing off harder, and the normal cap would still make
/// it wait longer than necessary between checks (L4).
pub const NOT_FOUND_RETRY_SECS: f64 = 600.0;

/// Delay before the next attempt after `failures` consecutive non-404
/// failures. 404s use `NOT_FOUND_RETRY_SECS` instead; see `record_failure`.
pub fn backoff_secs(failures: u32) -> f64 {
    if failures == 0 {
        return 0.0;
    }
    let exp = (failures - 1).min(5);
    (BACKOFF_BASE_SECS * f64::from(1u32 << exp)).min(BACKOFF_CAP_SECS)
}

/// Stable per-storm request id. Reusable because every result is cancelled.
pub fn adeck_net_id(storm_id: &str) -> String {
    format!("nhc-adeck-{storm_id}")
}

/// Result of polling one request id.
pub enum NetPoll {
    Pending,
    Done(Result<Vec<u8>, String>),
    /// No such id: Ply evicted it or it was never fired.
    Missing,
}

/// Transport boundary for a-deck downloads. `PlyNet` implements it today; the
/// planned app-owned HTTP module replaces it later. Contract: after `cancel`,
/// the id is free and a new `get` with it starts a fresh request.
pub trait NetPort {
    fn get(&mut self, id: &str, url: &str);
    fn poll(&mut self, id: &str) -> NetPoll;
    fn cancel(&mut self, id: &str);
}

/// `NetPort` over Ply 1.1 `net`. The only Ply-specific code in the feature.
pub struct PlyNet;

impl NetPort for PlyNet {
    fn get(&mut self, id: &str, url: &str) {
        use ply_engine::prelude::net;
        // The scheduler only fires from Idle, so any entry still under this id
        // is stale; free it, or Ply would silently ignore this `get`.
        if let Some(req) = net::request(id) {
            req.cancel();
        }
        net::get(id, url, |c| c.header("User-Agent", "rustywx/0.3"));
    }

    fn poll(&mut self, id: &str) -> NetPoll {
        use ply_engine::prelude::net;
        match net::request(id) {
            None => NetPoll::Missing,
            Some(req) => match req.response() {
                None => NetPoll::Pending,
                Some(Ok(resp)) => NetPoll::Done(Ok(resp.bytes().to_vec())),
                Some(Err(e)) => NetPoll::Done(Err(e)),
            },
        }
    }

    /// Removes Ply's entry and frees the id. A hung ureq thread keeps running
    /// until the socket dies, but its result goes to a dropped channel.
    fn cancel(&mut self, id: &str) {
        use ply_engine::prelude::net;
        if let Some(req) = net::request(id) {
            req.cancel();
        }
    }
}

/// Monotonic time source behind the attempt deadline (review S6). The wall
/// clock `now: f64` passed into `tick` can jump (sleep/resume, an NTP or
/// manual clock step), so it can't safely measure elapsed time for a single
/// in-flight request. `SystemClock` is the real source; tests substitute a
/// fake to control elapsed time deterministically, without a real sleep.
trait Clock {
    fn now(&self) -> Instant;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// What the NHC panel shows for a storm with no (or refreshing) guidance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotStatus {
    Idle,
    Downloading,
    Parsing,
    Retrying {
        in_secs: u32,
    },
    /// No a-deck has ever been published for this storm (404). Distinct
    /// from `Retrying` because it isn't a failure in the usual sense and
    /// uses its own flat retry cadence (L4).
    WaitingForGuidance {
        in_secs: u32,
    },
}

pub fn status_message(status: &SlotStatus) -> String {
    match status {
        SlotStatus::Idle => "No model guidance yet".to_string(),
        SlotStatus::Downloading => "Downloading model guidance\u{2026}".to_string(),
        SlotStatus::Parsing => "Processing model guidance\u{2026}".to_string(),
        SlotStatus::Retrying { in_secs } => {
            format!("Model guidance unavailable, retrying in {in_secs} s")
        }
        SlotStatus::WaitingForGuidance { in_secs } => {
            format!("Waiting for model guidance, checking again in {in_secs} s")
        }
    }
}

#[derive(Default)]
pub struct TickReport {
    /// Caller should `show_toast(.., ErrorKind::Network)`.
    pub show_toast: bool,
}

#[derive(Debug, PartialEq)]
pub struct ParseOutcome {
    pub storm_id: String,
    pub show_toast: bool,
}

enum Phase {
    Idle,
    InFlight { started_at: Instant },
    Downloaded(Vec<u8>),
    Parsing { generation: u64 },
}

struct Slot {
    url: String,
    net_id: String,
    phase: Phase,
    next_due: f64,
    failures: u32,
    /// Per-storm toast streak (review S7): only the first failure of THIS
    /// storm's own streak toasts. A different storm's success or failure
    /// never touches this flag.
    toast_shown_in_streak: bool,
    /// True once this storm's a-deck has come back 404 (not published yet).
    /// Drives the flat `NOT_FOUND_RETRY_SECS` cadence and the
    /// `WaitingForGuidance` status instead of the normal backoff (L4).
    not_found: bool,
}

/// Count a failure for one slot: back off (or use the flat 404 cadence), go
/// idle, and say whether to toast. `quiet` failures (404, or a parse
/// failure) never toast and never arm the streak, so they can't suppress a
/// later genuine failure's toast. `not_found` additionally switches this
/// slot to the flat retry cadence and the `WaitingForGuidance` status.
fn record_failure(slot: &mut Slot, now: f64, quiet: bool, not_found: bool) -> bool {
    slot.failures = slot.failures.saturating_add(1);
    slot.not_found = not_found;
    slot.next_due = now
        + if not_found {
            NOT_FOUND_RETRY_SECS
        } else {
            backoff_secs(slot.failures)
        };
    slot.phase = Phase::Idle;
    if quiet || slot.toast_shown_in_streak {
        return false;
    }
    slot.toast_shown_in_streak = true;
    true
}

pub struct AdeckFetcher {
    slots: BTreeMap<String, Slot>,
    clock: Box<dyn Clock>,
}

impl Default for AdeckFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl AdeckFetcher {
    pub fn new() -> Self {
        Self {
            slots: BTreeMap::new(),
            clock: Box::new(SystemClock),
        }
    }

    /// Test-only seam for the monotonic clock (review S6): lets tests
    /// control elapsed time for the attempt deadline deterministically,
    /// without a real sleep.
    #[cfg(test)]
    fn with_clock(clock: impl Clock + 'static) -> Self {
        Self {
            slots: BTreeMap::new(),
            clock: Box::new(clock),
        }
    }

    /// Per-frame driver. `active` are `CurrentStorms.json` ids; unsupported
    /// ids are ignored. Never blocks: it only fires, polls, and cancels.
    pub fn tick(&mut self, now: f64, active: &[String], net: &mut impl NetPort) -> TickReport {
        let mut report = TickReport::default();
        let wanted: Vec<(String, String)> = active
            .iter()
            .filter_map(|id| {
                let id = id.trim().to_ascii_lowercase();
                adeck_url(&id).map(|url| (id, url))
            })
            .collect();

        // Storms that left CurrentStorms.json: cancel and forget.
        let gone: Vec<String> = self
            .slots
            .keys()
            .filter(|k| !wanted.iter().any(|(id, _)| id == *k))
            .cloned()
            .collect();
        for id in gone {
            if let Some(slot) = self.slots.remove(&id)
                && matches!(slot.phase, Phase::InFlight { .. })
            {
                net.cancel(&slot.net_id);
            }
        }
        for (id, url) in wanted {
            let net_id = adeck_net_id(&id);
            self.slots.entry(id).or_insert(Slot {
                url,
                net_id,
                phase: Phase::Idle,
                next_due: now,
                failures: 0,
                toast_shown_in_streak: false,
                not_found: false,
            });
        }

        for (storm_id, slot) in self.slots.iter_mut() {
            match slot.phase {
                Phase::Idle if now >= slot.next_due => {
                    net.get(&slot.net_id, &slot.url);
                    slot.phase = Phase::InFlight {
                        started_at: self.clock.now(),
                    };
                }
                Phase::InFlight { started_at } => {
                    // Poll the transport FIRST (review S6): a response that
                    // finished this frame must always be processed, never
                    // cancelled and discarded because the deadline also
                    // elapsed this frame. Every exit path cancels, so the id
                    // is free for the retry and a cached Err is never
                    // re-read (review C2).
                    let outcome = match net.poll(&slot.net_id) {
                        NetPoll::Pending => {
                            let elapsed = self.clock.now().saturating_duration_since(started_at);
                            if elapsed.as_secs_f64() >= ATTEMPT_TIMEOUT_SECS {
                                net.cancel(&slot.net_id);
                                Some(Err(format!("timed out after {ATTEMPT_TIMEOUT_SECS} s")))
                            } else {
                                None
                            }
                        }
                        NetPoll::Done(result) => {
                            net.cancel(&slot.net_id);
                            Some(result)
                        }
                        NetPoll::Missing => Some(Err("request handle evicted".to_owned())),
                    };
                    match outcome {
                        None => {}
                        Some(Ok(bytes)) => slot.phase = Phase::Downloaded(bytes),
                        Some(Err(e)) => {
                            eprintln!("Warning: a-deck download failed storm={storm_id}: {e}");
                            let not_found = e.contains("404");
                            if record_failure(slot, now, not_found, not_found) {
                                report.show_toast = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        report
    }

    /// Hand the next downloaded body to the parse worker, tagging it with
    /// `generation`. Call only when the parse queue has capacity.
    pub fn take_parse_job(&mut self, generation: u64) -> Option<(String, Vec<u8>)> {
        let (storm_id, slot) = self
            .slots
            .iter_mut()
            .find(|(_, s)| matches!(s.phase, Phase::Downloaded(_)))?;
        match std::mem::replace(&mut slot.phase, Phase::Parsing { generation }) {
            Phase::Downloaded(bytes) => Some((storm_id.clone(), bytes)),
            other => {
                slot.phase = other;
                None
            }
        }
    }

    pub fn owns_generation(&self, generation: u64) -> bool {
        self.slots
            .values()
            .any(|s| matches!(s.phase, Phase::Parsing { generation: g } if g == generation))
    }

    /// True while any slot is in the shared parse queue (`Phase::Parsing`).
    /// The submit loop in `main.rs` uses this to cap a-decks to at most one
    /// in-flight parse job, so they can't fill all of `QUEUE_CAPACITY` and
    /// starve the NHC metadata, alerts, and borders jobs that share it.
    pub fn has_parsing(&self) -> bool {
        self.slots
            .values()
            .any(|s| matches!(s.phase, Phase::Parsing { .. }))
    }

    /// Record a parse result. `None` when the generation isn't ours (storm
    /// removed or overlay disabled meanwhile): the caller drops the result.
    pub fn parse_finished(&mut self, generation: u64, ok: bool, now: f64) -> Option<ParseOutcome> {
        let (storm_id, slot) = self.slots.iter_mut().find(
            |(_, s)| matches!(s.phase, Phase::Parsing { generation: g } if g == generation),
        )?;
        let show_toast = if ok {
            slot.failures = 0;
            slot.not_found = false;
            slot.next_due = now + REFRESH_SECS;
            slot.phase = Phase::Idle;
            slot.toast_shown_in_streak = false;
            false
        } else {
            // A parse failure is a bad gzip/inflate, or a valid-but-empty
            // a-deck (a brand-new invest with no parseable rows yet) — never
            // a transport problem, so it never toasts as a "Network error"
            // (L4). It also isn't a 404, so it stays on the normal backoff
            // schedule rather than the flat not-found cadence.
            record_failure(slot, now, true, false)
        };
        Some(ParseOutcome {
            storm_id: storm_id.clone(),
            show_toast,
        })
    }

    /// Overlay switched off: cancel downloads and forget all slots, so
    /// switching it back on fetches immediately.
    pub fn disable(&mut self, net: &mut impl NetPort) {
        for slot in self.slots.values() {
            if matches!(slot.phase, Phase::InFlight { .. }) {
                net.cancel(&slot.net_id);
            }
        }
        self.slots.clear();
    }

    pub fn is_active(&self) -> bool {
        !self.slots.is_empty()
    }

    pub fn status(&self, storm_id: &str, now: f64) -> SlotStatus {
        let Some(slot) = self.slots.get(&storm_id.to_ascii_lowercase()) else {
            return SlotStatus::Idle;
        };
        match slot.phase {
            Phase::InFlight { .. } => SlotStatus::Downloading,
            Phase::Downloaded(_) | Phase::Parsing { .. } => SlotStatus::Parsing,
            Phase::Idle if slot.not_found => SlotStatus::WaitingForGuidance {
                in_secs: (slot.next_due - now).max(0.0).ceil() as u32,
            },
            Phase::Idle if slot.failures > 0 => SlotStatus::Retrying {
                in_secs: (slot.next_due - now).max(0.0).ceil() as u32,
            },
            Phase::Idle => SlotStatus::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;
    use std::time::Duration;

    /// Scripted stand-in for Ply net with its real id semantics.
    #[derive(Default)]
    struct FakeNet {
        /// Ids with an entry (pending or finished, not yet cancelled).
        live: HashSet<String>,
        fired: Vec<(String, String)>,
        /// `get` calls Ply would have silently ignored. Must stay empty.
        ignored: Vec<String>,
        responses: HashMap<String, NetPoll>,
        cancelled: Vec<String>,
    }

    impl FakeNet {
        fn evict(&mut self, id: &str) {
            self.live.remove(id);
        }
        fn respond(&mut self, id: &str, poll: NetPoll) {
            self.responses.insert(id.to_owned(), poll);
        }
    }

    impl NetPort for FakeNet {
        fn get(&mut self, id: &str, url: &str) {
            if !self.live.insert(id.to_owned()) {
                self.ignored.push(id.to_owned());
                return;
            }
            self.fired.push((id.to_owned(), url.to_owned()));
        }
        fn poll(&mut self, id: &str) -> NetPoll {
            if !self.live.contains(id) {
                return NetPoll::Missing;
            }
            self.responses.remove(id).unwrap_or(NetPoll::Pending)
        }
        fn cancel(&mut self, id: &str) {
            self.live.remove(id);
            self.cancelled.push(id.to_owned());
        }
    }

    /// Fake monotonic clock (review S6): shares one `Instant` cell with the
    /// test via `Rc`, so the test can advance it directly to make elapsed
    /// time deterministic, instead of sleeping for real.
    #[derive(Clone)]
    struct FakeClock(Rc<Cell<Instant>>);

    impl FakeClock {
        fn new() -> Self {
            Self(Rc::new(Cell::new(Instant::now())))
        }
        fn advance(&self, d: Duration) {
            self.0.set(self.0.get() + d);
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            self.0.get()
        }
    }

    const AL: &str = "nhc-adeck-al092026";
    const EP: &str = "nhc-adeck-ep182026";

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn fires_once_per_active_storm_with_stable_ids() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        f.tick(0.0, &ids(&["al092026", "EP182026"]), &mut net);
        assert_eq!(
            net.fired,
            [
                (
                    AL.to_owned(),
                    "https://ftp.nhc.noaa.gov/atcf/aid_public/aal092026.dat.gz".to_owned()
                ),
                (
                    EP.to_owned(),
                    "https://ftp.nhc.noaa.gov/atcf/aid_public/aep182026.dat.gz".to_owned()
                ),
            ]
        );
        f.tick(1.0, &ids(&["al092026", "ep182026"]), &mut net);
        assert_eq!(net.fired.len(), 2, "no re-fire while in flight");
        assert!(net.ignored.is_empty());
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Downloading);
    }

    #[test]
    fn ignores_unsupported_storm_ids() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        f.tick(0.0, &ids(&["wp012026", "al852026", ""]), &mut net);
        assert!(net.fired.is_empty());
        assert!(!f.is_active());
    }

    #[test]
    fn success_cancels_handle_then_refreshes_same_id_later() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Ok(vec![1, 2, 3])));
        assert!(!f.tick(1.0, &active, &mut net).show_toast);
        assert_eq!(net.cancelled, [AL], "finished entry is released");
        assert_eq!(
            f.take_parse_job(7),
            Some(("al092026".to_owned(), vec![1, 2, 3]))
        );
        assert_eq!(f.take_parse_job(8), None);
        assert!(f.owns_generation(7));
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Parsing);
        let outcome = f.parse_finished(7, true, 2.0).unwrap();
        assert_eq!(outcome.storm_id, "al092026");
        assert!(!outcome.show_toast);
        assert!(!f.owns_generation(7));
        f.tick(901.0, &active, &mut net);
        assert_eq!(net.fired.len(), 1, "not due before REFRESH_SECS");
        f.tick(902.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        assert_eq!(net.fired[1].0, AL, "same stable id, re-fired after cancel");
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn has_parsing_true_only_while_a_slot_is_in_the_parsing_phase() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        assert!(!f.has_parsing(), "idle/in-flight slot is not parsing");
        net.respond(AL, NetPoll::Done(Ok(vec![1, 2, 3])));
        f.tick(1.0, &active, &mut net);
        assert!(!f.has_parsing(), "downloaded-but-not-taken is not parsing");
        assert_eq!(
            f.take_parse_job(7),
            Some(("al092026".to_owned(), vec![1, 2, 3]))
        );
        assert!(f.has_parsing(), "take_parse_job moved the slot to Parsing");
        f.parse_finished(7, true, 2.0);
        assert!(!f.has_parsing(), "parse_finished cleared the Parsing slot");
    }

    #[test]
    fn network_error_cancels_then_retries_with_backoff() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 503".into())));
        assert!(
            f.tick(1.0, &active, &mut net).show_toast,
            "first failure toasts"
        );
        assert_eq!(net.cancelled, [AL], "failed entry is released, not re-read");
        assert_eq!(
            f.status("al092026", 1.0),
            SlotStatus::Retrying { in_secs: 30 }
        );
        f.tick(30.0, &active, &mut net);
        assert_eq!(net.fired.len(), 1);
        f.tick(31.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2, "retry fires after backoff");
        assert!(
            net.ignored.is_empty(),
            "retry was not swallowed by a stale entry"
        );
        net.respond(AL, NetPoll::Done(Err("http status: 503".into())));
        assert!(
            !f.tick(32.0, &active, &mut net).show_toast,
            "one toast per streak"
        );
        assert_eq!(
            f.status("al092026", 32.0),
            SlotStatus::Retrying { in_secs: 60 }
        );
    }

    #[test]
    fn not_found_retries_quietly_on_its_own_flat_cadence() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al942026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(
            "nhc-adeck-al942026",
            NetPoll::Done(Err("http status: 404".into())),
        );
        assert!(!f.tick(1.0, &active, &mut net).show_toast);
        assert_eq!(
            f.status("al942026", 1.0),
            SlotStatus::WaitingForGuidance {
                in_secs: NOT_FOUND_RETRY_SECS as u32,
            }
        );
        // The flat cadence does not grow with repeated 404s.
        f.tick(1.0 + NOT_FOUND_RETRY_SECS, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        net.respond(
            "nhc-adeck-al942026",
            NetPoll::Done(Err("http status: 404".into())),
        );
        f.tick(2.0 + NOT_FOUND_RETRY_SECS, &active, &mut net);
        assert_eq!(
            f.status("al942026", 2.0 + NOT_FOUND_RETRY_SECS),
            SlotStatus::WaitingForGuidance {
                in_secs: NOT_FOUND_RETRY_SECS as u32,
            }
        );
    }

    #[test]
    fn deadline_cancels_hung_request_and_retries() {
        let clock = FakeClock::new();
        let mut f = AdeckFetcher::with_clock(clock.clone());
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        clock.advance(Duration::from_millis(59_900));
        f.tick(59.9, &active, &mut net);
        assert!(net.cancelled.is_empty());
        clock.advance(Duration::from_millis(100));
        assert!(f.tick(60.0, &active, &mut net).show_toast);
        assert_eq!(net.cancelled, [AL]);
        f.tick(90.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn deadline_is_measured_by_a_monotonic_clock_not_wall_clock() {
        let clock = FakeClock::new();
        let mut f = AdeckFetcher::with_clock(clock.clone());
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        // Wall clock jumps far ahead (NTP step, laptop sleep/resume, or a
        // long frame stall), but the monotonic clock the deadline is
        // measured against has barely moved.
        clock.advance(Duration::from_millis(100));
        f.tick(10_000.0, &active, &mut net);
        assert!(
            net.cancelled.is_empty(),
            "a huge wall-clock jump alone must not trip the deadline"
        );
        assert_eq!(
            f.status("al092026", 10_000.0),
            SlotStatus::Downloading,
            "the request is still genuinely in flight"
        );
        // Now the monotonic clock actually reaches the deadline.
        clock.advance(Duration::from_secs_f64(ATTEMPT_TIMEOUT_SECS));
        assert!(f.tick(10_000.1, &active, &mut net).show_toast);
        assert_eq!(net.cancelled, [AL]);
    }

    #[test]
    fn response_arriving_same_tick_as_the_deadline_is_processed_not_cancelled() {
        let clock = FakeClock::new();
        let mut f = AdeckFetcher::with_clock(clock.clone());
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        // The deadline has technically elapsed by this tick...
        clock.advance(Duration::from_secs_f64(ATTEMPT_TIMEOUT_SECS));
        // ...but a real response is sitting ready in the same tick.
        net.respond(AL, NetPoll::Done(Ok(vec![7])));
        assert!(
            !f.tick(60.0, &active, &mut net).show_toast,
            "a response that arrived must win over a deadline that also elapsed this tick"
        );
        assert_eq!(
            net.cancelled,
            [AL],
            "success still frees the id for the next refresh"
        );
        assert_eq!(
            f.take_parse_job(1),
            Some(("al092026".to_owned(), vec![7])),
            "the response was handed to the parser, not discarded as a timeout"
        );
    }

    #[test]
    fn evicted_handle_counts_as_failure() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.evict(AL);
        f.tick(1.0, &active, &mut net);
        assert!(matches!(
            f.status("al092026", 1.0),
            SlotStatus::Retrying { .. }
        ));
    }

    #[test]
    fn parse_failure_is_quiet_and_does_not_spend_the_toast_streak() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Ok(vec![0])));
        f.tick(1.0, &active, &mut net);
        f.take_parse_job(1).unwrap();
        // A parse failure (bad gzip, or a valid-but-empty a-deck for a
        // brand-new invest) is not a transport problem and must stay quiet,
        // the same as a 404 (L4) — not toast as "Network error".
        assert!(!f.parse_finished(1, false, 1.0).unwrap().show_toast);
        assert!(matches!(
            f.status("al092026", 1.0),
            SlotStatus::Retrying { .. }
        ));

        // The quiet parse failure must not have "spent" this storm's toast
        // streak: a later genuine network failure of the SAME storm still
        // toasts.
        f.tick(31.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 500".into())));
        assert!(f.tick(32.0, &active, &mut net).show_toast);

        // ...and a success afterward resets the streak as usual.
        f.tick(32.0 + BACKOFF_CAP_SECS.min(60.0), &active, &mut net);
    }

    #[test]
    fn one_storms_success_does_not_reset_another_storms_toast_streak() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let both = ids(&["al092026", "ep182026"]);
        f.tick(0.0, &both, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 500".into())));
        assert!(
            f.tick(1.0, &both, &mut net).show_toast,
            "A's first failure toasts"
        );

        // B succeeds. Under the old fetcher-wide flag, any success would
        // clear it and let A's next failure toast again even though
        // nothing about A actually changed (review S7).
        net.respond(EP, NetPoll::Done(Ok(vec![1])));
        f.tick(2.0, &both, &mut net);
        let (storm, _) = f.take_parse_job(1).unwrap();
        assert_eq!(storm, "ep182026");
        assert!(!f.parse_finished(1, true, 2.0).unwrap().show_toast);

        // A retries (same ongoing failure streak) and fails again.
        f.tick(31.0, &both, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 500".into())));
        assert!(
            !f.tick(32.0, &both, &mut net).show_toast,
            "A's own streak is still active; B's unrelated success must not re-arm it"
        );
    }

    #[test]
    fn another_storms_first_failure_still_toasts_during_an_unrelated_ongoing_failure() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let both = ids(&["al092026", "ep182026"]);
        f.tick(0.0, &both, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 500".into())));
        assert!(
            f.tick(1.0, &both, &mut net).show_toast,
            "A's first failure toasts"
        );

        // B's first failure, while A's streak is still ongoing. Under the
        // old fetcher-wide flag this stayed silent because A had already
        // "used up" the one shared toast (review S7): each storm's streak
        // must be independent.
        net.respond(EP, NetPoll::Done(Err("http status: 500".into())));
        assert!(
            f.tick(2.0, &both, &mut net).show_toast,
            "B's own first failure must toast even while A is still failing"
        );
    }

    #[test]
    fn backoff_caps_lower_than_a_healthy_refresh() {
        const _: () = assert!(
            BACKOFF_CAP_SECS < REFRESH_SECS,
            "a failing storm must not wait as long between checks as a healthy one"
        );
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        let mut now = 0.0;
        f.tick(now, &active, &mut net);
        // Fail enough times in a row to climb past where the old 900s cap
        // would have allowed, and confirm it's capped at BACKOFF_CAP_SECS.
        let mut last_retry_in_secs = 0;
        for _ in 0..6 {
            net.respond(AL, NetPoll::Done(Err("http status: 500".into())));
            now += 0.1;
            f.tick(now, &active, &mut net);
            let SlotStatus::Retrying { in_secs } = f.status("al092026", now) else {
                panic!("expected Retrying after a failure");
            };
            last_retry_in_secs = in_secs;
            now += f64::from(in_secs) + 0.1;
            f.tick(now, &active, &mut net);
        }
        assert_eq!(last_retry_in_secs, BACKOFF_CAP_SECS as u32);
    }

    #[test]
    fn removed_storm_cancels_download_and_disowns_parse() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let both = ids(&["al092026", "ep182026"]);
        f.tick(0.0, &both, &mut net);
        net.respond(EP, NetPoll::Done(Ok(vec![9])));
        f.tick(1.0, &both, &mut net);
        let (storm, _) = f.take_parse_job(5).unwrap();
        assert_eq!(storm, "ep182026");
        f.tick(2.0, &[], &mut net);
        assert!(net.cancelled.contains(&AL.to_owned()));
        assert!(!f.owns_generation(5), "late parse result must be dropped");
        assert_eq!(f.parse_finished(5, true, 3.0), None);
        assert_eq!(f.status("al092026", 2.0), SlotStatus::Idle);
    }

    #[test]
    fn hung_storm_does_not_block_other_storms() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let both = ids(&["al092026", "ep182026"]);
        f.tick(0.0, &both, &mut net);
        net.respond(EP, NetPoll::Done(Ok(vec![4])));
        f.tick(1.0, &both, &mut net);
        assert_eq!(f.take_parse_job(1), Some(("ep182026".to_owned(), vec![4])));
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Downloading);
    }

    #[test]
    fn disable_cancels_and_enable_refires() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        f.disable(&mut net);
        assert_eq!(net.cancelled, [AL]);
        assert!(!f.is_active());
        f.tick(5.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn backoff_doubles_then_caps_at_300s() {
        let table: Vec<f64> = (0..=7).map(backoff_secs).collect();
        assert_eq!(table, [0.0, 30.0, 60.0, 120.0, 240.0, 300.0, 300.0, 300.0]);
        assert_eq!(backoff_secs(u32::MAX), BACKOFF_CAP_SECS);
    }

    #[test]
    fn status_messages() {
        assert_eq!(
            status_message(&SlotStatus::Downloading),
            "Downloading model guidance\u{2026}"
        );
        assert_eq!(
            status_message(&SlotStatus::Retrying { in_secs: 30 }),
            "Model guidance unavailable, retrying in 30 s"
        );
        assert_eq!(status_message(&SlotStatus::Idle), "No model guidance yet");
        assert_eq!(
            status_message(&SlotStatus::WaitingForGuidance { in_secs: 600 }),
            "Waiting for model guidance, checking again in 600 s"
        );
    }
}
