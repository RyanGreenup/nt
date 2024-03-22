use std::path::PathBuf;

mod utils;
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
        Some(Commands::Search { sem: s, reindex: r, query, init }) => {
            if *s {
                let sn = "Semantic Search";
                if *r {
                    println!("Reindexing the {sn}");
                } else {
                    println!("{sn}");
                }
            } else {
                let sn = "Tantivy Search";
                tantivy_search::run(config, verbose, *r, query, *init);
                if *r {
                    println!("Reindexing the {sn}");
                } else {
                    println!("{sn}");
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
            if cli.fzf {
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



/*
   TODO Implement for Tantivy Search

   */

/*

    function __tantivy_search
        set index $HOME/Sync/Projects/2024/tantivy-search/slipbox_index
        tantivy search -i $index --query "$argv" | jq '.path[]' | sort -u | tac | tr -d '"'
    end
    function __tantivy_skim
        set notes_dir $argv
        cd $notes_dir

        sk -m -i -c '__tantivy_search {}'                           \
            --bind pgup:preview-page-up,pgdn:preview-page-down      \
            --preview "bat --style grid --color=always                  \
                                --terminal-width 80 $notes_dir/{+}      \
                                --italic-text=always                    \
                                --decorations=always" | sed "s#^#$notes_dir/#"
    end

    __tantivy_search ~/Notes/slipbox
   */

// TODO
// Search maybe should have optional query
// for the reindex and reinit
