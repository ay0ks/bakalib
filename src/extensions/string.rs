use rand::distributions::{Alphanumeric, DistString};

/// Extension to built-in `String` type
pub trait StringExtension {
    /// Generate random alphanumeric string
    ///
    /// Example:
    /// ```rs
    /// String::random(6);
    /// ```
    fn random(len: i64) -> String;

    /// Generate random alphanumeric string
    ///
    /// Example:
    /// ```rs
    /// let tokens = "a b c d e f".to_string().baka_split(" ");
    /// ```
    fn baka_split(&mut self, delim: &str) -> Vec<String>;

    /// Convert `String` to `&str`
    fn to_str(&mut self) -> &str;

    /// Convert `String` to `&'static str`
    fn to_static_str(&mut self) -> &'static str;
}

impl StringExtension for String {
    fn random(len: i64) -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), len.try_into().unwrap())
    }

    fn baka_split(&mut self, delim: &str) -> Vec<String> {
        self.as_mut_str()
            .split(delim)
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn to_str(&mut self) -> &str {
        &self.as_mut_str()[..]
    }

    fn to_static_str(&mut self) -> &'static str {
        Box::leak(self.clone().into_boxed_str())
    }
}
