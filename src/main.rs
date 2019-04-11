use {crate::models::Options as AppOptions, structopt::StructOpt};

mod models;

fn main() {
    let app_options = AppOptions::from_args();
    dbg!(&app_options);
    /*    if matches.is_present("type") && matches.is_present("regex_column") {
        if let Some("type") = matches.value_of("regex_column") {
            panic!("Error: Cannot regex on column 'type' it is disabled")
        }
    }

    let regex_opts = {
        let regex = match matches.value_of("regex") {
            Some(r) => Some(Regex::new(r).unwrap()),
            None => None,
        };
        let column = match matches.value_of("regex_column") {
            Some("key") => Some(RegexOn::Entry),
            Some("type") => Some(RegexOn::Type),
            Some("sep") => Some(RegexOn::Separator),
            Some("value") => Some(RegexOn::Value),
            None => None,
            Some(_) => panic!("Error: column value is not one of the allowed values"),
        };

        (regex, column)
    };
    let by_line = matches.is_present("line");
    let show_type = !matches.is_present("type");
    let separator = match matches.value_of("separator") {
        Some("c") => ",",
        Some("t") => "\t",
        Some("cs") => ", ",
        Some(s) => s,
        _ => panic!("Separator missing"),
    };
    let debug_level = match matches.occurrences_of("verbosity") {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 3,
    };
    let line_number = matches.values_of_lossy("line");

    // Place CLI options into a central location
    let options = Options::new(
        show_type,
        separator.to_owned(),
        debug_level,
        line_number,
        regex_opts,
    );

    // Set up the writer: either to stdout or a file
    let mut writer = BufWriter::new(get_writer(matches.value_of("output"), &options));

    // Processes any files in the order they were inputted to the CLI, skipping on a failed open
    // If a "-" is set as an input option will read from stdin
    // If input is omitted completely will read from stdin
    match matches.values_of("input") {
        Some(files) => {
            let mut file_list: Vec<_> = files.collect();
            file_list.dedup_by_key(|f| *f == "-");
            for file in file_list {
                let input = get_reader(Some(file));
                if input.is_ok() {
                    let status = match to_csv(&options, input.unwrap(), writer.by_ref()) {
                        Ok(res) => res,
                        Err(e) => json!({ "Error(s) encountered": formated_error(&e) }),
                    };
                    if *options.get_debug_level() >= 2 {
                        eprintln!(
                            "\n--- Finished input: {}, with status: {} ---\n==>",
                            file, status
                        );
                    }
                } else {
                    if *options.get_debug_level() >= 1 {
                        eprintln!(
                            "\n--- Error: {} could not be opened, skipping... ---\n",
                            file
                        )
                    }
                    continue;
                }
            }
        }
        None => {
            let input = ReadFrom::Stdin(io::stdin());
            let status = match to_csv(&options, input, writer.by_ref()) {
                Ok(res) => res,
                Err(e) => json!({ "Error(s) encountered": formated_error(&e) }),
            };
            if *options.get_debug_level() >= 2 {
                eprintln!("\n--- Finished stdin with status: {} ---\n==>", &status)
            }
        }
    }
    */
}
