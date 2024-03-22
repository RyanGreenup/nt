use crate::config::Config;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn run(config: Config, verbose: bool) {
    let note_taking_dir = config.note_taking_dir;
    println!("Running backlinks ... {note_taking_dir}");
    let mut note_taking_dir = std::path::PathBuf::from(note_taking_dir);
    let map = walk(&mut note_taking_dir, verbose);
    map.printer();
}

struct NotesMap {
    notes: HashMap<String, Note>,
}
impl NotesMap {
    fn printer(&self) {
        for (path_key, note_value) in &self.notes {
            // cut the content to about 80 char
            let n = 80;
            let content: String = note_value.content.chars().take(n).collect();
            let content = content.replace("\n", "\n\t");
            let content = format!("{}\n\n...", content);

            // assert_eq!(path_key, &note_value.path);
            println!("{:?}\n\t{content}", path_key);
        }
    }
}

struct Note {
    path: String,
    content: String,
    backlinks: Option<Vec<String>>,
}

impl NotesMap {
    fn new() -> Self {
        Self {
            notes: HashMap::new(),
        }
    }

    fn add(&mut self, path: String, content: String) {
        let note = Note {
            path: path.clone(),
            content,
            backlinks: Option::None,
        };
        self.notes.insert(path, note);
    }

    fn add_backlink(&mut self, from: String, to: String) {
        if let Some(note) = self.notes.get_mut(&to) {
            match note.backlinks {
                Some(ref mut backlinks) => {
                    backlinks.push(from);
                }
                None => {
                    note.backlinks = Some(vec![from]);
                }
            }
        }
    }

    fn get(&self, path: &str) -> Option<&Note> {
        self.notes.get(path)
    }
}

fn walk(filepath: &mut Path, verbose: bool) -> NotesMap {
    let mut file_map = NotesMap::new();
    walk_dir(filepath, &mut file_map, verbose);
    for (path, _content) in &file_map.notes {
        if verbose {
            println!("Path: {:?}", path);
        }
    }
    file_map
}

fn walk_dir(filepath: &Path, file_map: &mut NotesMap, verbose: bool) {
    if filepath.is_dir() {
        // ignore hidden directories
        if filepath
            .file_name()
            .unwrap_or_else(|| panic!("Unable to get Filename from {:?}", filepath))
            .to_str()
            .unwrap_or_else(|| panic!("Unable to get string from {:?}", filepath))
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
                file_map.add(path_str.to_owned(), content);
            } else {
                if verbose {
                    eprintln!("Failed to read file: {:?}", filepath.display());
                }
            }
        }
    }
}
