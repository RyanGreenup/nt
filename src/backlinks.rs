use crate::config::Config;
use regex;
// import serde for derive magic
use duct::cmd;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn run(config: Config, file: &PathBuf, verbose: bool) {
    // For each directory we need to get the relative path of the file to search for
    // Then use ripgrep to search for that file.
    let dirs = get_dirs(Path::new(&config.note_taking_dir));
    println!("{:?}", dirs);
}

fn get_dirs(dir: &Path) -> Vec<String> {
    // unfortunately grep returns 1 on no files found
    // so we'll have to treat failure as no files found
    if let Ok(stduot) = cmd!("fd", "-t", "d", ".", dir).read() {
        stduot.lines().map(|x| x.to_string()).collect()
    } else {
        eprintln!("Non Zero Exit using fd on {:?}", dir);
        vec![]
    }
}

fn grep(dir: &Path, query: &Path) -> Vec<String> {
    // unfortunately grep returns 1 on no files found
    // so we'll have to treat failure as no files found
    match cmd!("rg", "-l", query, dir).read() {
        Ok(stdout) => stdout.lines().map(|x| x.to_string()).collect(),
        Err(_) => {
            eprintln!("No files found (or error) for {:?}", query);
            vec![]
        }
    }
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

    // TODO this should be multicore
    // No, use mapping not iter?
    fn set_backlinks(&mut self) {
        for (_path_key, note_value) in self.notes.clone() {
            println!("Setting backlinks for {:?}", note_value.path);
            // TODO should this function be called set_links
            get_links(&note_value, self);
        }
    }

    fn debug_backlink_printer(&mut self) {
        for (path_key, note_value) in self.notes.clone() {
            let p = path_key;
            let b = note_value.backlinks.clone();
            match b {
                Some(backlinks) => {
                    println!("{:?}\n\t{:?}", p, backlinks);
                    println!("\n\n");
                }
                None => {}
            }
        }
    }
}

/// Get a vector of strings that match the following conditions
///  * exts are [.md, .org, .txt, .rmd, .ipynb]
///  * ends in ext for ext in exts
///  * of the form [](filepath)
///  * [[filepath]]
///  * [[filepath.ext]]
fn get_links(note: &Note, notes_map: &mut NotesMap) {
    // This can't be a method because it's a part of the hasmap and
    // looping creates an iter that takes ownership
    // Looping over a mutable reference to then mutate is not allowed (easily)
    let exts = vec!["md", "org", "txt", "rmd", "ipynb"];

    // match any word that ends in ext
    let re = regex::Regex::new(r"\b\w+\.(md|org|txt|rmd|ipynb)\b").expect("Failed to create regex");
    if let Some(caps) = re.captures(note.content.as_str()) {
        if let Some(cap) = caps.get(0) {
            notes_map.add_backlink(note.path.clone(), cap.as_str().to_string());
        }
    }

    // Match anything inside Normal markdown links
}

#[derive(Clone)]
struct Note {
    /// The path to the file
    path: String,
    /// The content in the file
    content: String,
    /// Files that link to this one
    backlinks: Option<Vec<String>>,
}

impl Note {}

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

fn get_dirs_native(parent_directory: &Path, verbose: bool) -> Vec<String> {
    let dirs = walk(&parent_directory, verbose);
    dirs.keys().map(|x| x.to_string()).collect()
}

fn walk(filepath: &Path, verbose: bool) -> HashMap<String, Vec<String>> {
    let mut file_map = HashMap::new();
    walk_dir_get_dirs(&filepath, &mut file_map, verbose);
    for (path, _content) in &file_map {
        if verbose {
            println!("Path: {:?}", path);
        }
    }
    file_map
}

fn walk_dir_get_dirs(filepath: &Path, file_map: &mut HashMap<String, Vec<String>>, verbose: bool) {
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

        if let Ok(entries) = fs::read_dir(filepath) {
            for entry in entries {
                if let Ok(e) = entry {
                    let path = e.path();
                    walk_dir_get_dirs(&path, file_map, verbose);
                }
            }
        }
    } else {
        // In here filepath is known to be a file not a dir
        // Try to get a string
        if let Some(path_str) = filepath.to_str() {
            // Try to get parent dir of this file
            if let Some(parent_dir) = filepath.parent() {
                // Try to get that as a string
                if let Some(parent_dir) = parent_dir.to_str() {
                    // Store it in the map
                    file_map
                        // Try to get the vector out
                        .entry(parent_dir.to_owned())
                        // Otherwise add an empty vector
                        .or_insert(vec![])
                        // Push this string into that vector
                        .push(path_str.to_owned());
                }
            }
        }
    }
}
