use std::sync::Mutex;

#[macro_export]
macro_rules! mutex {
    ($x:expr) => {{
        let mut mutex = Mutex::new($x);
        mutex
    }};
}
