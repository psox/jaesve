use {
    clap::arg_enum,
    failure::Error as fError,
    serde_json::{
        json, Value as JsonValue,
        Value::{
            Array as jArray, Bool as jBool, Null as jNull, Number as jNumber, Object as jObject,
            String as jString,
        },
    },
    std::{
        collections::VecDeque,
        fs::File,
        io::{BufRead, Write},
        result,
    },
    structopt::StructOpt,
};

arg_enum! {
    #[derive(Debug)]
    pub enum ColumnNames {
        Path,
        Type,
        Value,
        Index
    }
}

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab_case")]
pub struct Options {
    #[structopt(short = "t", long)]
    /// Print without json object type
    hide_type: bool,
    #[structopt(short, long)]
    /// Print a header row
    print_header: bool,
    #[structopt(
        short,
        long,
        raw(possible_values = r##"&["0","1","2","3"]"##),
        default_value = "0"
    )]
    /// Sets level of debug output
    verbose: u8,
    #[structopt(short, long, default_value = ", ")]
    /// The field separator to use
    /// \t => tab
    separator: String,
    #[structopt(short, long, default_value = "\"")]
    /// Beginning delimiter for the fields
    left_delimiter: String,
    #[structopt(short, long, default_value = "\"")]
    /// End delimiter for the fields
    right_delimiter: String,
    #[structopt(short, long, default_value = "\n")]
    /// End delimiter for the record
    end_of_record: String,
    #[structopt(short, long, default_value = "")]
    /// Start delimiter for the record
    beginning_of_record: String,
    #[structopt(short, long)]
    /// Enable multi document processing and add an index
    /// column to the front of the output starting at the
    /// line provided
    multi_documents: Option<i64>,
    #[structopt(short = "x", long, default_value = ".*")]
    /// Search regular expression
    regex: String,
    #[structopt(
        short,
        long,
        raw(
            possible_values = "&ColumnNames::variants()",
            case_insensitive = "true"
        ),
        default_value = "Value"
    )]
    /// Column for regex to apply to
    column: ColumnNames,
    #[structopt(short, long, default_value = "-")]
    /// List of input file names where '-' => <STDIN>
    pub input: Vec<String>,
    #[structopt(short, long, default_value = "-")]
    /// List of output file where '-' => <STDOUT>
    pub output: String,
}

type FailureResult<T> = result::Result<T, fError>;

// Opens a write stream to either stdout or a file, depending on the user
// If it can't open a file it will attempt to create it
// If it can't create it, will default to stdout
pub fn get_writer(file_name: &str) -> Box<Write> {
    if file_name == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(File::create(file_name).unwrap())
    }
}

// Either opens from a file or stdin if the "filename" is "-"
pub fn get_reader(file_name: &str) -> Result<ReadFrom, String> {
    if file_name == "-" {
        Ok(ReadFrom::Stdin(std::io::stdin()))
    } else {
        match File::open(file_name) {
            Ok(file) => Ok(ReadFrom::File(file)),
            Err(error) => Err(error.to_string()),
        }
    }
}

// Puts all the pieces together
pub fn to_csv<W: Write>(
    options: &Options,
    input: ReadFrom,
    mut output: W,
) -> FailureResult<JsonValue> {
    match input {
        ReadFrom::File(f) => {
            let data: JsonValue = serde_json::from_reader(f)?;
            let packet = JsonPacket::new(data);
            packet.print(options, &mut output);
        }
        ReadFrom::Stdin(s) => {
            if options.multi_documents.is_some() {
                s.lock()
                    .lines()
                    .filter_map(std::result::Result::ok)
                    .filter_map(|line| {
                        let data = serde_json::from_str(line.as_str());
                        data.ok()
                    })
                    .for_each(|value: JsonValue| {
                        let packet = JsonPacket::new(value);
                        packet.print(options, &mut output);
                    })
            } else {
                let data: JsonValue = serde_json::from_reader(s)?;
                let packet = JsonPacket::new(data);
                packet.print(options, &mut output);
            }
        }
    }

    Ok(json!(0))
}

// Function that writes the formatted output to the writer
// The work-horse of the rebel fleet
// If something goes wrong, writes the error to stderr and moves on
fn write<W: Write>(options: &Options, mut output: W, entry: &str, val: Option<&JsonValue>) {
    let regex = &options.regex;
    let separator = &options.separator;
    let show_type = !options.hide_type;
    let value = match val {
        Some(jObject(_)) => "".to_string(),
        Some(jArray(_)) => "".to_string(),
        Some(jString(s)) => s.to_string(),
        Some(jNumber(n)) => n.to_string(),
        Some(jBool(b)) => b.to_string(),
        Some(jNull) => "NULL".to_string(),
        None => "NO_VALUE".to_string(),
    };
    let mut formated_output = String::new();

    if show_type {
        let type_of = match val {
            Some(val) => match val {
                jObject(_) => "Map",
                jArray(_) => "Array",
                jString(_) => "String",
                jNumber(_) => "Number",
                jBool(_) => "Bool",
                jNull => "Null",
            },
            None => "NO_TYPE",
        };
        let fmt = format!(
            r##""{}"{}"{}"{}"{}""##,
            entry, separator, type_of, separator, value
        );
        formated_output.push_str(&fmt);
    } else {
        let fmt = format!(r##""{}"{}"{}""##, entry, separator, value);
        formated_output.push_str(&fmt);
    }
    // match regex_opts.get_regex() {
    //     Some(r) => {
    //         let column = match regex_opts.get_column() {
    //             Some(RegexOn::Entry) => entry,
    //             Some(RegexOn::Value) => value.as_str(),
    //             Some(RegexOn::Type) => match val {
    //                 Some(val) => match val {
    //                     jObject(_) => "Map",
    //                     jArray(_) => "Array",
    //                     jString(_) => "String",
    //                     jNumber(_) => "Number",
    //                     jBool(_) => "Bool",
    //                     jNull => "Null",
    //                 },
    //                 None => "NO_TYPE",
    //             },
    //             Some(RegexOn::Separator) => separator,
    //             None => panic!("Error: Need a column to regex match on"),
    //         };

    //     if r.is_match(column) {
    //         writeln!(output.by_ref(), "{}", formated_output.as_str())
    //             .map_err(|e| eprintln!("An error occurred while writing: {}", e))
    //             .unwrap_or(())
    //     }
    // }
    // None => writeln!(output.by_ref(), "{}", formated_output.as_str())
    //     .map_err(|e| eprintln!("An error occurred while writing: {}", e))
    //     .unwrap_or(()),
    // }
}

// Small function for formatting any error (chains) failureResult catches
pub fn formated_error(err: &::failure::Error) -> String {
    let mut format = err.to_string();
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        format.push_str(": ");
        format.push_str(&next.to_string());
        prev = next;
    }
    format
}

pub enum ReadFrom {
    File(std::fs::File),
    Stdin(std::io::Stdin),
}

// Struct for creating and holding a list of json pointers
// for arbitrary JsonValues
struct JsonPacket {
    object: JsonValue,
    plist: Vec<String>,
}

impl JsonPacket {
    pub fn new(object: JsonValue) -> Self {
        let plist = JsonPacket::parse_json(&object);
        JsonPacket { object, plist }
    }

    // Convenience function around write that allows for clearer flow
    pub fn print<W: Write>(&self, options: &Options, output: &mut W) {
        for entry in &self.plist {
            let data = self.object.pointer(&entry);
            write(options, output.by_ref(), entry, data);
        }
    }

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
