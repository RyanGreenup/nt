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
    // get the absolute path of the file
    let target = file
        .canonicalize()
        .unwrap_or_else(|_| panic!("Unable to get abs path of {:?}", file));
    // For each directory we need to get the relative path of the file to search for
    // Then use ripgrep to search for that file.
    let dirs = get_dirs(Path::new(&config.note_taking_dir));

    let mut backlinks: Vec<String> = vec![];
    for d in dirs {
        // get the relative path from a file in this directory to this file
        // relative path to target from d

        let relpath = relpath(&Path::new(&d), &target);
        println!(
            "Under {:?}, the relpath to {:?} is:\n\t{:?}",
            d, target, relpath
        );

        let backlinks_under_dir = grep(&Path::new(&d), &Path::new(&relpath));
        println!("Backlinks for {:?}: {:#?}", target, backlinks_under_dir);
        for backlink in backlinks_under_dir {
            backlinks.push(backlink);
        }
    }

    for b in backlinks {
        let b = b.replace(&config.note_taking_dir, "");
        println!("{b}");
    }
}

/// Returns the relative path if possible
/// If not possible returns the absolute path
/// Input should be absolute path.
fn relpath(dir: &Path, target: &Path) -> String {
    if let Ok(dir) = dir.canonicalize() {
        if let Ok(target) = target.canonicalize() {
            let dir_components = dir.components().collect::<Vec<_>>();
            let tgt_components = target.components().collect::<Vec<_>>();

            let common_length = dir_components
                .iter()
                .zip(tgt_components.iter())
                .take_while(|&(d, t)| d == t)
                .count();

            let mut relative_path = std::iter::repeat("..")
                .take(dir_components.len() - common_length)
                .collect::<Vec<_>>();

            relative_path.extend(
                tgt_components[common_length..]
                    .iter()
                    .map(|c| c.as_os_str().to_str().unwrap()),
            );

            return relative_path.join("/");
        }

        return dir
            .to_str()
            .unwrap_or_else(|| panic!("Unable to get string for directory {:?}", dir))
            .to_string();
    }
    dir.to_str()
        .unwrap_or_else(|| panic!("Unable to get string for directory {:?}", dir))
        .to_string()
}

fn get_dirs(dir: &Path) -> Vec<String> {
    // unfortunately grep returns 1 on no files found
    // so we'll have to treat failure as no files found
    if let Ok(stduot) = cmd!("fd", "-t", "d", ".", dir).read() {
        let mut out: Vec<String> = stduot.lines().map(|x| x.to_string()).collect();
        let dir = dir.canonicalize().expect("Unable to get abs path");
        out.push(dir.to_str().unwrap().to_string());
        out
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
            eprintln!(
                "No files found (or error) for {:?}in directory {:?}",
                query, dir
            );
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
