use async_channel::Receiver;
use std::time::Duration;

pub fn poll<T>(interval: Duration, read: impl Fn() -> Option<T> + Send + 'static) -> Receiver<T>
where
T: Clone + PartialEq + Send + 'static,
{
    let (tx, rx) = async_channel::unbounded();
    std::thread::spawn(move || {
        let mut last: Option<T> = None;
        loop {
            if let Some(v) = read() {
                if last.as_ref() != Some(&v) {
                    if tx.send_blocking(v.clone()).is_err() {
                        break;
                    }
                    last = Some(v);
                }
            }
            std::thread::sleep(interval);
        }
    });
    rx
}
