use crate::config::Config;
// import serde for derive magic
use duct::cmd;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn run(config: Config, file: &PathBuf, absolute: bool, nested: bool, verbose: bool) {
    // get the absolute path of the file
    let target = file
        .canonicalize()
        .unwrap_or_else(|_| panic!("Unable to get abs path of {:?}", file));

    // Get the directories under the slipbox
    // For each directory the backlinks will be relative
    let notes_dir = Path::new(&config.note_taking_dir);
    let dirs = get_dirs(notes_dir);

    let backlinks = match nested {
        true => dirs
            .into_iter()
            .flat_map(|d| {
                // Get the relative path
                let relpath = relpath(&Path::new(&d), &target);
                grep(&Path::new(&d), &Path::new(&relpath), verbose)
            })
            .collect(),
        false => {
            // Get the relative path
            let relpath = relpath(notes_dir, &target);

            grep(notes_dir, &Path::new(&relpath), verbose)
        }
    };

    // Print the backlinks (assuming they are relative to this dir)
    for b in backlinks {
        if absolute {
            println!("{b}");
        } else {
            let mut relpath = b.replace(&config.note_taking_dir, "");
            // drop the leading slash
            if relpath.starts_with("/") {
                relpath = relpath[1..].to_string();
            }
            println!("{relpath}");
        }
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

fn grep(dir: &Path, query: &Path, verbose: bool) -> Vec<String> {
    // unfortunately grep returns 1 on no files found
    // so we'll have to treat failure as no files found
    match cmd!("rg", "-l", query, dir).read() {
        Ok(stdout) => stdout.lines().map(|x| x.to_string()).collect(),
        Err(_) => {
            if verbose {
                eprintln!("Non Zero Exit using rg on {:?}", dir);
            }
            vec![]
        }
    }
}

// Allow dead code in the event that we want to avoid shelling out
// ripgrep and fd are very fast and efficient, so this is unlikely
#[allow(dead_code)]
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

#[allow(dead_code)]
fn grep_native(dir: &Path, query: &str, verbose: bool) -> Vec<String> {
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let file_path = entry.path();

                        // check if query is in content
                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                            if content.contains(query) {
                                results.push(file_path.to_str().unwrap().to_string());
                            }
                        } else if verbose {
                            eprintln!("Failed to read {:?}", file_path);
                        }
                    }
                }
            } else if verbose {
                eprintln!("Failed to process an entry in {:?}", dir);
            }
        }
    } else if verbose {
        eprintln!("Failed to open {:?}", dir);
    }
    results
}
