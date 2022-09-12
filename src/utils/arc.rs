use std::sync::Arc;

#[macro_export]
macro_rules! arc {
    ($x:expr) => {{
        let mut arc = Arc::new($x);
        arc
    }};
}
