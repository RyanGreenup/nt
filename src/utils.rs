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
