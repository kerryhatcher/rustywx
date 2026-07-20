//! Live-network test, excluded from normal runs. Execute explicitly with:
//! `cargo test --test network -- --ignored`

#[tokio::test]
#[ignore = "requires network access to the NEXRAD AWS archive"]
async fn fetches_and_decodes_latest_kjgx_volume() {
    let scan = rustywx::data::fetch_latest_scan("KJGX", None)
        .await
        .expect("fetch+decode should succeed")
        .expect("first fetch should return a scan");

    assert!(
        !scan.reflectivity.is_empty(),
        "volume should contain reflectivity sweeps"
    );
    let sweep = &scan.reflectivity[0];
    assert!(
        sweep.radials.len() > 300,
        "a sweep should have hundreds of radials"
    );

    // Sanity: decoded dBZ values fall in a plausible range.
    let values: Vec<f32> = sweep
        .radials
        .iter()
        .flat_map(|r| r.gates.iter().flatten().copied())
        .collect();
    assert!(
        values.iter().all(|v| (-35.0..=95.0).contains(v)),
        "dBZ range check failed"
    );
}
