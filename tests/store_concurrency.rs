use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_delivery_telemetry::schema::{IdempotencyKey, Timestamp, TraceId};
use agent_delivery_telemetry::store::{NewEvent, Store};

#[test]
fn independent_writers_share_a_wal_database() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("adt-{}-{nonce}.sqlite", std::process::id()));
    drop(Store::open(&path).unwrap());

    let barrier = Arc::new(Barrier::new(3));
    let writers: Vec<_> = (0..2)
        .map(|index| {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut store = Store::open(path).unwrap();
                barrier.wait();
                store
                    .append(&NewEvent {
                        trace_id: TraceId::try_from("trace-4").unwrap(),
                        key: IdempotencyKey::try_from(format!("event-{index}").as_str()).unwrap(),
                        observed_at: Timestamp(index),
                        payload: "payload".to_owned(),
                    })
                    .unwrap();
            })
        })
        .collect();

    barrier.wait();
    for writer in writers {
        writer.join().unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(store.journal_mode().unwrap(), "wal");
    assert_eq!(
        store
            .load_trace(&TraceId::try_from("trace-4").unwrap())
            .unwrap()
            .len(),
        2
    );
    std::fs::remove_file(path).unwrap();
}
