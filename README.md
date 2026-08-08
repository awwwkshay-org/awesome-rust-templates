# Awesome Rust Templates

Reusable, production-oriented Rust application starters.

## Templates

| Template | Stack |
| --- | --- |
| [`fullstack-mono`](templates/fullstack-mono) | Axum API, Dioxus SSR/web/mobile UI, shared Rust types, PostgreSQL, Docker, and GitHub CI/CD |

Each directory under `templates/` is a complete standalone repository. It owns
its application README, documentation, lockfile, Docker configuration, and
GitHub workflows.

## Create a project

Install the latest tagged version directly from GitHub:

```sh
cargo install --git https://github.com/awwwkshay-org/awesome-rust-templates \
  --tag v0.3.0 --locked --package awesome-rust-templates
```

This installs the `art` binary in Cargo's binary directory. Tagged GitHub
releases also provide prebuilt archives for Apple Silicon macOS, Intel macOS,
and x64/ARM64 Linux. Because `art` generates a fresh project lockfile, Cargo
must still be installed when running the downloaded binary.

You can also run the generator directly from this checkout:

```sh
cargo run -p awesome-rust-templates --bin art -- . --template fullstack-mono
```

Or install it locally:

```sh
cargo install --path tools/awesome-rust-templates --locked
art . --template fullstack-mono
```

This initializes the full-stack template directly in the current directory. To
create a named child directory instead, run:

```sh
art --template fullstack-mono --name example-fullstack-mono
```

For an interactive setup, omit the remaining options:

```sh
art
```

The CLI will show the available templates and ask for the project name and Git
initialization preference.

For a quick setup using all defaults, pass `--yes` (or `-y`):

```sh
art my-project --yes
```

When `DIRECTORY` is supplied, its final path component is also used as the
project name. Pass `--name` when you want to create a named child directory or
override that inferred name.

The generator downloads the template asset matching its own version from the
project's GitHub release, verifies its SHA-256 checksum, and caches it in the
operating system's standard cache directory. The first use requires network
access; later projects using the same version work from the cache. After
customizing the Cargo manifests, it runs `cargo generate-lockfile` so
`Cargo.lock` always matches the generated package names and dependency
declarations. By default it initializes a Git repository on `main`, stages the
generated project, and creates an `Initial commit` using your configured Git
identity. Pass a directory as the positional argument, and use `--no-git` to
skip Git initialization and the commit. The former `--init [DIRECTORY]` form
remains available for compatibility. Existing files are never overwritten.

## Validate

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The root CI additionally validates the Rust workspace, PostgreSQL integration
tests, Dioxus server/client targets, and both containers inside every maintained
template.
