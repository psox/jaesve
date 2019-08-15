use {
    crate::models::error::ErrorKind,
    serde_json::{
        from_slice, Value as JsonValue,
        Value::{
            Array as jArray, Bool as jBool, Null as jNull, Number as jNumber, Object as jObject,
            String as jString,
        },
    },
    std::{
        collections::VecDeque,
        convert::TryFrom,
        error::Error,
        io::{Result as ioResult, Write as ioWrite},
        path::PathBuf,
        str::FromStr,
    },
};

/// Convenience macro for logging match arms
#[macro_export]
macro_rules! match_with_log {
    ( $val:expr, $log:expr) => {{
        $log;
        $val
    }};
}

/// Supported read source options
#[derive(Debug)]
pub enum ReadFrom {
    File(PathBuf),
    Stdin,
}

// Displays either 'Stdin' or a file name, if file name contains non ASCII
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

pub trait Builder {
    type Block: std::fmt::Display;
    type Error: Error;

    fn identifer(&self) -> Result<Self::Block, Box<dyn self::Error>>;

    fn delimiter(&self) -> Result<Self::Block, Box<dyn self::Error>>;

    fn r#type(&self) -> Result<Self::Block, Box<dyn self::Error>>;

    fn pointer(&self) -> Result<Self::Block, Box<dyn self::Error>>;

    fn value(&self) -> Result<Self::Block, Box<dyn self::Error>>;
}

#[derive(Debug)]
pub enum BlockKind {
    Ident(usize),
    Delimiter(char),
    Type(JType),
    Pointer(String),
    Value(Option<String>),
}

impl std::fmt::Display for BlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BlockKind::Ident(i) => write!(f, "{}", i),
            BlockKind::Delimiter(d) => write!(f, "{}", d),
            BlockKind::Type(t) => write!(f, "{}", t),
            BlockKind::Pointer(p) => write!(f, "{}", p),
            BlockKind::Value(v) => write!(f, "{}", v.as_ref().unwrap_or(&String::default())),
        }
    }
}

#[derive(Debug)]
pub struct Output {
    blocks: Vec<BlockKind>,
}

impl Output {
    fn get_ident(&self) -> Option<BlockKind> {
        self.blocks.iter().find_map(|kind| match kind {
            BlockKind::Ident(i) => Some(BlockKind::Ident(*i)),
            _ => None,
        })
    }

    fn get_delimiter(&self) -> Option<BlockKind> {
        self.blocks.iter().find_map(|kind| match kind {
            BlockKind::Delimiter(d) => Some(BlockKind::Delimiter(*d)),
            _ => None,
        })
    }

    fn get_type(&self) -> Option<BlockKind> {
        self.blocks.iter().find_map(|kind| match kind {
            BlockKind::Type(t) => Some(BlockKind::Type(*t)),
            _ => None,
        })
    }

    fn get_pointer(&self) -> Option<BlockKind> {
        self.blocks.iter().find_map(|kind| match kind {
            BlockKind::Pointer(p) => Some(BlockKind::Pointer(p.clone())),
            _ => None,
        })
    }

    fn get_value(&self) -> Option<BlockKind> {
        self.blocks.iter().find_map(|kind| match kind {
            BlockKind::Value(v) => Some(BlockKind::Value(v.clone())),
            _ => None,
        })
    }
}

impl Builder for Output {
    type Block = BlockKind;
    type Error = ErrorKind;

    fn identifer(&self) -> Result<Self::Block, Box<dyn self::Error>> {
        self.get_ident().ok_or(Box::new(ErrorKind::Generic))
    }

    fn delimiter(&self) -> Result<Self::Block, Box<dyn self::Error>> {
        self.get_delimiter().ok_or(Box::new(ErrorKind::Generic))
    }

    fn r#type(&self) -> Result<Self::Block, Box<dyn self::Error>> {
        self.get_type().ok_or(Box::new(ErrorKind::Generic))
    }

    fn pointer(&self) -> Result<Self::Block, Box<dyn self::Error>> {
        self.get_pointer().ok_or(Box::new(ErrorKind::Generic))
    }

    fn value(&self) -> Result<Self::Block, Box<dyn self::Error>> {
        self.get_value().ok_or(Box::new(ErrorKind::Generic))
    }
}

pub struct OutputBuilder {
    blocks: [Option<BlockKind>; 5],
}

impl OutputBuilder {
    pub fn new() -> Self {
        let blocks: [Option<BlockKind>; 5] = Default::default();
        Self { blocks }
    }

    pub fn done(mut self) -> Output {
        let mut blocks = Vec::new();
        for opt in &mut self.blocks {
            if opt.is_some() {
                let block = std::mem::replace(opt, None);
                blocks.push(block.unwrap())
            }
        }

        Output { blocks }
    }

    pub fn ident(mut self, id: usize) -> Self {
        self.blocks[0] = Some(BlockKind::Ident(id));
        self
    }

    pub fn delim(mut self, delim: char) -> Self {
        self.blocks[1] = Some(BlockKind::Delimiter(delim));
        self
    }

    pub fn type_of(mut self, jtype: JType) -> Self {
        self.blocks[2] = Some(BlockKind::Type(jtype));
        self
    }

    pub fn pointer(mut self, jptr: String) -> Self {
        self.blocks[3] = Some(BlockKind::Pointer(jptr));
        self
    }

    pub fn value(mut self, val: Option<String>) -> Self {
        self.blocks[4] = Some(BlockKind::Value(val));
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JType {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

impl From<JsonValue> for JType {
    fn from(json: JsonValue) -> Self {
        match json {
            jObject(_) => JType::Object,
            jArray(_) => JType::Array,
            jString(_) => JType::String,
            jNumber(_) => JType::Number,
            jBool(_) => JType::Bool,
            jNull => JType::Null,
        }
    }
}

impl From<&JsonValue> for JType {
    fn from(json: &JsonValue) -> Self {
        match json {
            jObject(_) => JType::Object,
            jArray(_) => JType::Array,
            jString(_) => JType::String,
            jNumber(_) => JType::Number,
            jBool(_) => JType::Bool,
            jNull => JType::Null,
        }
    }
}

impl std::fmt::Display for JType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let kind = match self {
            JType::Object => "Object",
            JType::Array => "Array",
            JType::String => "String",
            JType::Number => "Number",
            JType::Bool => "Bool",
            JType::Null => "Null",
        };

        write!(f, "{}", kind)
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
        // Checked by clap, unwrap here is safe
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

pub struct JsonScan<I> {
    iter: I,
    prev: Option<u8>,
    state: ScanState,
    /// (InQuotes, OutQuotes)
    offsets: (usize, usize),
}

impl<I> JsonScan<I>
where
    I: Iterator<Item = ioResult<u8>>,
{
    pub fn new(iter: I) -> JsonScan<I> {
        JsonScan {
            iter,
            prev: None,
            state: ScanState::OutQuotes,
            offsets: (0, 0),
        }
    }

    pub fn outside_quotes(&self) -> bool {
        match self.state {
            ScanState::OutQuotes => true,
            ScanState::InQuotes => false,
        }
    }

    pub fn offsets(&self) -> (usize, usize) {
        self.offsets
    }

    fn handle_state(&mut self) {
        match self.prev {
            Some(b'\\') => (),
            _ => match self.state {
                ScanState::InQuotes => {
                    self.offsets.1 = 0; // Reset OutQuotes counter
                    self.state = ScanState::OutQuotes
                }
                ScanState::OutQuotes => {
                    self.offsets.0 = 0; // Reset InQuotes counter
                    self.state = ScanState::InQuotes
                }
            },
        }
    }

    fn increment_offset(&mut self) {
        match self.state {
            ScanState::InQuotes => self.offsets.0 += 1,
            ScanState::OutQuotes => self.offsets.1 += 1,
        }
    }
}

impl<I> Iterator for JsonScan<I>
where
    I: Iterator<Item = ioResult<u8>>,
{
    type Item = ioResult<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(b @ b'"')) => {
                self.handle_state();
                //self.offset(); Should starting a new offset be 0 or 1?
                self.prev = Some(b);
                Some(Ok(b))
            }
            Some(Ok(b)) => {
                self.increment_offset();
                self.prev = Some(b);
                Some(Ok(b))
            }
            Some(Err(e)) => {
                self.increment_offset();
                self.prev = None;
                Some(Err(e))
            }
            None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ScanState {
    InQuotes,
    OutQuotes,
}

pub struct JsonPointer<'j> {
    ident: usize,
    queue: VecDeque<(&'j JsonValue, String)>,
    pbuf: Vec<OutputBuilder>,
}

impl<'j> JsonPointer<'j> {
    pub fn new(json: &'j JsonValue, meta: (usize, String, Option<usize>)) -> Self {
        let (mut queue, pbuf) = match meta.2 {
            Some(hint) => (VecDeque::with_capacity(hint), Vec::with_capacity(hint)),
            None => (VecDeque::new(), Vec::new()),
        };

        queue.push_back((json, meta.1));

        Self {
            ident: meta.0,
            queue,
            pbuf,
        }
    }

    pub fn parse_next(&mut self) -> Option<OutputBuilder> {
        loop {
            let value = self.queue.pop_front();
            match value {
                Some((jObject(map), ref s)) => {
                    for (k, v) in map.iter() {
                        let new_path = s.clone() + "/" + k;
                        if v.is_object() {
                            self.pbuf.push(
                                OutputBuilder::new()
                                    .ident(self.ident)
                                    .pointer(new_path.clone())
                                    .value(None)
                                    .type_of(value.as_ref().unwrap().0.into()),
                            );
                        }
                        if v.is_array() {
                            self.pbuf.push(
                                OutputBuilder::new()
                                    .ident(self.ident)
                                    .pointer(new_path.clone())
                                    .value(None)
                                    .type_of(value.as_ref().unwrap().0.into()),
                            );
                        }
                        self.queue.push_back((v, new_path));
                    }
                }
                Some((jArray(a), ref s)) => {
                    for (i, v) in a.iter().enumerate() {
                        let new_path = s.clone() + "/" + &i.to_string();
                        self.queue.push_back((v, new_path));
                    }
                }
                Some((jString(val), ref jptr)) => {
                    self.pbuf.push(
                        OutputBuilder::new()
                            .ident(self.ident)
                            .pointer(String::from(jptr))
                            .value(Some(val.to_string()))
                            .type_of(value.as_ref().unwrap().0.into()),
                    );
                    break;
                }
                Some((jNumber(val), ref jptr)) => {
                    self.pbuf.push(
                        OutputBuilder::new()
                            .ident(self.ident)
                            .pointer(String::from(jptr))
                            .value(Some(val.to_string()))
                            .type_of(value.as_ref().unwrap().0.into()),
                    );
                    break;
                }
                Some((jBool(val), ref jptr)) => {
                    self.pbuf.push(
                        OutputBuilder::new()
                            .ident(self.ident)
                            .pointer(String::from(jptr))
                            .value(Some(val.to_string()))
                            .type_of(value.as_ref().unwrap().0.into()),
                    );
                    break;
                }
                Some((tp @ jNull, jptr)) => {
                    self.pbuf.push(
                        OutputBuilder::new()
                            .ident(self.ident)
                            .pointer(jptr)
                            .value(Some(String::from("null")))
                            .type_of(tp.into()),
                    );
                    break;
                }
                None => break,
            }
        }
        self.pbuf.pop()
    }
}

impl<'j> Iterator for JsonPointer<'j> {
    type Item = OutputBuilder;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next()
    }
}

pub struct JsonPacket {
    ident: usize,
    base_path: String,
    json: JsonValue,
}

impl JsonPacket {
    fn size_hint(&self) -> Option<usize> {
        match self.json {
            jObject(ref val) => match val.iter().size_hint() {
                (_, Some(ub)) => Some(ub),
                (lb, None) => Some(lb),
            },
            jArray(ref val) => Some(val.len()),
            _ => None,
        }
    }

    pub fn into_inner(self) -> (JsonValue, (usize, String, Option<usize>)) {
        let hint = self.size_hint();
        (self.json, (self.ident, self.base_path, hint))
    }
}

impl TryFrom<(usize, Option<Vec<u8>>, Vec<u8>)> for JsonPacket {
    type Error = ErrorKind;

    fn try_from(
        packet: (usize, Option<Vec<u8>>, Vec<u8>),
    ) -> std::result::Result<Self, Self::Error> {
        let base_path: String = from_slice(packet.1.unwrap_or_default().as_slice())?;
        let json: JsonValue = from_slice(packet.2.as_slice())?;

        Ok(JsonPacket {
            ident: packet.0,
            base_path,
            json,
        })
    }
}

// pub fn parse_json(&'j mut self) {
//     match self.item.size_hint() {
//         Some(hint) => self.queue.reserve(hint),
//         None => (),
//     }
//     let path = self.item.base_path.clone();
//     self.queue.push_back((&self.item.json, path));

//     loop {
//         let value = self.queue.pop_front();
//         match value {
//             Some((jObject(map), ref s)) => {
//                 for (k, v) in map.iter() {
//                     let new_path = s.clone() + "/" + k;
//                     if v.is_object() {
//                         self.pbuf.push(new_path.clone());
//                     }
//                     if v.is_array() {
//                         self.pbuf.push(new_path.clone());
//                     }
//                     self.queue.push_back((v, new_path));
//                 }
//             }
//             Some((jArray(a), ref s)) => {
//                 for (i, v) in a.iter().enumerate() {
//                     let new_path = s.clone() + "/" + &i.to_string();
//                     self.queue.push_back((v, new_path));
//                 }
//             }
//             Some((jString(_), s)) => self.pbuf.push(s),
//             Some((jNumber(_), s)) => self.pbuf.push(s),
//             Some((jBool(_), s)) => self.pbuf.push(s),
//             Some((jNull, s)) => self.pbuf.push(s),
//             None => break,
//         }
//     }
// }
