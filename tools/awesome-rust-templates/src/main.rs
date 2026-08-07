use std::{
    fs,
    io::{BufRead, Cursor, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};

const RELEASE_REPOSITORY: &str = "awwwkshay-org/awesome-rust-templates";

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Template {
    FullstackMono,
}

impl Template {
    fn default_project_name(self) -> &'static str {
        match self {
            Self::FullstackMono => "fullstack-mono",
        }
    }

    fn asset_name(self) -> &'static str {
        match self {
            Self::FullstackMono => "fullstack-mono",
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "art",
    version,
    about = "Create projects from Awesome Rust Templates"
)]
struct Args {
    /// Initialize a project in DIRECTORY (defaults to the current directory).
    #[arg(
        long,
        value_name = "DIRECTORY",
        num_args = 0..=1,
        default_missing_value = ".",
        required = true
    )]
    init: PathBuf,

    /// Template to use.
    #[arg(long, value_enum)]
    template: Option<Template>,

    /// Create the project in a new child directory with this name.
    #[arg(long)]
    name: Option<String>,

    /// Do not initialize a Git repository.
    #[arg(long)]
    no_git: bool,
}

fn main() -> Result<()> {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let args = resolve_args(Args::parse(), &mut stdin, &mut stdout)?;
    create_from_args(&args)
}

fn create_from_args(args: &Args) -> Result<()> {
    let template = args
        .template
        .context("a template is required; pass --template or run art --init interactively")?;
    let archive = load_template(template)?;
    create_from_args_with_archive(args, &archive, true)
}

fn create_from_args_with_archive(
    args: &Args,
    archive: &[u8],
    should_generate_lockfile: bool,
) -> Result<()> {
    let template = args
        .template
        .context("a template is required; pass --template or run art --init interactively")?;
    let project_name = args
        .name
        .as_deref()
        .unwrap_or_else(|| template.default_project_name());
    validate_name(project_name)?;

    let destination = args
        .name
        .as_ref()
        .map_or_else(|| args.init.clone(), |name| args.init.join(name));
    let destination_existed = destination.exists();

    if args.name.is_some() && destination_existed {
        bail!("destination already exists: {}", destination.display());
    }
    if destination_existed && !destination.is_dir() {
        bail!("destination is not a directory: {}", destination.display());
    }

    fs::create_dir_all(&destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    if let Err(error) = scaffold(
        project_name,
        &destination,
        archive,
        !args.no_git,
        should_generate_lockfile,
    ) {
        if !destination_existed {
            let _ = fs::remove_dir_all(&destination);
        }
        return Err(error);
    }

    println!("Created {project_name} at {}", destination.display());
    println!(
        "Next: cd {} && docker compose up --build",
        destination.display()
    );
    Ok(())
}

fn load_template(template: Template) -> Result<Vec<u8>> {
    let version = env!("CARGO_PKG_VERSION");
    let asset = format!("{}-v{version}.tar.gz", template.asset_name());
    let cache = dirs::cache_dir()
        .context("could not determine the operating system cache directory")?
        .join("art")
        .join(version);
    let archive_path = cache.join(&asset);
    let checksum_path = cache.join(format!("{asset}.sha256"));

    if archive_path.is_file() && checksum_path.is_file() {
        let archive = fs::read(&archive_path)
            .with_context(|| format!("failed to read {}", archive_path.display()))?;
        let checksum = fs::read_to_string(&checksum_path)
            .with_context(|| format!("failed to read {}", checksum_path.display()))?;
        if verify_checksum(&archive, &checksum).is_ok() {
            return Ok(archive);
        }
    }

    fs::create_dir_all(&cache)
        .with_context(|| format!("failed to create template cache at {}", cache.display()))?;
    let release = format!("https://github.com/{RELEASE_REPOSITORY}/releases/download/v{version}");
    println!("Downloading template {asset}...");
    let checksum = download(&format!("{release}/{asset}.sha256"))?;
    let checksum = String::from_utf8(checksum).context("template checksum is not valid UTF-8")?;
    let archive = download(&format!("{release}/{asset}"))?;
    verify_checksum(&archive, &checksum)?;

    let archive_download = cache.join(format!("{asset}.download"));
    let checksum_download = cache.join(format!("{asset}.sha256.download"));
    fs::write(&archive_download, &archive)
        .with_context(|| format!("failed to write {}", archive_download.display()))?;
    fs::write(&checksum_download, &checksum)
        .with_context(|| format!("failed to write {}", checksum_download.display()))?;
    fs::rename(&archive_download, &archive_path)
        .with_context(|| format!("failed to cache {}", archive_path.display()))?;
    fs::rename(&checksum_download, &checksum_path)
        .with_context(|| format!("failed to cache {}", checksum_path.display()))?;
    Ok(archive)
}

fn download(url: &str) -> Result<Vec<u8>> {
    let response = ureq::get(url)
        .call()
        .with_context(|| format!("failed to download {url}"))?;
    response
        .into_body()
        .read_to_vec()
        .with_context(|| format!("failed to read {url}"))
}

fn verify_checksum(archive: &[u8], checksum_file: &str) -> Result<()> {
    let expected = checksum_file
        .split_whitespace()
        .next()
        .context("template checksum file is empty")?;
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("template checksum file contains an invalid SHA-256 digest");
    }
    let actual = sha256_hex(archive);
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("downloaded template failed SHA-256 verification");
    }
    Ok(())
}

fn sha256_hex(contents: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(contents);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn resolve_args<R: BufRead, W: Write>(
    mut args: Args,
    input: &mut R,
    output: &mut W,
) -> Result<Args> {
    if args.template.is_some() {
        return Ok(args);
    }

    writeln!(output, "Choose a template:")?;
    writeln!(output, "  1) fullstack-mono")?;
    loop {
        let selection = prompt(input, output, "Template [1]: ")?;
        match selection.as_str() {
            "" | "1" | "fullstack-mono" => {
                args.template = Some(Template::FullstackMono);
                break;
            }
            _ => writeln!(output, "Please enter 1 or fullstack-mono.")?,
        }
    }

    loop {
        let name = prompt(
            input,
            output,
            "Project name (leave blank to initialize the target directory): ",
        )?;
        if name.is_empty() {
            break;
        }
        match validate_name(&name) {
            Ok(()) => {
                args.name = Some(name);
                break;
            }
            Err(error) => writeln!(output, "{error}")?,
        }
    }

    if !args.no_git {
        loop {
            let initialize_git = prompt(input, output, "Initialize a Git repository? [Y/n]: ")?;
            match initialize_git.to_ascii_lowercase().as_str() {
                "" | "y" | "yes" => break,
                "n" | "no" => {
                    args.no_git = true;
                    break;
                }
                _ => writeln!(output, "Please enter y or n.")?,
            }
        }
    }

    Ok(args)
}

fn prompt<R: BufRead, W: Write>(input: &mut R, output: &mut W, message: &str) -> Result<String> {
    write!(output, "{message}")?;
    output.flush()?;
    let mut answer = String::new();
    input.read_line(&mut answer)?;
    Ok(answer.trim().to_owned())
}

fn scaffold(
    name: &str,
    destination: &Path,
    template: &[u8],
    initialize_git: bool,
    should_generate_lockfile: bool,
) -> Result<()> {
    ensure_no_conflicts(name, destination, template)?;

    let decoder = GzDecoder::new(Cursor::new(template));
    tar::Archive::new(decoder)
        .unpack(destination)
        .context("failed to unpack the downloaded template")?;
    customize(name, destination)?;
    if should_generate_lockfile {
        generate_lockfile(destination)?;
    }
    if initialize_git {
        init_git(destination)?;
    }
    Ok(())
}

fn ensure_no_conflicts(name: &str, destination: &Path, template: &[u8]) -> Result<()> {
    for app in [format!("{name}-api"), format!("{name}-ui")] {
        let target = destination.join("apps").join(app);
        if target.exists() {
            bail!(
                "refusing to overwrite existing directory: {}",
                target.display()
            );
        }
    }

    let decoder = GzDecoder::new(Cursor::new(template));
    for entry in tar::Archive::new(decoder)
        .entries()
        .context("failed to inspect the downloaded template")?
    {
        let entry = entry.context("failed to inspect a downloaded template entry")?;
        if entry.header().entry_type().is_file() {
            let path = entry
                .path()
                .context("downloaded template contains an invalid path")?;
            let target = destination.join(path.as_ref());
            if target.exists() {
                bail!("refusing to overwrite existing file: {}", target.display());
            }
        }
    }
    Ok(())
}

fn customize(name: &str, root: &Path) -> Result<()> {
    let title = human_title(name);
    let database = name.replace('-', "_");
    let api_name = format!("{name}-api");
    let ui_name = format!("{name}-ui");
    let shared_name = format!("{name}-shared");
    for (path, from, to) in [
        (
            "README.md",
            "# Rust full-stack template",
            format!("# {title}"),
        ),
        (
            "apps/ui/Dioxus.toml",
            "fullstack-mono-template",
            ui_name.clone(),
        ),
        ("apps/ui/Dioxus.toml", "Todo Template", title.clone()),
        ("apps/ui/src/main.rs", "Todo Template", title),
        ("docker-compose.yml", "app_test", format!("{database}_test")),
        ("docker-compose.yml", "app", database.clone()),
        ("apps/api/.env.example", "/app", format!("/{database}")),
        ("docs/deployment.md", "template-api", format!("{name}-api")),
        ("docs/deployment.md", "template-ui", format!("{name}-ui")),
    ] {
        replace(&root.join(path), from, &to)?;
    }

    for (from, to) in [
        ("apps/api".to_owned(), format!("apps/{api_name}")),
        ("apps/ui".to_owned(), format!("apps/{ui_name}")),
        (
            "name = \"api\"".to_owned(),
            format!("name = \"{api_name}\""),
        ),
        ("name = \"ui\"".to_owned(), format!("name = \"{ui_name}\"")),
        (
            "name = \"shared\"".to_owned(),
            format!("name = \"{shared_name}\""),
        ),
        (
            "shared = { path =".to_owned(),
            format!("shared = {{ package = \"{shared_name}\", path ="),
        ),
        ("-p api".to_owned(), format!("-p {api_name}")),
        ("-p ui".to_owned(), format!("-p {ui_name}")),
        ("--exclude ui".to_owned(), format!("--exclude {ui_name}")),
        (
            "target/release/api".to_owned(),
            format!("target/release/{api_name}"),
        ),
        (
            "/usr/local/bin/api".to_owned(),
            format!("/usr/local/bin/{api_name}"),
        ),
        (
            "ENTRYPOINT [\"api\"]".to_owned(),
            format!("ENTRYPOINT [\"{api_name}\"]"),
        ),
        ("target/dx/ui".to_owned(), format!("target/dx/{ui_name}")),
    ] {
        replace_in_text_files(root, &from, &to)?;
    }

    fs::rename(root.join("apps/api"), root.join("apps").join(&api_name))
        .with_context(|| format!("failed to rename the API app to {api_name}"))?;
    fs::rename(root.join("apps/ui"), root.join("apps").join(&ui_name))
        .with_context(|| format!("failed to rename the UI app to {ui_name}"))?;
    Ok(())
}

fn generate_lockfile(root: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(root)
        .output()
        .context("Cargo is unavailable; it is required to generate Cargo.lock")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cargo generate-lockfile failed: {}", stderr.trim());
    }
    Ok(())
}

fn replace_in_text_files(root: &Path, from: &str, to: &str) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            replace_in_text_files(&path, from, to)?;
        } else if let Ok(contents) = fs::read_to_string(&path)
            && contents.contains(from)
        {
            fs::write(&path, contents.replace(from, to))
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
    }
    Ok(())
}

fn replace(path: &Path, from: &str, to: &str) -> Result<()> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    fs::write(path, contents.replace(from, to))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn init_git(root: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(root)
        .status()
        .context("Git is unavailable; use --no-git to skip initialization")?;
    if !status.success() {
        bail!("git init failed; use --no-git to skip initialization");
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && name
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        && !name.contains("--");
    if !valid {
        bail!("project name must use lowercase letters, numbers, and single hyphens");
    }
    Ok(())
}

fn human_title(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().chain(characters).collect())
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::Component, sync::OnceLock};

    use flate2::{Compression, write::GzEncoder};
    use walkdir::WalkDir;

    fn args(init: PathBuf, name: Option<&str>) -> Args {
        Args {
            init,
            template: Some(Template::FullstackMono),
            name: name.map(str::to_owned),
            no_git: true,
        }
    }

    fn create_without_lockfile(args: &Args) -> Result<()> {
        create_from_args_with_archive(args, test_template(), false)
    }

    fn test_template() -> &'static [u8] {
        static ARCHIVE: OnceLock<Vec<u8>> = OnceLock::new();
        ARCHIVE.get_or_init(|| {
            let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .unwrap();
            let template = repository.join("templates/fullstack-mono");
            let encoder = GzEncoder::new(Vec::new(), Compression::fast());
            let mut archive = tar::Builder::new(encoder);
            for entry in WalkDir::new(&template).follow_links(false) {
                let entry = entry.unwrap();
                let relative = entry.path().strip_prefix(&template).unwrap();
                if relative.as_os_str().is_empty() || excluded_from_test_template(relative) {
                    continue;
                }
                if entry.file_type().is_file() {
                    archive
                        .append_path_with_name(entry.path(), relative)
                        .unwrap();
                } else if entry.file_type().is_dir() {
                    archive.append_dir(relative, entry.path()).unwrap();
                }
            }
            archive.into_inner().unwrap().finish().unwrap()
        })
    }

    fn excluded_from_test_template(path: &Path) -> bool {
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

    #[test]
    fn parses_both_supported_command_shapes() {
        let current =
            Args::try_parse_from(["art", "--init", ".", "--template", "fullstack-mono"]).unwrap();
        assert_eq!(current.init, PathBuf::from("."));
        assert!(matches!(current.template, Some(Template::FullstackMono)));
        assert!(current.name.is_none());

        let named = Args::try_parse_from([
            "art",
            "--init",
            "--template",
            "fullstack-mono",
            "--name",
            "example-fullstack-mono",
        ])
        .unwrap();
        assert_eq!(named.init, PathBuf::from("."));
        assert_eq!(named.name.as_deref(), Some("example-fullstack-mono"));
    }

    #[test]
    fn prompts_for_missing_options() {
        let parsed = Args::try_parse_from(["art", "--init"]).unwrap();
        let mut answers = Cursor::new(b"1\nfinsnap\nn\n");
        let mut output = Vec::new();

        let resolved = resolve_args(parsed, &mut answers, &mut output).unwrap();

        assert!(matches!(resolved.template, Some(Template::FullstackMono)));
        assert_eq!(resolved.name.as_deref(), Some("finsnap"));
        assert!(resolved.no_git);
        let transcript = String::from_utf8(output).unwrap();
        assert!(transcript.contains("Choose a template"));
        assert!(transcript.contains("Project name"));
        assert!(transcript.contains("Initialize a Git repository"));
    }

    #[test]
    fn validates_names() {
        assert!(validate_name("my-app-2").is_ok());
        assert!(validate_name("My App").is_err());
        assert!(validate_name("my--app").is_err());
    }

    #[test]
    fn verifies_template_checksums() {
        let digest = sha256_hex(b"template");
        assert!(verify_checksum(b"template", &format!("{digest}  template.tar.gz")).is_ok());
        assert!(verify_checksum(b"changed", &digest).is_err());
        assert!(verify_checksum(b"template", "not-a-digest").is_err());
    }

    #[test]
    fn initializes_the_target_directory() {
        let temporary = tempfile::tempdir().unwrap();
        create_without_lockfile(&args(temporary.path().to_owned(), None)).unwrap();

        assert!(
            temporary
                .path()
                .join("apps/fullstack-mono-api/src/main.rs")
                .is_file()
        );
        assert!(temporary.path().join(".github/workflows/ci.yml").is_file());
        let dioxus =
            fs::read_to_string(temporary.path().join("apps/fullstack-mono-ui/Dioxus.toml"))
                .unwrap();
        assert!(dioxus.contains("name = \"fullstack-mono-ui\""));
    }

    #[test]
    fn creates_a_named_child_directory() {
        let temporary = tempfile::tempdir().unwrap();
        create_without_lockfile(&args(temporary.path().to_owned(), Some("invoice-box"))).unwrap();
        let project = temporary.path().join("invoice-box");
        assert!(project.join("apps/invoice-box-api/src/main.rs").is_file());
        assert!(project.join("apps/invoice-box-ui/src/main.rs").is_file());
        assert!(!project.join("apps/api").exists());
        assert!(!project.join("apps/ui").exists());
        assert!(project.join(".github/workflows/ci.yml").is_file());
        assert!(!project.join("target").exists());
        assert!(!project.join("tools").exists());

        let dioxus = fs::read_to_string(project.join("apps/invoice-box-ui/Dioxus.toml")).unwrap();
        assert!(dioxus.contains("name = \"invoice-box-ui\""));
        assert!(dioxus.contains("title = \"Invoice Box\""));

        let workspace = fs::read_to_string(project.join("Cargo.toml")).unwrap();
        assert!(workspace.contains("apps/invoice-box-api"));
        assert!(workspace.contains("apps/invoice-box-ui"));
        let api_manifest =
            fs::read_to_string(project.join("apps/invoice-box-api/Cargo.toml")).unwrap();
        assert!(api_manifest.contains("name = \"invoice-box-api\""));
        assert!(api_manifest.contains("package = \"invoice-box-shared\""));
        let ui_manifest =
            fs::read_to_string(project.join("apps/invoice-box-ui/Cargo.toml")).unwrap();
        assert!(ui_manifest.contains("name = \"invoice-box-ui\""));
        let shared_manifest =
            fs::read_to_string(project.join("packages/shared/Cargo.toml")).unwrap();
        assert!(shared_manifest.contains("name = \"invoice-box-shared\""));
        assert!(!project.join("Cargo.lock").exists());
    }

    #[test]
    fn refuses_to_overwrite_files_in_the_target_directory() {
        let temporary = tempfile::tempdir().unwrap();
        fs::write(temporary.path().join("README.md"), "keep me").unwrap();

        let error = create_without_lockfile(&args(temporary.path().to_owned(), None)).unwrap_err();
        assert!(error.to_string().contains("refusing to overwrite"));
        assert_eq!(
            fs::read_to_string(temporary.path().join("README.md")).unwrap(),
            "keep me"
        );
    }
}
