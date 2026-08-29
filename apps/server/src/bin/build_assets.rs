use anyhow::{anyhow, Context};
use std::fs;
use std::path::{Path, PathBuf};

const ASSETS_DIR: &str = "assets";
const DIST_DIR: &str = "dist";
const SKIP_EXTENSIONS: [&str; 1] = ["map"];

fn process_and_copy_asset(asset_path: &PathBuf, to_hash: bool) -> anyhow::Result<()> {
    // take path and open file buffer
    let base_path = asset_path.with_extension("");
    let base_path = base_path.to_str().context("Invalid UTF-8 in filename")?;
    let ext = asset_path
        .extension()
        .context(format!(
            "Asset {} path has no extension",
            asset_path.display()
        ))?
        .to_str()
        .context("Invalid UTF-8 chars in extension")?;
    let data =
        fs::read(asset_path).context(format!("Failed reading file at {}", asset_path.display()))?;
    // take hash of the content of the file using blake3
    let dest_relative_path: String = if to_hash {
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
    let dest_full_path = PathBuf::from(DIST_DIR).join(&dest_relative_path);
    println!(
        "Building asset: asset_path={}, dest_full_path={}",
        asset_path.display(),
        dest_full_path.display()
    );

    let dest_dir = dest_full_path
        .parent()
        .context("Destination path has no parent directory")?;

    fs::create_dir_all(dest_dir).with_context(|| {
        format!(
            "Failed creating destination directory {}",
            dest_dir.display()
        )
    })?;

    fs::write(&dest_full_path, &data)
        .with_context(|| format!("Failed writing asset to {}", dest_full_path.display()))?;

    Ok(())
}

fn should_skip(path: &Path) -> anyhow::Result<bool> {
    path.extension().map_or_else(
        // None option will be for extensionless files like MacOS .DS_Store
        || Ok(true),
        |ext| {
            if SKIP_EXTENSIONS.contains(&ext.to_str().context("Invalid UTF-8 in extension")?) {
                Ok(true)
            } else {
                Ok(false)
            }
        },
    )
}

fn is_vendor_asset(asset_path: &Path) -> anyhow::Result<bool> {
    let vendor_path = Path::new(ASSETS_DIR).join("vendor");
    Ok(asset_path.starts_with(
        vendor_path
            .to_str()
            .context(format!("Invalid vendor path: {}", vendor_path.display()))?,
    ))
}

fn build_assets(path: &PathBuf) -> anyhow::Result<()> {
    if path.is_file() {
        if should_skip(path)? {
            return Ok(());
        }
        let to_hash =
            !is_vendor_asset(path).context(format!("Invalid path: {}", path.display()))?;
        process_and_copy_asset(path, to_hash)?;
    }
    for entry in fs::read_dir(path)? {
        let asset_path = entry?.path();
        if asset_path.is_dir() {
            build_assets(&asset_path)?;
            continue;
        }
        if should_skip(&asset_path)? {
            continue;
        }
        let to_hash = !is_vendor_asset(&asset_path)
            .context(format!("Invalid asset_path: {}", asset_path.display()))?;
        process_and_copy_asset(&asset_path, to_hash)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let assets_path = PathBuf::from(ASSETS_DIR);
    if !assets_path.exists() {
        println!(
            "No assets dir '{}' in current directory found.",
            assets_path.display()
        );
        return Err(anyhow!(format!("No assets dir '{}' in current directory found.",
            assets_path.display())))
    }
    if !assets_path.is_dir() {
        return Err(anyhow!(format!("Assets path '{}' is a file and not a dir",
            assets_path.display())))
    }

    let dist_dir = assets_path
        .parent()
        .context(format!(
            "Invalid assets parent dir assets_path={}",
            assets_path.display()
        ))?
        .join(DIST_DIR);
    if dist_dir.exists() {
        if !dist_dir.is_dir() {
            return Err(anyhow!(format!("dist_dir={} exists and is a file", dist_dir.display())));
        }
        fs::remove_dir_all(&dist_dir)?;
        println!("Cleaned existing {}", dist_dir.display());
    }
    fs::create_dir_all(dist_dir)?;
    build_assets(&PathBuf::from(ASSETS_DIR))?;
    Ok(())
}
