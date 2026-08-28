use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

const SKIP_EXTENSIONS: [&str; 1] = ["map"];

fn process_and_copy_asset(path: &PathBuf, to_hash: bool) -> anyhow::Result<()> {
    let base_path = path.with_extension("");
    let base_path = base_path.to_str().context("Invalid UTF-8 in filename")?;
    let ext = path
        .extension()
        .context("Asset path has no extension")?
        .to_str()
        .context("Invalid UTF-8 chars in extension")?;
    // take path and open file buffer
    let data = fs::read(path).context(format!("Failed reading file at {}", path.display()))?;
    // take hash of the content of the file using blake3
    let source_file_path: String = if to_hash {
        let hex_str = blake3::hash(&data).to_hex();
        let short_hash = hex_str
            .as_str()
            .get(..12)
            .context("BLAKE3 digest unexpectedly shorter than 12 charachters")?;
        format!("{base_path}.{short_hash}.{ext}")
    } else {
        format!("{base_path}.{ext}")
    };
    // In the outdir dump file's data that we read earlier with same hierarchy but with hash in its name
    let dest_path = PathBuf::from("dist").join(&source_file_path);
    println!("path={}, dest_path={}", path.display(), dest_path.display());

    let dest_dir = dest_path
        .parent()
        .context("Destination path has no parent directory")?;

    fs::create_dir_all(dest_dir).with_context(|| {
        format!(
            "Failed creating destination directory {}",
            dest_dir.display()
        )
    })?;

    fs::write(&dest_path, &data)
        .with_context(|| format!("Failed writing asset to {}", dest_path.display()))?;

    Ok(())
}

fn should_skip(path: &Path) -> anyhow::Result<bool> {
    if let Some(ext) = path.extension()
        && SKIP_EXTENSIONS.contains(&ext.to_str().context("Invalid UTF-8 in extension")?)
    {
        return Ok(true);
    }
    Ok(path.ends_with(".DS_Store"))
}

fn build_assets(dir: &PathBuf) -> anyhow::Result<()> {
    if dir.is_file() {
        if should_skip(dir)? {
            return Ok(());
        }
        let to_hash = !dir
            .to_str()
            .context("Error trying to read pathname")?
            .contains("vendor");
        process_and_copy_asset(dir, to_hash)?;
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() {
            if should_skip(&path)? {
                continue;
            }
            let to_hash = !path
                .to_str()
                .context("Error trying to read pathname")?
                .contains("vendor");
            process_and_copy_asset(&path, to_hash)?;
            continue;
        }
        build_assets(&path)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let dist_dir = Path::new("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir)?;
        println!("Cleaned existing dist/ dir");
    }
    fs::create_dir_all(dist_dir)?;
    build_assets(&PathBuf::from("assets"))?;
    Ok(())
}
