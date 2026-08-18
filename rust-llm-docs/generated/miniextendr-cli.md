# miniextendr v0.1.0

`miniextendr` — maintainer CLI for miniextendr-based R packages.

Wraps the configure/install/document/vendor development loop in one
binary (`miniextendr <command>`). This is a binary-only crate; the
framework runtime lives in `miniextendr-api` and the scaffolding for
end-user packages in the `minirextendr` R package.

---

## Modules

### `commands::cargo`

`pub mod cargo;`

### `commands::config`

`pub mod config;`

### `commands::feature`

`pub mod feature;`

### `commands::init`

`pub mod init;`

`miniextendr init` — scaffold packages from the canonical minirextendr
templates.

Every file comes from the embedded copy of `minirextendr/inst/templates/`
(see [`crate::scaffold`]), so a CLI-generated package uses exactly the
same build system as `minirextendr::create_miniextendr_package()` /
`create_miniextendr_monorepo()` / `use_miniextendr()`: the two-mode
install latch keyed on `inst/vendor.tar.xz` presence, configure-written
`.cargo/config.toml`, wrapper + wasm-registry generation, and no `just`
requirement (#1351).

### `commands::lint`

`pub mod lint;`

### `commands::render`

`pub mod render;`

### `commands::rust`

`pub mod rust;`

### `commands::status`

`pub mod status;`

### `commands::vendor`

`pub mod vendor;`

### `commands::workflow`

`pub mod workflow;`

---

## Structs

### `cli::CargoBuildOpts`

```rust
pub struct CargoBuildOpts
```

Shared build options for cargo commands.

**Fields:**

- `release`: `bool`
  - Build in release mode.
- `features`: `Option<String>`
  - Comma-separated list of features to enable.
- `no_default_features`: `bool`
  - Disable default features.
- `all_features`: `bool`
  - Enable all features.
- `target`: `Option<String>`
  - Build target triple.
- `offline`: `bool`
  - Enable offline mode.

### `cli::Cli`

```rust
pub struct Cli
```

**Fields:**

- `path`: `String`
  - Project directory (default: current directory).
- `quiet`: `bool`
  - Suppress output.
- `json`: `bool`
  - Output in JSON format.
- `command`: `Command`

### `project::ProjectContext`

```rust
pub struct ProjectContext
```

Discovered project paths.

**Fields:**

- `root`: `std::path::PathBuf`
  - The project root (where DESCRIPTION or Cargo.toml lives).
- `cargo_manifest`: `Option<std::path::PathBuf>`
  - `src/rust/Cargo.toml` if this is an R package with Rust.
- `description`: `Option<std::path::PathBuf>`
  - `DESCRIPTION` file if this is an R package.
- `configure_ac`: `Option<std::path::PathBuf>`
  - `configure.ac` if autoconf is set up.
- `configure`: `Option<std::path::PathBuf>`
  - `configure` script.

**Inherent associated items:**

#### `description_field`

```rust
fn description_field(self: &Self, field: &str) -> Option<String>
```

Read a field's value from `DESCRIPTION`, if present.

DCF (Debian Control File, the format `DESCRIPTION` uses) allows a
field's value to continue onto following lines: a continuation line
starts with whitespace and is joined onto the value of the field it
extends, separated by a single space.

#### `discover`

```rust
fn discover(path: &Path) -> Result<Self>
```

Discover project structure starting from `path`.

#### `has_miniextendr`

```rust
fn has_miniextendr(self: &Self) -> bool
```

Check if this looks like a miniextendr project.

#### `package_name`

```rust
fn package_name(self: &Self) -> Option<String>
```

The `Package` field from `DESCRIPTION`, if present.

#### `require_cargo_manifest`

```rust
fn require_cargo_manifest(self: &Self) -> Result<&Path>
```

Returns the cargo manifest path, or an error with guidance.

#### `require_configure_ac`

```rust
fn require_configure_ac(self: &Self) -> Result<&Path>
```

Returns the configure.ac path, or an error with guidance.

### `scaffold::PlanEntry`

```rust
pub struct PlanEntry
```

One template → destination mapping.

**Fields:**

- `template`: `&'static str`
  - Template path relative to the template-type directory (the `prefix`
- `dest`: `Dest`
- `render`: `Render`
- `exec`: `bool`
  - Whether the destination gets mode 755 (R build hooks must be

### `scaffold::TemplateData`

```rust
pub struct TemplateData
```

Substitution variables for template rendering. Mirrors `template_data()`
in `minirextendr/R/utils.R`: `package`, `package_rs`, `Package`,
`features_var`, `year`, plus optional `crate_name`/`crate_name_rs` and
`rpkg_name` for the monorepo template.

**Fields:**

- `package`: `String`
- `crate_name`: `Option<String>`

**Inherent associated items:**

#### `new`

```rust
fn new(package: &str) -> Self
```

#### `pairs`

```rust
fn pairs(self: &Self) -> &[(&'static str, String)]
```

#### `with_crate`

```rust
fn with_crate(self: Self, crate_name: &str) -> Self
```

#### `with_rpkg`

```rust
fn with_rpkg(self: Self, rpkg_name: &str) -> Self
```

---

## Enums

### `cli::CargoCmd`

```rust
pub enum CargoCmd
```

**Variants:**

- `Init { name: Option<String>, edition: String }`
  - Initialize Rust crate in src/rust.
- `New { name: String, lib: bool, edition: String }`
  - Create a new Rust crate in the workspace.
- `Add { dep: String, features: Option<String>, no_default_features: bool, optional: bool, rename: Option<String>, crate_path: Option<String>, git: Option<String>, branch: Option<String>, tag: Option<String>, rev: Option<String>, dev: bool, build: bool, dry_run: bool }`
  - Add a Rust dependency.
- `Rm { dep: String, dev: bool, build: bool, dry_run: bool }`
  - Remove a Rust dependency.
- `Update { dep: Option<String>, precise: Option<String>, dry_run: bool }`
  - Update Cargo.lock.
- `Build { opts: CargoBuildOpts, jobs: Option<u32> }`
  - Run cargo build.
- `Check { opts: CargoBuildOpts }`
  - Run cargo check.
- `Test { opts: CargoBuildOpts, no_run: bool, test_args: Vec<String> }`
  - Run cargo test.
- `Clippy { opts: CargoBuildOpts, all_targets: bool }`
  - Run cargo clippy.
- `Fmt { check: bool }`
  - Run cargo fmt.
- `Doc { open: bool, no_deps: bool, opts: CargoBuildOpts }`
  - Build Rust documentation.
- `Search { query: String, limit: u32 }`
  - Search crates.io.
- `Deps { depth: u32, duplicates: bool, invert: Option<String> }`
  - Show dependency tree.
- `Clean`
  - Clean cargo build artifacts.

### `cli::Command`

```rust
pub enum Command
```

**Variants:**

- `Init { cmd: InitCmd }`
  - Create or add miniextendr to a project.
- `Workflow { cmd: WorkflowCmd }`
  - Build, document, check, and manage R package.
- `Status { cmd: StatusCmd }`
  - Check project status.
- `Cargo { cmd: CargoCmd }`
  - Run cargo commands in project context.
- `Vendor { cmd: VendorCmd }`
  - Manage vendored dependencies.
- `Feature { cmd: FeatureCmd }`
  - Manage Cargo features and detection rules.
- `Render { cmd: RenderCmd }`
  - Rmarkdown/Quarto integration.
- `Rust { cmd: RustCmd }`
  - Dynamic Rust compilation.
- `Config { cmd: ConfigCmd }`
  - Show configuration.
- `Lint`
  - Run miniextendr-lint (checks macro/module consistency).
- `Clean`
  - Clean build artifacts.
- `Completions { shell: clap_complete::Shell }`
  - Generate shell completions.

### `cli::ConfigCmd`

```rust
pub enum ConfigCmd
```

**Variants:**

- `Show`
  - Show current miniextendr.yml config.
- `Defaults`
  - Show default config values.

### `cli::FeatureCmd`

```rust
pub enum FeatureCmd
```

**Variants:**

- `Enable { name: String }`
  - Enable a feature: r6, s4, s7, serde, vctrs, rayon, build-rs, knitr, rmarkdown, quarto, feature-detection.
- `List`
  - List Cargo features and optional dependencies.
- `Detect { cmd: FeatureDetectCmd }`
  - Configure-time feature detection.
- `Rule { cmd: FeatureRuleCmd }`
  - Feature detection rules.

### `cli::FeatureDetectCmd`

```rust
pub enum FeatureDetectCmd
```

**Variants:**

- `Init`
  - Set up configure-time feature detection infrastructure.
- `Update`
  - Update runtime feature detection after adding/removing features.

### `cli::FeatureRuleCmd`

```rust
pub enum FeatureRuleCmd
```

**Variants:**

- `Add { feature: String, detect: String, cargo_spec: Option<String>, optional_dep: bool }`
  - Add a feature detection rule.
- `Remove { feature: String }`
  - Remove a feature detection rule.
- `List`
  - List current feature detection rules.

### `cli::InitCmd`

```rust
pub enum InitCmd
```

**Variants:**

- `Package { dest: String }`
  - Create a new R package with miniextendr.
- `Monorepo { dest: String, package: Option<String>, crate_name: Option<String>, rpkg_name: Option<String>, local_path: Option<String>, miniextendr_version: String }`
  - Create a Rust workspace with embedded R package.
- `Use { template_type: String, rpkg_name: Option<String>, miniextendr_version: String, local_path: Option<String> }`
  - Add miniextendr scaffolding to an existing project.

### `cli::RenderCmd`

```rust
pub enum RenderCmd
```

**Variants:**

- `KnitrSetup`
  - Set up knitr integration.
- `Rmarkdown`
  - Set up rmarkdown integration.
- `Quarto`
  - Set up Quarto integration.
- `QuartoPre`
  - Run Quarto pre-render hook.
- `Html`
  - HTML document format with miniextendr sync.
- `Pdf`
  - PDF document format with miniextendr sync.
- `Word`
  - Word document format with miniextendr sync.

### `cli::RustCmd`

```rust
pub enum RustCmd
```

**Variants:**

- `Source { code: String }`
  - Source Rust code dynamically.
- `Function { code: String }`
  - Define a single Rust function.
- `Clean`
  - Clean compiled Rust code.

### `cli::StatusCmd`

```rust
pub enum StatusCmd
```

**Variants:**

- `Has`
  - Check if project has miniextendr setup (native, no R needed).
- `Show`
  - Show which miniextendr files are present/missing.
- `Validate`
  - Validate miniextendr configuration is ready to build.

### `cli::VendorCmd`

```rust
pub enum VendorCmd
```

**Variants:**

- `Pack`
  - Create vendor.tar.xz for CRAN submission.
- `Versions`
  - List available miniextendr versions.
- `Miniextendr { miniextendr_version: String, dest: Option<String>, refresh: bool, local_path: Option<String> }`
  - Download/copy miniextendr crates to vendor/.
- `CratesIo`
  - Vendor external crates.io dependencies.
- `Sync`
  - Sync vendored crates from local miniextendr source.
- `SyncCheck`
  - Verify vendored crates match workspace sources.
- `SyncDiff`
  - Show diff between workspace and vendor.
- `CacheInfo`
  - Show cache directory info and cached versions.
- `CacheClear { cache_version: Option<String> }`
  - Remove cached miniextendr archives.
- `UseLib { crate_name: String, dev_path: Option<String> }`
  - Vendor a local path dependency for CRAN submission.

### `cli::WorkflowCmd`

```rust
pub enum WorkflowCmd
```

**Variants:**

- `Autoconf`
  - Run autoconf to generate configure script from configure.ac.
- `Configure`
  - Run ./configure to generate Makevars and build config.
- `Document`
  - Generate R wrappers from Rust code (devtools::document).
- `Build { no_install: bool }`
  - Full two-pass build: autoconf, configure, install, document, install.
- `Install { r_cmd: bool, args: Vec<String> }`
  - Install R package (R CMD INSTALL or devtools::install).
- `Check { error_on: String, check_dir: Option<String>, args: Vec<String> }`
  - Run R CMD check or devtools::check.
- `Test { filter: Option<String> }`
  - Run R package tests (devtools::test).
- `Doctor`
  - Comprehensive project health check.
- `Upgrade`
  - Upgrade miniextendr package to latest conventions.
- `CheckRust`
  - Validate Rust toolchain is available.
- `Sync`
  - Sync project: autoconf + configure + document.
- `DevLink`
  - Link package for development (devtools::load_all).

### `scaffold::Dest`

```rust
pub enum Dest
```

Destination of a scaffolded file, relative to the scaffold root.

**Variants:**

- `Path(&'static str)`
  - Fixed relative path.
- `PackageDoc`
  - `R/<package>-package.R`.
- `CratePath(&'static str)`
  - `<crate_name>/<path>` (monorepo core crate).

### `scaffold::Render`

```rust
pub enum Render
```

How a template file's content is transformed before writing.

**Variants:**

- `Mustache`
  - `{{{key}}}` then `{{key}}` substitution; unresolved `{{identifier}}`
- `TripleOnly`
  - Literal `{{{key}}}` substitution only; `{{...}}` preserved.
- `Verbatim`
  - Byte-for-byte copy.
- `IgnoreFilter`
  - Ignore-file prep: drop blank lines and `#` comments.

---

## Functions

### `bridge::bash`

```rust
fn bash(script: &str, cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run a shell command via `bash -c`.

### `bridge::find_rscript`

```rust
fn find_rscript() -> anyhow::Result<std::path::PathBuf>
```

Locate the `Rscript` binary.

Search order:
1. `$R_HOME/bin/Rscript`
2. `Rscript` on `$PATH`

### `bridge::has_program`

```rust
fn has_program(name: &str) -> bool
```

Check if a program is available on PATH.

### `bridge::program_version`

```rust
fn program_version(name: &str) -> Option<String>
```

Get version output from a program.

### `bridge::rscript_eval`

```rust
fn rscript_eval(expr: &str, cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run `Rscript -e '<expr>'` in the given directory.

Forwards stdout/stderr directly for interactive feel.
Returns an error if the process exits non-zero.

### `bridge::run_command`

```rust
fn run_command(program: &str, args: &[impl AsRef<std::ffi::OsStr>], cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run an arbitrary command, forwarding stdio.

### `bridge::run_command_capture`

```rust
fn run_command_capture(program: &str, args: &[impl AsRef<std::ffi::OsStr>], cwd: &std::path::Path) -> anyhow::Result<String>
```

Run an arbitrary command and capture stdout.

### `commands::cargo::dispatch`

```rust
fn dispatch(cmd: &crate::cli::CargoCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::config::dispatch`

```rust
fn dispatch(cmd: &crate::cli::ConfigCmd, ctx: &crate::project::ProjectContext, _quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::dispatch`

```rust
fn dispatch(cmd: &crate::cli::Command, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::feature::dispatch`

```rust
fn dispatch(cmd: &crate::cli::FeatureCmd, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::init::dispatch`

```rust
fn dispatch(cmd: &crate::cli::InitCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::lint::run`

```rust
fn run(ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

Run miniextendr-lint via cargo check on the project's Rust crate.

The lint runs as a build script; cargo check triggers it.
Lint output appears as cargo warnings.

### `commands::render::dispatch`

```rust
fn dispatch(cmd: &crate::cli::RenderCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::rust::dispatch`

```rust
fn dispatch(cmd: &crate::cli::RustCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::status::dispatch`

```rust
fn dispatch(cmd: &crate::cli::StatusCmd, ctx: &crate::project::ProjectContext, _quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::vendor::dispatch`

```rust
fn dispatch(cmd: &crate::cli::VendorCmd, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::workflow::dispatch`

```rust
fn dispatch(cmd: &crate::cli::WorkflowCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `output::print_json`

```rust
fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()>
```

Serialize `value` as pretty JSON and print it to stdout.

### `output::print_status`

```rust
fn print_status(msg: &str)
```

Print a simple status message.

### `project::find_workspace_root`

```rust
fn find_workspace_root(start: &std::path::Path) -> Option<std::path::PathBuf>
```

Find the workspace root containing `start`.

Tries `git rev-parse --show-toplevel` first (fast, accurate when in a git repo),
then falls back to walking up to 3 parent directories looking for a `Cargo.toml`
with `[workspace]`.

### `project::parse_description_field`

```rust
fn parse_description_field(content: &str, field: &str) -> Option<String>
```

Parse a single field's value out of DCF-formatted `content` (the format
used by `DESCRIPTION` files), joining continuation lines onto the field
they extend.

### `scaffold::apply_plan`

```rust
fn apply_plan(root: &std::path::Path, prefix: &str, plan: &[PlanEntry], data: &TemplateData, fresh: bool) -> anyhow::Result<()>
```

Apply a scaffold plan rooted at `root`, reading templates from
`prefix` (e.g. `"templates/rpkg"` or `"templates/monorepo/rpkg"`).

`fresh` selects ignore-file semantics: a fresh scaffold writes the
filtered patterns outright (byte-identical to the R fresh-file path, see
minirextendr #1151); over an existing package (`init use`) the patterns
are appended with dedupe, mirroring `usethis::use_build_ignore()` /
`use_git_ignore()`. Every other file is overwritten — the template is the
source of truth, matching `use_template()`'s delete-first behavior.

### `scaffold::desc_ensure_r_floor`

```rust
fn desc_ensure_r_floor(content: &str) -> String
```

Ensure `Depends` declares at least the [`R_VERSION_FLOOR`] R version
floor (~ `mx_desc_ensure_r_floor()` in `minirextendr/R/utils.R`). Merges
rather than overwrites: no `Depends` field adds one carrying the floor;
an existing `Depends` without an `R` entry gains the floor (prepended,
other entries untouched); an `R` entry with a provably lower `>=`/`>`
floor — or no version constraint at all — is raised; a floor already at
or above ours, or a constraint using another operator, is left untouched.

### `scaffold::desc_set_field`

```rust
fn desc_set_field(content: &str, field: &str, value: &str) -> String
```

Set a single-line field in DCF `content`, replacing an existing field (and
its continuation lines) or appending (~ `mx_desc_set()`).

### `scaffold::description_content`

```rust
fn description_content(package: &str) -> String
```

Canonical fresh DESCRIPTION (~ the hand-written literal in
`create_rpkg_subdirectory()`, `minirextendr/R/create.R`).

### `scaffold::dest_rel`

```rust
fn dest_rel(entry: &PlanEntry, data: &TemplateData) -> anyhow::Result<std::path::PathBuf>
```

Resolve a plan entry's destination relative to the scaffold root.

### `scaffold::embedded`

```rust
fn embedded(rel: &str) -> &'static str
```

Look up an embedded file by its path relative to `minirextendr/inst/`.

Panics on a miss: the manifest is a compile-time constant, so a missing
key is a programming error, not a runtime condition.

### `scaffold::ignore_patterns`

```rust
fn ignore_patterns(content: &str) -> Vec<&str>
```

Ignore-file patterns: non-blank, non-comment lines
(~ `mx_ignore_patterns()` in `minirextendr/R/utils.R`).

### `scaffold::license_content`

```rust
fn license_content(package: &str) -> String
```

Minimal LICENSE body for `License: MIT + file LICENSE`
(~ `mx_license_content()`).

### `scaffold::minimal_namespace`

```rust
fn minimal_namespace(package: &str) -> String
```

Minimal roxygen2-managed NAMESPACE (~ `mx_minimal_namespace()`): carries
the roxygen2 header so a later `devtools::document()` overwrites it
cleanly, plus `useDynLib()` so the fresh shared library loads.

### `scaffold::r_depends_entry`

```rust
fn r_depends_entry() -> String
```

`Depends` entry carrying [`R_VERSION_FLOOR`] (~ `mx_r_depends_entry()`).

### `scaffold::render`

```rust
fn render(template_rel: &str, content: &str, render: Render, data: &TemplateData) -> anyhow::Result<String>
```

Render `content` according to `render`, substituting from `data`.

### `scaffold::to_rust_name`

```rust
fn to_rust_name(name: &str) -> String
```

Convert an R package name to a Rust-safe identifier (dots and hyphens
become underscores). Mirrors `to_rust_name()` in `minirextendr/R/utils.R`.

### `scaffold::write_config_scripts`

```rust
fn write_config_scripts(root: &std::path::Path) -> anyhow::Result<()>
```

Copy the bundled `config.guess` / `config.sub` scripts into `tools/`.

### `scaffold::write_file`

```rust
fn write_file(path: &std::path::Path, content: &str, exec: bool) -> anyhow::Result<()>
```

Write `content` to `path`, creating parent directories; set mode 755 when
`exec` (unix).

---

## Constants

### `project::MINIEXTENDR_CRATES`

```rust
pub const MINIEXTENDR_CRATES: &[&str] = _;
```

Miniextendr workspace crate names — the crates that get vendored/synced.

### `scaffold::CONFIG_BUILD_FIELDS`

```rust
pub const CONFIG_BUILD_FIELDS: &[(&str, &str)] = _;
```

`Config/build/*` DESCRIPTION fields required by miniextendr packages
(~ `MX_CONFIG_BUILD_FIELDS` in `minirextendr/R/utils.R`).

### `scaffold::CONFIG_SCRIPTS`

```rust
pub const CONFIG_SCRIPTS: &[(&str, &str)] = _;
```

Bundled autoconf helper scripts (`AC_CONFIG_AUX_DIR([tools])`), copied
755 into `tools/` (~ `copy_config_scripts()`).

### `scaffold::EMBEDDED`

```rust
pub const EMBEDDED: &[(&str, &str)] = _;
```

Every file under `minirextendr/inst/templates/` plus the bundled autoconf
helper scripts under `minirextendr/inst/scripts/`, embedded verbatim at
compile time. Keys are paths relative to `minirextendr/inst/`.

### `scaffold::MONOREPO_ROOT_PLAN`

```rust
pub const MONOREPO_ROOT_PLAN: &[PlanEntry] = _;
```

The monorepo workspace-root surface, mirroring
`create_miniextendr_monorepo()`. Note the root `.gitignore` is a full
mustache render (it substitutes `{{rpkg_name}}`), unlike the rpkg ignore
files, and `tools/bump-version.R` uses `copy_template()` semantics.

### `scaffold::RPKG_PLAN`

```rust
pub const RPKG_PLAN: &[PlanEntry] = _;
```

The R-package scaffold surface, mirroring `use_miniextendr()`'s standalone
path and `create_rpkg_subdirectory()`'s monorepo path (which write the
same files from `templates/rpkg/` and `templates/monorepo/rpkg/`
respectively). Render modes match the R helpers: `use_template()` entries
are [`Render::Mustache`], `fs::file_copy()` entries are
[`Render::Verbatim`], ignore files are [`Render::IgnoreFilter`].

### `scaffold::RUST_SYSTEM_REQUIREMENT`

```rust
pub const RUST_SYSTEM_REQUIREMENT: &str = "Rust (>= 1.85)";
```

`SystemRequirements` entry for the Rust toolchain (shared between the
fresh DESCRIPTION and the `init use` merge path).

### `scaffold::R_VERSION_FLOOR`

```rust
pub const R_VERSION_FLOOR: &str = "4.5";
```

Minimum R version required by miniextendr-backed packages — the runtime
calls `R_getVarEx` (the `Rf_findVarInFrame` replacement), which only
exists on R >= 4.5.0 (#1300), so any package linking miniextendr-api
inherits this floor (~ `MX_R_FLOOR` in `minirextendr/R/utils.R`; the
`r_floor_matches_minirextendr_and_rpkg` test asserts both mirrors and
`rpkg/DESCRIPTION` agree).
