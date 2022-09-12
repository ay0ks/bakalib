use std::thread;
use std::time::Duration;

pub fn set_interval(interval: i32, mut delegate: Box<dyn FnMut(i32) -> bool + Send>) {
    thread::spawn(move || {
        let mut i = 0;

        loop {
            if delegate(i) {
                break;
            }

            i += 1;

            thread::sleep(Duration::from_millis((interval * 1000).try_into().unwrap()));
        }
    });
}
