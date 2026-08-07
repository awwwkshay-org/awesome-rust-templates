use std::{
    env,
    fs::File,
    path::{Component, Path},
};

use flate2::{Compression, write::GzEncoder};
use tar::Builder;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or("generator must live at tools/awesome-rust-templates")?;
    let template = repository.join("templates/fullstack-mono");
    let archive_path = Path::new(&env::var("OUT_DIR")?).join("fullstack-mono.tar.gz");
    let encoder = GzEncoder::new(File::create(archive_path)?, Compression::best());
    let mut archive = Builder::new(encoder);

    for entry in WalkDir::new(&template).follow_links(false) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(&template)?;
        if relative.as_os_str().is_empty() || excluded(relative) {
            continue;
        }
        if entry.file_type().is_file() {
            archive.append_path_with_name(entry.path(), relative)?;
        } else if entry.file_type().is_dir() {
            archive.append_dir(relative, entry.path())?;
        }
    }

    archive.into_inner()?.finish()?;
    println!("cargo:rerun-if-changed={}", template.display());
    Ok(())
}

fn excluded(path: &Path) -> bool {
    if path == Path::new("Cargo.lock") {
        return true;
    }
    if matches!(
        path.components().next(),
        Some(Component::Normal(value)) if value == ".git" || value == "target"
    ) {
        return true;
    }
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".DS_Store")
    ) || matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(value) if value == ".env" || (value.starts_with(".env.") && value != ".env.example")
    )
}
