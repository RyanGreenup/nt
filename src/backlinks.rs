use crate::config::Config;

pub fn run(config: Config) {
    let note_taking_dir = config.note_taking_dir;
    println!("Running backlinks ... {note_taking_dir}");
}

