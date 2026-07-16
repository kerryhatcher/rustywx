//! Live-network test for the TIGERweb state-boundary fetch, excluded from
//! normal runs. Execute explicitly with:
//! `cargo test --test borders_network -- --ignored`

#[test]
#[ignore = "requires network access to the Census TIGERweb REST API"]
fn fetches_and_caches_state_borders() {
    let path = std::env::temp_dir().join(format!(
        "rustywx-live-borders-test-{}.geojson",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path); // ensure a clean slate

    let rings = rustywx::borders::load_or_fetch(&path).expect("fetch+parse should succeed");

    assert!(!rings.is_empty(), "should return at least one border ring");
    assert!(path.exists(), "should have written a local cache file");

    std::fs::remove_file(&path).ok();
}
