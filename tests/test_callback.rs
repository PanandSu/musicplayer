use std::sync::{Arc, Mutex};
use rodio::stream::DeviceSinkBuilder;

#[test]
fn test_custom_error_callback() {
    let err_count = Arc::new(Mutex::new(0));
    let err_count_clone = err_count.clone();

    let builder = DeviceSinkBuilder::from_default_device().unwrap();
    let mut sink = builder
        .with_error_callback(move |_err| {
            let mut count = err_count_clone.lock().unwrap();
            *count += 1;
        })
        .open_sink_or_fallback()
        .unwrap();

    sink.log_on_drop(false);
}
