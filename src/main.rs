// #![deny(missing_docs)]
mod event;
mod git;
mod processor;

use crate::processor::{ProcessOptions, Processor};
use crate::{git::GitInfo, processor::Addon};
use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::io::Write;
use std::ops::Deref;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    path::Path,
};

#[derive(Debug, Parser)]
#[clap(name = "Markdown Test Reporter", version, about, author, long_about = None)]
struct Cli {
    /// The filename of the JSON test data. Unnecessary or unparsable lines will be ignored
    #[clap(value_parser, default_value_t = String::from("test-output.json"))]
    input: String,
    /// The name of the output file
    #[clap(short, long, value_parser)]
    output: Option<String>,
    /// Disable report metadata
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    disable_front_matter: bool,
    /// git top-level location
    #[clap(short, long, value_parser, default_value_t = String::from("."))]
    git: String,
    /// Show only the summary section
    #[clap(short, long, action)]
    summary: bool,
    /// Be quiet
    #[clap(short, long, conflicts_with = "verbose")]
    quiet: bool,
    /// Be more verbose. May be repeated multiple times
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Disable extracting git information
    #[clap(short, long, action = clap::ArgAction::SetTrue, conflicts_with = "git")]
    no_git: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // let cmd = Cli::command();
    // let args = cmd.get_matches();
    // let cli:Cli = args.into();
    // let args = cmd.get_matches_from()
    // let a:ArgMatches = cli.into();
    // let b: ArgMatches = ArgMatches::from(cli);
    // let mut args = cmd.get_matches_from(cmd);

    // Parse filepaths
    let input_path= Path::new(&cli.input);
    log::debug!("input_path: {}", input_path.display());

    let file_stem = input_path.file_stem().ok_or_else(|| anyhow::anyhow!("unable to parse input filename")).unwrap().to_str().unwrap();
    log::debug!("file_stem: {}", file_stem);

    let output_file= match cli.output {
        Some(o) => o,
        None => (String::from(file_stem) + ".md")
    };

    let mut addons = Vec::<Box<dyn Addon>>::new();


    if !cli.no_git {
        addons.push(Box::new(GitInfo::new(Path::new(&cli.git), cli.git != String::from("."))))
    }

    let log_level = match (cli.quiet, cli.verbose) {
        (true, _) => LevelFilter::Off,
        (_, 0) => LevelFilter::Warn,
        (_, 1) => LevelFilter::Info,
        (_, 2) => LevelFilter::Debug,
        (_, _) => LevelFilter::Trace,
    };

    TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )?;

    log::debug!("Reading from: {}", input_path.display());
    log::debug!("Writing to: {}", output_file);

    let input = File::open(input_path)?;
    let reader = BufReader::new(input);

    let output: Box<dyn Write> = match output_file.deref() {
        "-" => Box::new(std::io::stdout()),
        output => Box::new(File::create(output)?),
    };
    let writer = BufWriter::new(output);

    {
        let mut processor = Processor::new(
            writer,
            ProcessOptions {
                disable_front_matter: cli.disable_front_matter,
                addons,
                summary: cli.summary 
            },
        );

        for line in reader.lines() {
            processor.line(&line?)?;
        }
    }

    Ok(())
}


#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}