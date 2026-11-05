use agent_delivery_telemetry::store::Store;

#[test]
fn opening_store_applies_current_migration() {
    let store = Store::open_in_memory().expect("open store");

    assert_eq!(store.migration_version().unwrap(), 1);
}
