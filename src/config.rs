use std::env;

// A struct for meta info including things like note taking directory, vim path, vscode path, default editor etc.
// This struct will be used to store the meta info and will be passed around to various functions
pub struct Config {
    pub note_taking_dir: String,
    pub vim_path: String,
    pub vscode_path: String,
    pub default_editor: String,
}

impl Config {
    pub fn default() -> Config {
        let home = env::var("HOME").expect("HOME not set");
        Config {
            note_taking_dir: format!("{home}"),
            vim_path: "/usr/bin/nvim".to_string(),
            vscode_path: "/usr/bin/codium".to_string(),
            default_editor: "vim".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn new(
        note_taking_dir: String,
        vim_path: String,
        vscode_path: String,
        default_editor: String,
    ) -> Config {
        Config {
            note_taking_dir,
            vim_path,
            vscode_path,
            default_editor,
        }
    }
}

// Test that the note_taking_dir is set correctly
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn note_taking_directory_exists() {
        let config = Config::default();

        let read_dir = std::fs::read_dir(&config.note_taking_dir);
        assert!(
            read_dir.is_ok(),
            "Note taking directory does not exist or cannot be read: {}",
            config.note_taking_dir
        );

        // Optionally, if you also want to ensure that the directory is not empty,
        // you might want to add such a check (but note that this may fail if the directory is indeed empty by design):
        let entries = read_dir.unwrap();
        assert!(
            entries.count() > 0,
            "Note taking directory is empty: {}",
            config.note_taking_dir
        );
    }
    #[test]
    fn config_default_editor_paths_exist() {
        let config = Config::default();

        // It's critical to test each path in isolation to know exactly which one fails, so split this into two assertions instead of looping.
        assert!(
            std::fs::File::open(&config.vim_path).is_ok(),
            "Vim path does not exist: {}",
            config.vim_path
        );

        assert!(
            std::fs::File::open(&config.vscode_path).is_ok(),
            "VSCode path does not exist: {}",
            config.vscode_path
        );
    }

    // Check the directory exists by trying to list the files in it

    // let dir = std::fs::read_dir(&config.note_taking_dir);
    // assert!(dir.is_ok());
}
