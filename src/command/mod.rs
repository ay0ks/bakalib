use crate::extensions::string::StringExtension;

use std::collections::HashMap;

use regex::Regex;

/// # BRC Command Parser
///
/// Properties:
///
/// * `groups`: HashMap<String, String>
pub struct CommandParser {
    groups: HashMap<String, String>,
}

const PATTERN: &str =
    r"(?P<target>:\S+ )?(?P<command>\S+) ?(\{(?P<args>\S+(?: \S+)*)\})? ?(?P<tail>:.*)?";

impl CommandParser {
    /// ### Initialize a new instance of `CommandParser`
    ///
    /// Arguments:
    ///
    /// * `source`: The string to be parsed.
    pub fn new(source: String) -> Self {
        let re = Regex::new(PATTERN).unwrap();
        let caps = re
            .captures(if source.len() > 0 {
                source.as_str()
            } else {
                "noop :XXX"
            })
            .unwrap();
        let dict: HashMap<String, String> = re
            .capture_names()
            .flatten()
            .filter_map(|n| Some((n.to_string(), caps.name(n)?.as_str().to_string())))
            .collect();

        CommandParser { groups: dict }
    }

    /// Get the target of the command
    pub fn target(&mut self) -> &str {
        self.groups["target"].as_str().trim()
    }

    /// Get the command
    pub fn command(&mut self) -> &str {
        self.groups["command"].as_str()
    }

    /// Get command arguments
    pub fn args(&mut self) -> Vec<String> {
        self.groups["args"].clone().baka_split(" ")
    }

    /// Get command tail
    pub fn tail(&mut self) -> &str {
        self.groups["tail"].as_str()
    }
}
