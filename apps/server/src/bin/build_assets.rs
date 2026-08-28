use std::fs;
use std::path::PathBuf;
use anyhow::Context;

const SKIP_EXTENSIONS: [&str; 1] = ["map"];

fn should_skip(path: &PathBuf) -> anyhow::Result<bool> {
    if let Some(ext) = path.extension()
        && SKIP_EXTENSIONS.contains(&ext.to_str().context("Invalid UTF-8 in extension")?) {
            return Ok(true)
        }
    return Ok(path.ends_with(".DS_Store"))
}

fn walk_files(dir: &PathBuf) -> anyhow::Result<()> {
    if dir.is_file() {
        if should_skip(dir)? {
            return Ok(())
        }
        println!("{dir:?}");
        return Ok(())
    }
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.is_file() {            
            if should_skip(&path)? {
                continue;
            }
            println!("{path:?}");
            continue;
        }
        walk_files(&path)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()>{
    walk_files(&PathBuf::from("assets"))?;
    Ok(())
}
