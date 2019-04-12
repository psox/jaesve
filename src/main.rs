use {
    crate::models::{get_reader, get_writer, Options as AppOptions},
    std::io::BufWriter,
    structopt::StructOpt,
};

mod models;

fn main() {
    let app_options = AppOptions::from_args();
    dbg!(&app_options);

    // Set up the writer: either to stdout or a file
    let mut writer = BufWriter::new(get_writer(&app_options.output));

    for input_file in app_options.input {
        let reader = get_reader(&input_file);
    }

    /*

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
