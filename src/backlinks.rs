use crate::config::Config;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn run(config: Config, verbose: bool) {
    let note_taking_dir = config.note_taking_dir;
    println!("Running backlinks ... {note_taking_dir}");
    let mut note_taking_dir = std::path::PathBuf::from(note_taking_dir);
    let map = walk(&mut note_taking_dir, verbose);
    map_printer(&map);
}

fn map_printer(file_map: &HashMap<String, String>) {
    for (path, content) in file_map {
        // cut the content to about 80 char
        let i = std::cmp::min(content.len(), 80);
        let content = content[..i].to_string();
        let content = content.replace("\n", "\n\t");
        let content = format!("{}\n\n...", content);
        println!("{:?}\n\t{content}", path);
    }
}

fn walk(filepath: &mut Path, verbose: bool) -> HashMap<String, String> {
    let mut file_map = HashMap::<String, String>::new();
    walk_dir(filepath, &mut file_map, verbose);
    for (path, _content) in &file_map {
        if verbose {
            println!("Path: {:?}", path);
        }
    }
    file_map
}

fn walk_dir(filepath: &Path, file_map: &mut HashMap<String, String>, verbose: bool) {
    if filepath.is_dir() {
        // ignore hidden directories
        if filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with(".")
        {
            return;
        }
        for entry in fs::read_dir(filepath).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            walk_dir(&path, file_map, verbose);
        }
    } else {
        if let Some(path_str) = filepath.to_str() {
            if let Ok(content) = std::fs::read_to_string(path_str) {
                let path_str = filepath.to_str().expect("Failed to convert path to str");
                file_map.insert(path_str.to_owned(), content);
            } else {
                if verbose {
                    eprintln!("Failed to read file: {:?}", filepath.display());
                }
            }
        }
    }
}
