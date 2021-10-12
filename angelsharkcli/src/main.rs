use anyhow::{anyhow, Context, Error, Result};
use clap::{App, Arg, ArgMatches, SubCommand};
use csv::{QuoteStyle, WriterBuilder};
use libangelshark::{Acm, AcmRunner, Message, ParallelIterator};
use std::{
    fs::File,
    io::{stdin, stdout, BufWriter, Write},
};

fn main() -> Result<()> {
    // Parse arguments.
    let args = parse_args();

    // Collect logins.
    let path = args.value_of("config").unwrap_or("./asa.cfg");
    let logins_file =
        File::open(path).with_context(|| format!("Failed to open logins file: {}", path))?;
    let acms = Acm::from_logins(logins_file).with_context(|| "Failed to parse logins.")?;
    let inputs = Message::from_input(stdin()).with_context(|| "Failed to read input.")?;

    match args.subcommand() {
        ("test", _) => {
            // Echo the parsed logins and input.
            println!("{:?}", acms);
            println!("{:?}", inputs);
        }
        ("man", _) => {
            // Print manual pages for the given input.
            AcmRunner::new(acms, inputs)
                .manuals()
                .for_each(|(name, output)| match output {
                    Err(e) => eprintln!(
                        "{}",
                        anyhow!(e).context(format!("angelsharkcli: manual ({})", name))
                    ),
                    Ok(o) => println!("{}", o),
                });
        }
        ("print", Some(args)) => {
            // Run the input and print the output.
            let format = args.value_of("format").unwrap_or("tsv");
            let header_row = args.is_present("header_row");
            let prefix = args.value_of("prefix").unwrap_or_default();
            let to_file = args.is_present("to_file");

            AcmRunner::new(acms, inputs)
                .run()
                .filter_map(|(name, output)| match output {
                    Err(e) => {
                        eprintln!("angelsharkcli: runner ({}): {}", name, e);
                        None
                    }
                    Ok(messages) => Some((name, messages)),
                })
                .try_for_each(|(name, outputs)| {
                    for (command, datas) in outputs.into_iter().filter_map(|message| {
                        if message.command == "logoff" {
                            None
                        } else if let Some(error) = &message.error {
                            eprintln!("angelsharkcli: ossi ({}): {}", name, error);
                            None
                        } else if let Some(datas) = message.datas {
                            if header_row {
                                Some((
                                    message.command,
                                    vec![message.fields.unwrap_or_default()]
                                        .into_iter()
                                        .chain(datas.into_iter())
                                        .collect(),
                                ))
                            } else {
                                Some((message.command, datas))
                            }
                        } else {
                            None
                        }
                    }) {
                        let writer: BufWriter<Box<dyn Write>> = BufWriter::new(if to_file {
                            let filename = format!(
                                "./{}angelshark -- {} -- {}.{}",
                                prefix, name, command, format
                            );
                            let file = File::create(&filename).with_context(|| {
                                format!("Failed to create output file: {}", filename)
                            })?;
                            Box::new(file)
                        } else {
                            Box::new(stdout())
                        });

                        match format {
                            "json" => {
                                serde_json::to_writer_pretty(writer, &datas)
                                    .with_context(|| "Failed to write JSON.")?;
                            }
                            "csv" => {
                                let mut writer = WriterBuilder::new()
                                    .quote_style(QuoteStyle::Always)
                                    .from_writer(writer);

                                for data in datas {
                                    writer
                                        .write_record(&data)
                                        .with_context(|| "Failed to write CSV.")?;
                                }
                            }
                            _ => {
                                let mut writer = WriterBuilder::new()
                                    .delimiter(b'\t')
                                    .quote_style(QuoteStyle::Never)
                                    .from_writer(writer);

                                for data in datas {
                                    writer
                                        .write_record(&data)
                                        .with_context(|| "Failed to write TSV.")?;
                                }
                            }
                        }
                    }

                    Result::<(), Error>::Ok(())
                })?;
        }
        _ => {
            // Just run the input and print any errors encountered.
            AcmRunner::new(acms, inputs)
                .run()
                .for_each(|(name, output)| match output {
                    Err(e) => {
                        eprintln!(
                            "{}",
                            anyhow!(e).context(format!("angelsharkcli: runner ({})", name))
                        );
                    }
                    Ok(o) => {
                        for msg in o {
                            if let Some(e) = msg.error {
                                eprintln!(
                                    "{}",
                                    anyhow!(e)
                                        .context(format!("angelsharkcli: ossi ({})", name))
                                );
                            }
                        }
                    }
                });
        }
    }

    Ok(())
}

fn parse_args() -> ArgMatches<'static> {
    let app = App::new("Altruistic Angelshark CLI")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_about("\nReads STDIN and parses all lines as commands to be fed to one or more ACMs. When it reaches EOF, it stops parsing and starts executing the command(s) on the ACM(s). What it does with the output can be configured with subcommands and flags. If you like keeping your commands in a file, consider using the `<` to read it on STDIN. The default behavior is to run commands but print no output (for quick changes). Errors are printed on STDERR.")
        .arg(Arg::with_name("config").long("login-file").short("l").default_value("./asa.cfg").help("Set ACM login configuration file"))
        .subcommand(SubCommand::with_name("test").about("Prints parsed logins and inputs but does not run anything").long_about("Does not execute commands entered, instead prints out the ACM logins and inputs it read (useful for debugging)"))
        .subcommand(SubCommand::with_name("man").about("Prints command manual pages via `ossim` term").long_about("Reads commands on STDIN and prints their SAT manual pages on STDOUT"))
        .subcommand(SubCommand::with_name("print").about("Prints command output to STDOUT (or files) in a useful format").long_about("Runs commands on input and writes *their data entries* to STDOUT in variety of formats (and optionally to files)").arg(Arg::with_name("prefix").long("prefix").short("p").takes_value(true).requires("to_file").help("Prepend a prefix to all output filenames")).arg(Arg::with_name("to_file").short("t").long("to-file").help("Write output to separate files instead of STDOUT")).arg(Arg::with_name("header_row").short("h").long("header-row").help("Prepend header entry of hexadecimal field addresses to output")).arg(Arg::with_name("format").short("f").long("format").possible_values(&["csv", "json", "tsv"]).default_value("tsv").help("Format data should be printed in")));
    app.get_matches_safe().unwrap_or_else(|e| e.exit())
}
