use duct::cmd;
use std::path::PathBuf;

pub fn fzf_choose(dir: &str) -> PathBuf {
    // unfortunately grep returns 1 on no files found
    // so we'll have to treat failure as no files found
    // fd -t f  '\.org$|\.md$' | sk --preview 'bat {}'

    let regex = r#"\.org$|\.md$"#;
    match cmd!("fd", "-t", "f", regex, dir)
        .pipe(cmd!(
            "sk",
            "--height=80%",
            "--preview",
            "bat --color=always {}"
        ))
        .read()
    {
        Ok(stdout) => stdout.lines().map(|x| x.to_string()).collect(),
        Err(_) => {
            panic!("Non Zero Exit using fd | sk on {:?}", dir);
        }
    }
}


// TODO directory should be var
pub fn fzf_search() {
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
    let mut skim_command: String = format!("tantivy search -i {cache_dir} --query ").into();
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
