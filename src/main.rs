use std::path::PathBuf;

mod utils;
use duct::cmd;
use utils::fzf_choose;

mod backlinks;
mod config;
mod tantivy_search;

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

    /// Use FZF
    #[arg(short, long)]
    fzf: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Search Notes
    Search {
        /// The search Query
        query: String,

        /// Switch to Semantic Search
        #[arg(short, long)]
        sem: bool,

        /// Reindex the notes
        #[arg(short, long)]
        reindex: bool,

        /// Initialize the index
        #[arg(short, long)]
        init: bool,
    },

    /// Find a note by name
    Find {},

    /// Add a new note
    New {},

    /// Backlinks
    Backlinks {
        file: Option<PathBuf>,

        /// Print the backlinks in absolute paths rather than relative
        #[arg(short, long)]
        absolute: bool,

        /// Specify Notes are in nested heirarchy (default assumes flat directory)
        #[arg(short, long)]
        nested: bool,
    },

    /// Edit a note in Neovim
    Edit {},

    /// Open a note in VS Code
    Open {},
}

fn main() {
    run();
}

fn run() {
    let cli = Cli::parse();

    let config = config::Config::default();
    let verbose = cli.debug > 0;

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
        0 => {}
        1 => {}
        2 => {}
        _ => {}
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Search {
            sem: s,
            reindex: r,
            query,
            init,
        }) => {
            if *s {
                let sn = "Semantic Search";
                if *r {
                    println!("Reindexing the {sn}");
                } else {
                    println!("{sn}");
                }
            } else {
                let sn = "Tantivy Search";
                if cli.fzf {
                    tantivy_search::run(config, verbose, *r, query, *init);
                } else {
                    /*

                    sk -m -i -c tantivy search -i $index --query "$argv" | jq '.path[]' | sort -u | tac | tr -d '"'                                                    \
                        --bind pgup:preview-page-up,pgdn:preview-page-down      \
                        --preview "bat --style grid --color=always                  \
                                            --terminal-width 80 $notes_dir/{+}      \
                                            --italic-text=always                    \
                                            --decorations=always" | sed "s#^#$notes_dir/#"
                                   */
                    // TODO this should support relative and absolute
                    // let skim_command = r#"tantivy search -i /home/ryan/.cache/rust_nt/Notes/slipbox/slipbox --query '{}' | jq '.path[]' |  tr -d '"'"#;
                    // TODO get the cache directory somehow
                    // TODO move cache directory to config file probably (automation is the root of all evil)
                    // get home directory
                    let home = std::env::var("HOME").expect("HOME not set");
                    let cache_dir = format!("{home}/.cache/rust_nt/Notes/slipbox/slipbox");
                    let mut skim_command: String =
                        format!("tantivy search -i {cache_dir} --query ").into();
                    skim_command.push_str(r#"'{}' | jq '.path[]' |  tr -d '"'"#);

                    cmd!(
                        "sk",
                        "-m",
                        "-i",
                        "-c",
                        skim_command,
                        "--preview",
                        r#"bat --color=always {}"#
                    )
                    .run()
                    .expect("Unable to run sk");
                }
            }
        }
        Some(Commands::Find {}) => println!("Finding..."),
        Some(Commands::New {}) => println!("Adding..."),
        Some(Commands::Backlinks {
            file,
            absolute,
            nested,
        }) => {
            let f: PathBuf;
            if !cli.fzf {
                f = match file {
                    Some(_) => panic!("Cannot specify file with FZF"),
                    None => fzf_choose(config.note_taking_dir.as_str()),
                }
            } else {
                f = match file {
                    Some(f) => f.clone(),
                    None => fzf_choose(config.note_taking_dir.as_str()),
                }
            }
            backlinks::run(config, &f, *absolute, *nested, cli.debug > 0)
        }
        Some(Commands::Edit {}) => println!("Editing..."),
        Some(Commands::Open {}) => println!("Opening..."),
        None => {}
    }

    // Continued program logic goes here...
}

// TODO
// Search maybe should have optional query
// for the reindex and reinit
