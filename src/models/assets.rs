use {
    serde_json::{
        Value as JsonValue,
        Value::{
            Array as jArray, Bool as jBool, Null as jNull, Number as jNumber, Object as jObject,
            String as jString,
        },
    },
    std::{collections::VecDeque, io::Write as ioWrite, path::PathBuf, str::FromStr},
};

// Convenience macro for logging match arms
#[macro_export]
macro_rules! match_with_log {
    ( $val:expr, $log:expr) => {{
        $log;
        $val
    }};
}

// Supported read source options
#[derive(Debug)]
pub enum ReadFrom {
    File(PathBuf),
    Stdin,
}

// Displays either 'Stdin' or a file, if file contains non ASCII
// characters, they are replaced with � (U+FFFD)
impl std::fmt::Display for ReadFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let display = match self {
            ReadFrom::File(path) => format!(
                "File: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            ReadFrom::Stdin => format!("Stdin"),
        };

        write!(f, "{}", display)
    }
}

#[derive(Debug, PartialEq)]
pub enum RegexOn {
    Entry,
    Value,
    Type,
    Separator,
}

impl From<&str> for RegexOn {
    fn from(s: &str) -> Self {
        match s {
            "key" => RegexOn::Entry,
            "type" => RegexOn::Type,
            "sep" => RegexOn::Separator,
            "value" => RegexOn::Value,
            _ => RegexOn::Entry,
        }
    }
}

impl Default for RegexOn {
    fn default() -> Self {
        RegexOn::Entry
    }
}

pub struct RegexOptions {
    regex: regex::Regex,
    column: RegexOn,
}

impl RegexOptions {
    pub fn new(pattern: &str, column: RegexOn) -> Self {
        let regex = regex::Regex::from_str(pattern).unwrap();
        RegexOptions { regex, column }
    }

    pub fn get_regex(&self) -> &regex::Regex {
        &self.regex
    }

    pub fn get_column(&self) -> &RegexOn {
        &self.column
    }
}

// Struct for creating and holding a list of json pointers
// for arbitrary JsonValues
pub struct JsonPacket {
    object: JsonValue,
    plist: Vec<String>,
}

impl JsonPacket {
    pub fn new(object: JsonValue) -> Self {
        let plist = JsonPacket::parse_json(&object);
        JsonPacket { object, plist }
    }

    // Convenience function around write that allows for clearer flow
    // pub fn print<W: ioWrite>(&self, options: &Options, output: &mut W) {
    //     for entry in &self.plist {
    //         let data = self.object.pointer(&entry);
    //         write(options, output.by_ref(), entry, data);
    //     }
    // }

    // Unwinds the JsonValue, growing a Vec for every endpoint it finds
    // While queueing any maps or arrays for unwinding
    fn parse_json(json_value: &JsonValue) -> Vec<String> {
        let mut list: Vec<String> = Vec::new();
        let mut jqueue: VecDeque<(&JsonValue, String)> = VecDeque::new();
        jqueue.push_back((json_value, String::default()));

        loop {
            let value = jqueue.pop_front();
            match value {
                Some((jObject(map), ref s)) => {
                    for (k, v) in map.iter() {
                        let new_path = s.clone() + "/" + k;
                        if v.is_object() {
                            list.push(new_path.clone());
                        }
                        if v.is_array() {
                            list.push(new_path.clone());
                        }
                        jqueue.push_back((v, new_path));
                    }
                }
                Some((jArray(a), ref s)) => {
                    for (i, v) in a.iter().enumerate() {
                        let new_path = s.clone() + "/" + &i.to_string();
                        jqueue.push_back((v, new_path));
                    }
                }
                Some((jString(_), s)) => list.push(s),
                Some((jNumber(_), s)) => list.push(s),
                Some((jBool(_), s)) => list.push(s),
                Some((jNull, s)) => list.push(s),
                None => break,
            }
        }
        list
    }
}
