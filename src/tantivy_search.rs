use crate::config::Config;
use duct::cmd;

use serde_json;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::Path;
// TODO just use fd?
use walkdir::WalkDir;

use dirs;

pub fn run(config: Config, verbose: bool, reindex: bool, query: &String) {
    let cache = get_cache(&config.note_taking_dir);

    if reindex {
        // remove the cache directory
        let _ = std::fs::remove_dir_all(&cache);
        todo!("Reindexing the cache");
    }

    //check if the cache exists
    if !std::path::Path::new(&cache).exists() {
        println!("Cache does not exist, creating it...");
        let _ = std::fs::create_dir_all(&cache);
        create_tantivy(&cache);
        index_tantivy(
            std::path::Path::new(&cache),
            std::path::Path::new(&config.note_taking_dir),
            4,
        );
    }

    tantivy_search(&query, &cache, 15);
}

fn get_cache(notes_dir: &str) -> String {
    if let Some(cache_dir) = dirs::cache_dir() {
        println!("XDG Cache Directory: {}", cache_dir.display());
        let home = format!("{}/", std::env::var("HOME").expect("HOME not set"));
        let notes_dir = notes_dir.replace(&home, "");
        format!("{}/rust_nt/{notes_dir}/slipbox", cache_dir.display())
    } else {
        panic!("Could not find XDG Cache Directory");
    }
}

fn create_tantivy(cache_dir: &str) {
    println!("Creating Tantivy index in {}", cache_dir);
    println!(
        "Ensure that the Tantivy dictionary uses these same hardcoded fields: {:?}",
        FIELDS
    );

    cmd!("tantivy", "new", "-i", cache_dir)
        .run()
        .expect("Unable to run tantivy-cli");
}

fn index_tantivy(cache_dir: &Path, notes_dir: &Path, threads: u32) {
    // Create a jsonlines file
    let jsonlines = format!(
        "{}/slipbox.jsonl",
        cache_dir
            .to_str()
            .unwrap_or_else(|| panic!("Unable to convert cache_dir to string"))
    );
    let jsonlines = Path::new(&jsonlines);
    make_jsonlines(notes_dir, jsonlines);

    cmd!("cat", jsonlines)
        .pipe(cmd!(
            "tantivy",
            "index",
            "-i",
            cache_dir,
            "-t",
            threads.to_string()
        ))
        .run()
        .expect("Unable to run tantivy-cli");
}

// TODO should probably deal with json in here
fn tantivy_search(query: &str, cache_dir: &str, n: u32) {
    cmd!("tantivy", "search", "-i", cache_dir, "--query", query)
        .pipe(cmd!("jq", ".path[]"))
        .pipe(cmd!("tr", "-d", "\""))
        .pipe(cmd!("tac"))
        .pipe(cmd!("tail", "-n", n.to_string()))
        .run()
        .expect("Unable to run tantivy-cli for the search");
}

// Create a jsonlines file

/// This uses to fields
///     path: the path to the file
///     content: the content of the file
/// Ensure that the Tantivy dictionary uses these same hardcoded fields
fn make_jsonlines(dir_path: &Path, output_path: &Path) {
    let home = env::var("HOME").unwrap_or_else(|_| panic!("HOME environment variable not set"));

    // Get all markdown files
    let md_files: Vec<_> = WalkDir::new(dir_path)
        .into_iter()
        // Filter for valid entries
        .filter_map(|entry| entry.ok())
        // Filter for md files
        .filter(|entry| entry.path().extension() == Some(OsStr::new("md")))
        // Convert to string
        .map(|entry| entry.path().display().to_string())
        .collect();

    // Build a dictionary
    let d_list: Vec<HashMap<String, String>> = md_files
        .iter()
        .filter_map(|file| {
            let path = file.to_string();
            let content_result = std::fs::read_to_string(file);
            match content_result {
                Ok(content) => Some(HashMap::from([
                    ("path".to_string(), path),
                    ("content".to_string(), content),
                ])),
                Err(_) => None,
            }
        })
        .collect();

    // Print the dictionary
    print_dict(&d_list);

    // Write the dictionary to a jsonlines file
    let serialized = d_list
        .iter()
        .map(|d| serde_json::to_string(d).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write(output_path, serialized).expect("Unable to write file");
}

fn print_dict(d_list: &Vec<HashMap<String, String>>) {
    d_list.iter().for_each(|d| {
        // NOTE expect is safe here because we know the keys are in the dict
        let body = d.get("content").expect("Missing content in dict");
        let body = if body.len() > 10 { &body[0..10] } else { &body };
        let body = body.replace("\n", r#"  \n  "#);

        let path = d.get("path").expect("Missing path in dict");
        let path = path.replace("/home/ryan/Notes/slipbox/", "");

        println!("{path:<60}:\t{body}")
    });
}

// Constant for the fields
const FIELDS: [&str; 2] = ["path", "content"];
