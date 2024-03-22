use std::path::PathBuf;

mod backlinks;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Search Notes
    Search {
        /// Switch to Semantic Search
        #[arg(short, long)]
        sem: bool,

        /// Reindex the notes
        #[arg(short, long)]
        reindex: bool,
    },

    /// Find a note by name
    Find {},

    /// Add a new note
    New {},

    /// Backlinks
    Backlinks { file: PathBuf },

    /// Edit a note in Neovim
    Edit {},

    /// Open a note in VS Code
    Open {},
}

fn main() {
    run();
}

fn run() {
    let config = config::Config::default();

    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Search { sem: s, reindex: r }) => {
            if *s {
                let sn = "Semantic Search";
                if *r {
                    println!("Reindexing the {sn}");
                } else {
                    println!("{sn}");
                }
            } else {
                let sn = "Tantivy Search";
                if *r {
                    println!("Reindexing the {sn}");
                } else {
                    println!("{sn}");
                }
            }
        }
        Some(Commands::Find {}) => println!("Finding..."),
        Some(Commands::New {}) => println!("Adding..."),
        Some(Commands::Backlinks { file }) => backlinks::run(config, file, cli.debug > 0),
        Some(Commands::Edit {}) => println!("Editing..."),
        Some(Commands::Open {}) => println!("Opening..."),
        None => {}
    }

    // Continued program logic goes here...
}
