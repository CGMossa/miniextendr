//! Scaffolding engine over the canonical `minirextendr` templates.
//!
//! The single scaffold source of truth is `minirextendr/inst/templates/`
//! (itself derived from `rpkg/` via the template sync pipeline — see the root
//! `CLAUDE.md`). This module embeds those templates at compile time with
//! `include_str!`, so `miniextendr init` renders exactly the same build
//! system as `minirextendr::use_miniextendr()` and can never drift into a
//! parallel handwritten implementation again (#1351). A template file added
//! on disk without a matching entry in [`EMBEDDED`] fails the
//! `embedded_covers_disk` test.
//!
//! The substitution contract mirrors the R scaffolder
//! (`minirextendr/R/utils.R`):
//!
//! - [`Render::Mustache`] ~ `use_template()` (whisker): `{{{key}}}` then
//!   `{{key}}` replacement. Divergence, deliberately stricter: an unresolved
//!   `{{identifier}}` placeholder is a batched error instead of silently
//!   rendering as the empty string, so a template that grows a new variable
//!   fails loudly here. HTML escaping never applies — substituted values are
//!   package/crate names, which cannot contain `&<>"`.
//! - [`Render::TripleOnly`] ~ `copy_template()`: literal `{{{key}}}`
//!   replacement only, double-brace text preserved, no leftover check.
//! - [`Render::Verbatim`] ~ `fs::file_copy()`.
//! - [`Render::IgnoreFilter`] ~ `mx_ignore_patterns()`: blank and `#` comment
//!   lines dropped, patterns written as-is.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

// region: embedded template manifest

/// Embed a file from `minirextendr/inst/` keyed by its path relative to that
/// directory.
macro_rules! tpl {
    ($rel:literal) => {
        (
            $rel,
            include_str!(concat!("../../minirextendr/inst/", $rel)),
        )
    };
}

/// Every file under `minirextendr/inst/templates/` plus the bundled autoconf
/// helper scripts under `minirextendr/inst/scripts/`, embedded verbatim at
/// compile time. Keys are paths relative to `minirextendr/inst/`.
pub const EMBEDDED: &[(&str, &str)] = &[
    tpl!("scripts/config.guess"),
    tpl!("scripts/config.sub"),
    tpl!("templates/README.md"),
    tpl!("templates/miniextendr.yml"),
    tpl!("templates/r-release.yml"),
    // Standalone R package template.
    tpl!("templates/rpkg/Cargo.toml.tmpl"),
    tpl!("templates/rpkg/Makevars.in"),
    tpl!("templates/rpkg/Makevars.win"),
    tpl!("templates/rpkg/Rbuildignore"),
    tpl!("templates/rpkg/bootstrap.R"),
    tpl!("templates/rpkg/build.rs"),
    tpl!("templates/rpkg/cleanup"),
    tpl!("templates/rpkg/cleanup.ucrt"),
    tpl!("templates/rpkg/cleanup.win"),
    tpl!("templates/rpkg/configure.ac"),
    tpl!("templates/rpkg/configure.ucrt"),
    tpl!("templates/rpkg/configure.win"),
    tpl!("templates/rpkg/gitignore"),
    tpl!("templates/rpkg/inst_include/mx_abi.h"),
    tpl!("templates/rpkg/lib.rs"),
    tpl!("templates/rpkg/package.R"),
    tpl!("templates/rpkg/r_shim.h"),
    tpl!("templates/rpkg/stub.c"),
    tpl!("templates/rpkg/tools/bump-version.R"),
    tpl!("templates/rpkg/tools/detect-features.R"),
    tpl!("templates/rpkg/tools/lock-shape-check.R"),
    tpl!("templates/rpkg/win.def.in"),
    // Monorepo template: workspace root + core crate.
    tpl!("templates/monorepo/Cargo.toml.tmpl"),
    tpl!("templates/monorepo/gitignore"),
    tpl!("templates/monorepo/my-crate/Cargo.toml.tmpl"),
    tpl!("templates/monorepo/my-crate/src/lib.rs"),
    tpl!("templates/monorepo/tools/bump-version.R"),
    // Monorepo template: embedded R package.
    tpl!("templates/monorepo/rpkg/Cargo.toml.tmpl"),
    tpl!("templates/monorepo/rpkg/Makevars.in"),
    tpl!("templates/monorepo/rpkg/Makevars.win"),
    tpl!("templates/monorepo/rpkg/Rbuildignore"),
    tpl!("templates/monorepo/rpkg/bootstrap.R"),
    tpl!("templates/monorepo/rpkg/build.rs"),
    tpl!("templates/monorepo/rpkg/cleanup"),
    tpl!("templates/monorepo/rpkg/cleanup.ucrt"),
    tpl!("templates/monorepo/rpkg/cleanup.win"),
    tpl!("templates/monorepo/rpkg/configure.ac"),
    tpl!("templates/monorepo/rpkg/configure.ucrt"),
    tpl!("templates/monorepo/rpkg/configure.win"),
    tpl!("templates/monorepo/rpkg/gitignore"),
    tpl!("templates/monorepo/rpkg/inst_include/mx_abi.h"),
    tpl!("templates/monorepo/rpkg/lib.rs"),
    tpl!("templates/monorepo/rpkg/package.R"),
    tpl!("templates/monorepo/rpkg/r_shim.h"),
    tpl!("templates/monorepo/rpkg/stub.c"),
    tpl!("templates/monorepo/rpkg/tools/detect-features.R"),
    tpl!("templates/monorepo/rpkg/tools/lock-shape-check.R"),
    tpl!("templates/monorepo/rpkg/win.def.in"),
];

/// Look up an embedded file by its path relative to `minirextendr/inst/`.
///
/// Panics on a miss: the manifest is a compile-time constant, so a missing
/// key is a programming error, not a runtime condition.
pub fn embedded(rel: &str) -> &'static str {
    EMBEDDED
        .iter()
        .find(|(path, _)| *path == rel)
        .map(|(_, content)| *content)
        .unwrap_or_else(|| panic!("template not embedded: {rel}"))
}

// endregion

// region: template data (substitution contract)

/// Convert an R package name to a Rust-safe identifier (dots and hyphens
/// become underscores). Mirrors `to_rust_name()` in `minirextendr/R/utils.R`.
pub fn to_rust_name(name: &str) -> String {
    name.replace(['.', '-'], "_")
}

/// Substitution variables for template rendering. Mirrors `template_data()`
/// in `minirextendr/R/utils.R`: `package`, `package_rs`, `Package`,
/// `features_var`, `year`, plus optional `crate_name`/`crate_name_rs` and
/// `rpkg_name` for the monorepo template.
pub struct TemplateData {
    pub package: String,
    pub crate_name: Option<String>,
    pairs: Vec<(&'static str, String)>,
}

impl TemplateData {
    pub fn new(package: &str) -> Self {
        let pairs = vec![
            ("package", package.to_string()),
            ("package_rs", to_rust_name(package)),
            ("Package", title_case(package)),
            ("features_var", "CARGO_FEATURES".to_string()),
            ("year", current_year()),
        ];
        Self {
            package: package.to_string(),
            crate_name: None,
            pairs,
        }
    }

    pub fn with_crate(mut self, crate_name: &str) -> Self {
        self.pairs.push(("crate_name", crate_name.to_string()));
        self.pairs.push(("crate_name_rs", to_rust_name(crate_name)));
        self.crate_name = Some(crate_name.to_string());
        self
    }

    pub fn with_rpkg(mut self, rpkg_name: &str) -> Self {
        self.pairs.push(("rpkg_name", rpkg_name.to_string()));
        self
    }

    pub fn pairs(&self) -> &[(&'static str, String)] {
        &self.pairs
    }
}

/// Title-case a package name for the `{{Package}}` variable
/// (~ `tools::toTitleCase()`): uppercase the first alphabetic character of
/// each whitespace-separated word. Package names contain no whitespace, so in
/// practice only the leading character changes. No scaffolded template
/// consumes `{{Package}}` today (it appears only in the templates README).
fn title_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut at_word_start = true;
    for c in name.chars() {
        if c.is_whitespace() {
            at_word_start = true;
            out.push(c);
        } else if at_word_start && c.is_ascii_alphabetic() {
            out.push(c.to_ascii_uppercase());
            at_word_start = false;
        } else {
            at_word_start = false;
            out.push(c);
        }
    }
    out
}

/// Current UTC year, without a date-crate dependency (civil-from-days,
/// Howard Hinnant's algorithm).
fn current_year() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    civil_year_from_days(i64::try_from(secs / 86_400).unwrap_or(0)).to_string()
}

fn civil_year_from_days(days: i64) -> i64 {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 { y + 1 } else { y }
}

// endregion

// region: rendering

/// How a template file's content is transformed before writing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Render {
    /// `{{{key}}}` then `{{key}}` substitution; unresolved `{{identifier}}`
    /// placeholders are a (batched) error.
    Mustache,
    /// Literal `{{{key}}}` substitution only; `{{...}}` preserved.
    TripleOnly,
    /// Byte-for-byte copy.
    Verbatim,
    /// Ignore-file prep: drop blank lines and `#` comments.
    IgnoreFilter,
}

/// Render `content` according to `render`, substituting from `data`.
pub fn render(
    template_rel: &str,
    content: &str,
    render: Render,
    data: &TemplateData,
) -> Result<String> {
    match render {
        Render::Verbatim => Ok(content.to_string()),
        Render::IgnoreFilter => Ok(ignore_patterns(content).join("\n") + "\n"),
        Render::TripleOnly => {
            let mut out = content.to_string();
            for (key, value) in data.pairs() {
                out = out.replace(&format!("{{{{{{{key}}}}}}}"), value);
            }
            Ok(out)
        }
        Render::Mustache => {
            let mut out = content.to_string();
            for (key, value) in data.pairs() {
                out = out.replace(&format!("{{{{{{{key}}}}}}}"), value);
                out = out.replace(&format!("{{{{{key}}}}}"), value);
            }
            let leftovers = find_placeholders(&out);
            if !leftovers.is_empty() {
                bail!(
                    "template {template_rel} has unresolved placeholders: {} \
                     (the template grew a variable the CLI does not supply — \
                     update TemplateData in miniextendr-cli/src/scaffold.rs)",
                    leftovers.join(", ")
                );
            }
            Ok(out)
        }
    }
}

/// Ignore-file patterns: non-blank, non-comment lines
/// (~ `mx_ignore_patterns()` in `minirextendr/R/utils.R`).
pub fn ignore_patterns(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Find `{{identifier}}` / `{{{identifier}}}`-shaped placeholders left in
/// rendered output. Returns each distinct placeholder once, in order.
fn find_placeholders(content: &str) -> Vec<String> {
    let bytes = content.as_bytes();
    let mut found: Vec<String> = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if &bytes[i..i + 2] != b"{{" {
            i += 1;
            continue;
        }
        let mut j = i + 2;
        if j < bytes.len() && bytes[j] == b'{' {
            j += 1;
        }
        let ident_start = j;
        if j < bytes.len() && (bytes[j].is_ascii_alphabetic() || bytes[j] == b'_') {
            j += 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if content[j..].starts_with("}}") {
                let end = if content[j..].starts_with("}}}") {
                    j + 3
                } else {
                    j + 2
                };
                let placeholder = format!("{{{{{}}}}}", &content[ident_start..j]);
                if !found.contains(&placeholder) {
                    found.push(placeholder);
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    found
}

// endregion

// region: scaffold plans

/// Destination of a scaffolded file, relative to the scaffold root.
#[derive(Clone, Copy, Debug)]
pub enum Dest {
    /// Fixed relative path.
    Path(&'static str),
    /// `R/<package>-package.R`.
    PackageDoc,
    /// `<crate_name>/<path>` (monorepo core crate).
    CratePath(&'static str),
}

/// One template → destination mapping.
#[derive(Clone, Copy, Debug)]
pub struct PlanEntry {
    /// Template path relative to the template-type directory (the `prefix`
    /// argument of [`apply_plan`]).
    pub template: &'static str,
    pub dest: Dest,
    pub render: Render,
    /// Whether the destination gets mode 755 (R build hooks must be
    /// executable — `R CMD build` warns otherwise).
    pub exec: bool,
}

/// The R-package scaffold surface, mirroring `use_miniextendr()`'s standalone
/// path and `create_rpkg_subdirectory()`'s monorepo path (which write the
/// same files from `templates/rpkg/` and `templates/monorepo/rpkg/`
/// respectively). Render modes match the R helpers: `use_template()` entries
/// are [`Render::Mustache`], `fs::file_copy()` entries are
/// [`Render::Verbatim`], ignore files are [`Render::IgnoreFilter`].
pub const RPKG_PLAN: &[PlanEntry] = &[
    PlanEntry {
        template: "configure.ac",
        dest: Dest::Path("configure.ac"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "bootstrap.R",
        dest: Dest::Path("bootstrap.R"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "cleanup",
        dest: Dest::Path("cleanup"),
        render: Render::Mustache,
        exec: true,
    },
    PlanEntry {
        template: "cleanup.win",
        dest: Dest::Path("cleanup.win"),
        render: Render::Mustache,
        exec: true,
    },
    PlanEntry {
        template: "cleanup.ucrt",
        dest: Dest::Path("cleanup.ucrt"),
        render: Render::Mustache,
        exec: true,
    },
    PlanEntry {
        template: "configure.win",
        dest: Dest::Path("configure.win"),
        render: Render::Mustache,
        exec: true,
    },
    PlanEntry {
        template: "configure.ucrt",
        dest: Dest::Path("configure.ucrt"),
        render: Render::Mustache,
        exec: true,
    },
    PlanEntry {
        template: "Makevars.in",
        dest: Dest::Path("src/Makevars.in"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "Makevars.win",
        dest: Dest::Path("src/Makevars.win"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "win.def.in",
        dest: Dest::Path("src/win.def.in"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "stub.c",
        dest: Dest::Path("src/stub.c"),
        render: Render::Verbatim,
        exec: false,
    },
    PlanEntry {
        template: "r_shim.h",
        dest: Dest::Path("src/r_shim.h"),
        render: Render::Verbatim,
        exec: false,
    },
    PlanEntry {
        template: "Cargo.toml.tmpl",
        dest: Dest::Path("src/rust/Cargo.toml"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "build.rs",
        dest: Dest::Path("src/rust/build.rs"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "lib.rs",
        dest: Dest::Path("src/rust/lib.rs"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "inst_include/mx_abi.h",
        dest: Dest::Path("inst/include/mx_abi.h"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "package.R",
        dest: Dest::PackageDoc,
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "tools/lock-shape-check.R",
        dest: Dest::Path("tools/lock-shape-check.R"),
        render: Render::Verbatim,
        exec: false,
    },
    PlanEntry {
        template: "Rbuildignore",
        dest: Dest::Path(".Rbuildignore"),
        render: Render::IgnoreFilter,
        exec: false,
    },
    PlanEntry {
        template: "gitignore",
        dest: Dest::Path(".gitignore"),
        render: Render::IgnoreFilter,
        exec: false,
    },
];

/// The monorepo workspace-root surface, mirroring
/// `create_miniextendr_monorepo()`. Note the root `.gitignore` is a full
/// mustache render (it substitutes `{{rpkg_name}}`), unlike the rpkg ignore
/// files, and `tools/bump-version.R` uses `copy_template()` semantics.
pub const MONOREPO_ROOT_PLAN: &[PlanEntry] = &[
    PlanEntry {
        template: "Cargo.toml.tmpl",
        dest: Dest::Path("Cargo.toml"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "gitignore",
        dest: Dest::Path(".gitignore"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "tools/bump-version.R",
        dest: Dest::Path("tools/bump-version.R"),
        render: Render::TripleOnly,
        exec: false,
    },
    PlanEntry {
        template: "my-crate/Cargo.toml.tmpl",
        dest: Dest::CratePath("Cargo.toml"),
        render: Render::Mustache,
        exec: false,
    },
    PlanEntry {
        template: "my-crate/src/lib.rs",
        dest: Dest::CratePath("src/lib.rs"),
        render: Render::Mustache,
        exec: false,
    },
];

/// Bundled autoconf helper scripts (`AC_CONFIG_AUX_DIR([tools])`), copied
/// 755 into `tools/` (~ `copy_config_scripts()`).
pub const CONFIG_SCRIPTS: &[(&str, &str)] = &[
    ("scripts/config.guess", "tools/config.guess"),
    ("scripts/config.sub", "tools/config.sub"),
];

// endregion

// region: writing

/// Resolve a plan entry's destination relative to the scaffold root.
pub fn dest_rel(entry: &PlanEntry, data: &TemplateData) -> Result<PathBuf> {
    Ok(match entry.dest {
        Dest::Path(p) => PathBuf::from(p),
        Dest::PackageDoc => PathBuf::from(format!("R/{}-package.R", data.package)),
        Dest::CratePath(p) => {
            let crate_name = data
                .crate_name
                .as_deref()
                .context("plan entry needs a crate_name but none was set")?;
            Path::new(crate_name).join(p)
        }
    })
}

/// Apply a scaffold plan rooted at `root`, reading templates from
/// `prefix` (e.g. `"templates/rpkg"` or `"templates/monorepo/rpkg"`).
///
/// `fresh` selects ignore-file semantics: a fresh scaffold writes the
/// filtered patterns outright (byte-identical to the R fresh-file path, see
/// minirextendr #1151); over an existing package (`init use`) the patterns
/// are appended with dedupe, mirroring `usethis::use_build_ignore()` /
/// `use_git_ignore()`. Every other file is overwritten — the template is the
/// source of truth, matching `use_template()`'s delete-first behavior.
pub fn apply_plan(
    root: &Path,
    prefix: &str,
    plan: &[PlanEntry],
    data: &TemplateData,
    fresh: bool,
) -> Result<()> {
    for entry in plan {
        let template_rel = format!("{prefix}/{}", entry.template);
        let content = embedded(&template_rel);
        let dest = root.join(dest_rel(entry, data)?);

        if entry.render == Render::IgnoreFilter && !fresh && dest.is_file() {
            let existing = std::fs::read_to_string(&dest)
                .with_context(|| format!("failed to read {}", dest.display()))?;
            let merged = merge_ignore_lines(&existing, &ignore_patterns(content));
            write_file(&dest, &merged, entry.exec)?;
            continue;
        }

        let rendered = render(&template_rel, content, entry.render, data)?;
        write_file(&dest, &rendered, entry.exec)?;
    }
    Ok(())
}

/// Copy the bundled `config.guess` / `config.sub` scripts into `tools/`.
pub fn write_config_scripts(root: &Path) -> Result<()> {
    for (src, dest) in CONFIG_SCRIPTS {
        write_file(&root.join(dest), embedded(src), true)?;
    }
    Ok(())
}

/// Append `patterns` not already present in `existing` (exact line match),
/// preserving existing content.
fn merge_ignore_lines(existing: &str, patterns: &[&str]) -> String {
    let existing_lines: Vec<&str> = existing.lines().collect();
    let mut out = existing.trim_end_matches('\n').to_string();
    for pattern in patterns {
        if !existing_lines.contains(pattern) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(pattern);
        }
    }
    out.push('\n');
    out
}

/// Write `content` to `path`, creating parent directories; set mode 755 when
/// `exec` (unix).
pub fn write_file(path: &Path, content: &str, exec: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    #[cfg(unix)]
    if exec {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to chmod {}", path.display()))?;
    }
    Ok(())
}

// endregion

// region: shared scaffold content (mirrors of minirextendr R literals)

/// `Config/build/*` DESCRIPTION fields required by miniextendr packages
/// (~ `MX_CONFIG_BUILD_FIELDS` in `minirextendr/R/utils.R`).
pub const CONFIG_BUILD_FIELDS: &[(&str, &str)] = &[
    ("Config/build/bootstrap", "TRUE"),
    ("Config/build/never-clean", "true"),
    ("Config/build/extra-sources", "src/rust/Cargo.lock"),
];

/// `SystemRequirements` entry for the Rust toolchain (shared between the
/// fresh DESCRIPTION and the `init use` merge path).
pub const RUST_SYSTEM_REQUIREMENT: &str = "Rust (>= 1.85)";

/// Minimum R version required by miniextendr-backed packages — the runtime
/// calls `R_getVarEx` (the `Rf_findVarInFrame` replacement), which only
/// exists on R >= 4.5.0 (#1300), so any package linking miniextendr-api
/// inherits this floor (~ `MX_R_FLOOR` in `minirextendr/R/utils.R`; the
/// `r_floor_matches_minirextendr_and_rpkg` test asserts both mirrors and
/// `rpkg/DESCRIPTION` agree).
pub const R_VERSION_FLOOR: &str = "4.5";

/// `Depends` entry carrying [`R_VERSION_FLOOR`] (~ `mx_r_depends_entry()`).
pub fn r_depends_entry() -> String {
    format!("R (>= {R_VERSION_FLOOR})")
}

/// Canonical fresh DESCRIPTION (~ the hand-written literal in
/// `create_rpkg_subdirectory()`, `minirextendr/R/create.R`).
pub fn description_content(package: &str) -> String {
    let config_build = CONFIG_BUILD_FIELDS
        .iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    let r_depends = r_depends_entry();
    format!(
        "Package: {package}\n\
         Title: What the Package Does (One Line, Title Case)\n\
         Version: 0.0.0.9000\n\
         Depends: {r_depends}\n\
         Authors@R:\n    \
         person(\"First\", \"Last\", , \"first.last@example.com\", role = c(\"aut\", \"cre\"))\n\
         Description: What the package does (one paragraph).\n\
         License: MIT + file LICENSE\n\
         Encoding: UTF-8\n\
         SystemRequirements: {RUST_SYSTEM_REQUIREMENT}\n\
         {config_build}\n\
         Config/roxygen2/markdown: TRUE\n\
         Config/roxygen2/version: 8.0.0\n"
    )
}

/// Minimal LICENSE body for `License: MIT + file LICENSE`
/// (~ `mx_license_content()`).
pub fn license_content(package: &str) -> String {
    format!(
        "YEAR: {}\nCOPYRIGHT HOLDER: {package} authors\n",
        current_year()
    )
}

/// Minimal roxygen2-managed NAMESPACE (~ `mx_minimal_namespace()`): carries
/// the roxygen2 header so a later `devtools::document()` overwrites it
/// cleanly, plus `useDynLib()` so the fresh shared library loads.
pub fn minimal_namespace(package: &str) -> String {
    format!(
        "# Generated by roxygen2: do not edit by hand\n\nuseDynLib({package}, .registration = TRUE)\n"
    )
}

// endregion

// region: DESCRIPTION editing (init use)

/// Set a single-line field in DCF `content`, replacing an existing field (and
/// its continuation lines) or appending (~ `mx_desc_set()`).
pub fn desc_set_field(content: &str, field: &str, value: &str) -> String {
    let prefix = format!("{field}:");
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 1);
    let new_line = format!("{field}: {value}");
    let mut replaced = false;
    let mut i = 0;
    while i < lines.len() {
        if !replaced && lines[i].starts_with(&prefix) {
            out.push(new_line.clone());
            i += 1;
            // Skip continuation lines of the replaced field.
            while i < lines.len() && lines[i].starts_with(|c: char| c.is_whitespace()) {
                i += 1;
            }
            replaced = true;
            continue;
        }
        out.push(lines[i].to_string());
        i += 1;
    }
    if !replaced {
        out.push(new_line);
    }
    out.join("\n") + "\n"
}

/// Ensure `Depends` declares at least the [`R_VERSION_FLOOR`] R version
/// floor (~ `mx_desc_ensure_r_floor()` in `minirextendr/R/utils.R`). Merges
/// rather than overwrites: no `Depends` field adds one carrying the floor;
/// an existing `Depends` without an `R` entry gains the floor (prepended,
/// other entries untouched); an `R` entry with a provably lower `>=`/`>`
/// floor — or no version constraint at all — is raised; a floor already at
/// or above ours, or a constraint using another operator, is left untouched.
pub fn desc_ensure_r_floor(content: &str) -> String {
    let floor_entry = r_depends_entry();
    let Some(depends) = crate::project::parse_description_field(content, "Depends") else {
        return desc_set_field(content, "Depends", &floor_entry);
    };
    let mut entries: Vec<String> = depends
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect();
    let r_idx = entries
        .iter()
        .position(|entry| entry.split('(').next().unwrap_or("").trim() == "R");
    match r_idx {
        None => entries.insert(0, floor_entry),
        Some(i) => {
            let entry = &entries[i];
            match entry.find('(') {
                // Bare `R`: no floor at all — raise to ours.
                None => entries[i] = floor_entry,
                Some(open) => {
                    let constraint = entry[open + 1..].trim_end_matches(')').trim();
                    let version = constraint
                        .strip_prefix(">=")
                        .or_else(|| constraint.strip_prefix('>'))
                        .map(str::trim);
                    match version {
                        Some(v) if version_less_than(v, R_VERSION_FLOOR) => {
                            entries[i] = floor_entry;
                        }
                        // Already at or above the floor, or a constraint we
                        // can't compare (e.g. `R (== 4.4)`): leave untouched.
                        _ => return content.to_string(),
                    }
                }
            }
        }
    }
    desc_set_field(content, "Depends", &entries.join(", "))
}

/// `a < b` over dotted version strings, matching R's
/// `utils::compareVersion()` closely enough for R release versions: numeric
/// component-wise comparison where a missing component sorts lower
/// (`4.5 < 4.5.0`). Non-numeric components compare as 0.
fn version_less_than(a: &str, b: &str) -> bool {
    fn parts(version: &str) -> Vec<u64> {
        version
            .split(['.', '-'])
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    }
    parts(a) < parts(b)
}

// endregion

#[cfg(test)]
mod tests {
    use super::*;

    /// `minirextendr/inst/` in the repo checkout — the disk source the
    /// embedded manifest must mirror.
    fn inst_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../minirextendr/inst")
    }

    fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable template dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk_files(&path, out);
            } else {
                out.push(path);
            }
        }
    }

    /// Drift guard: every file under `minirextendr/inst/templates/` and
    /// `minirextendr/inst/scripts/` must be embedded with identical content,
    /// and every embedded key must still exist on disk. A template added,
    /// renamed, or removed without updating [`EMBEDDED`] fails here.
    #[test]
    fn embedded_covers_disk() {
        let inst = inst_root();
        let mut disk_files = Vec::new();
        walk_files(&inst.join("templates"), &mut disk_files);
        walk_files(&inst.join("scripts"), &mut disk_files);

        let mut problems = Vec::new();
        let mut seen = Vec::new();
        for path in &disk_files {
            let rel = path
                .strip_prefix(&inst)
                .expect("under inst root")
                .to_string_lossy()
                .replace('\\', "/");
            seen.push(rel.clone());
            match EMBEDDED.iter().find(|(p, _)| *p == rel) {
                None => problems.push(format!(
                    "{rel}: on disk but not embedded — add tpl!(\"{rel}\") to EMBEDDED"
                )),
                Some((_, embedded_content)) => {
                    let disk_content = std::fs::read_to_string(path).expect("readable template");
                    if disk_content != *embedded_content {
                        problems.push(format!(
                            "{rel}: embedded copy differs from disk (stale build?)"
                        ));
                    }
                }
            }
        }
        for (rel, _) in EMBEDDED {
            if !seen.contains(&(*rel).to_string()) {
                problems.push(format!(
                    "{rel}: embedded but missing on disk — template removed or renamed?"
                ));
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn mustache_substitutes_triple_then_double() {
        let data = TemplateData::new("my.pkg").with_rpkg("rp");
        let out = render(
            "t",
            "x {{{rpkg_name}}} y {{package}} z {{package_rs}}",
            Render::Mustache,
            &data,
        )
        .unwrap();
        assert_eq!(out, "x rp y my.pkg z my_pkg");
    }

    #[test]
    fn mustache_rejects_unknown_placeholders_batched() {
        let data = TemplateData::new("p");
        let err = render(
            "t",
            "{{unknown_a}} and {{unknown_b}} and {{unknown_a}}",
            Render::Mustache,
            &data,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("{{unknown_a}}"), "{err}");
        assert!(err.contains("{{unknown_b}}"), "{err}");
    }

    #[test]
    fn mustache_ignores_non_placeholder_braces() {
        let data = TemplateData::new("p");
        // Rust format strings, shell ${...}, R code — none are placeholders.
        let content = "format!(\"Hello, {}!\", name); ${srcdir}; if (x) {{ }}";
        assert_eq!(
            render("t", content, Render::Mustache, &data).unwrap(),
            content
        );
    }

    #[test]
    fn triple_only_preserves_double_braces() {
        let data = TemplateData::new("p").with_rpkg("rp");
        let out = render(
            "t",
            "{{{rpkg_name}}} keeps {{rpkg_name}}",
            Render::TripleOnly,
            &data,
        )
        .unwrap();
        assert_eq!(out, "rp keeps {{rpkg_name}}");
    }

    #[test]
    fn ignore_filter_drops_blanks_and_comments() {
        let data = TemplateData::new("p");
        let out = render(
            "t",
            "# comment\n\npattern1\npattern2\n",
            Render::IgnoreFilter,
            &data,
        )
        .unwrap();
        assert_eq!(out, "pattern1\npattern2\n");
    }

    #[test]
    fn merge_ignore_appends_missing_only() {
        let merged = merge_ignore_lines("existing\npattern1\n", &["pattern1", "pattern2"]);
        assert_eq!(merged, "existing\npattern1\npattern2\n");
    }

    /// Cross-language drift guard for the R version floor (#1366): the Rust
    /// mirror, the R-side `MX_R_FLOOR` constant, and `rpkg/DESCRIPTION`'s
    /// `Depends` line (the exemplar the floor is derived from) must all agree.
    #[test]
    fn r_floor_matches_minirextendr_and_rpkg() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");

        let utils_r = std::fs::read_to_string(repo_root.join("minirextendr/R/utils.R"))
            .expect("readable minirextendr/R/utils.R");
        let r_line = utils_r
            .lines()
            .find(|line| line.starts_with("MX_R_FLOOR"))
            .expect("MX_R_FLOOR defined in minirextendr/R/utils.R");
        assert_eq!(
            r_line.trim(),
            format!("MX_R_FLOOR <- \"{R_VERSION_FLOOR}\""),
            "R_VERSION_FLOOR drifted from minirextendr's MX_R_FLOOR"
        );

        let rpkg_desc = std::fs::read_to_string(repo_root.join("rpkg/DESCRIPTION"))
            .expect("readable rpkg/DESCRIPTION");
        assert!(
            rpkg_desc
                .lines()
                .any(|line| line.trim() == format!("Depends: {}", r_depends_entry())),
            "rpkg/DESCRIPTION's Depends floor drifted from R_VERSION_FLOOR"
        );
    }

    #[test]
    fn desc_ensure_r_floor_adds_depends_when_absent() {
        let content = "Package: p\nVersion: 1.0.0\n";
        let out = desc_ensure_r_floor(content);
        assert!(
            out.contains(&format!("Depends: {}\n", r_depends_entry())),
            "{out}"
        );
    }

    #[test]
    fn desc_ensure_r_floor_prepends_to_foreign_depends() {
        let content = "Package: p\nDepends: methods, utils\n";
        let out = desc_ensure_r_floor(content);
        assert!(
            out.contains(&format!("Depends: {}, methods, utils\n", r_depends_entry())),
            "{out}"
        );
    }

    #[test]
    fn desc_ensure_r_floor_raises_lower_floor() {
        for lower in ["R (>= 4.4)", "R (>=4.4)", "R (> 4.4)", "R"] {
            let content = format!("Package: p\nDepends: {lower}, methods\n");
            let out = desc_ensure_r_floor(&content);
            assert!(
                out.contains(&format!("Depends: {}, methods\n", r_depends_entry())),
                "{lower}: {out}"
            );
        }
    }

    #[test]
    fn desc_ensure_r_floor_keeps_equal_or_higher_floor() {
        for kept in ["R (>= 4.5)", "R (>= 4.5.0)", "R (>= 4.6)", "R (== 4.4)"] {
            let content = format!("Package: p\nDepends: {kept}, methods\n");
            assert_eq!(desc_ensure_r_floor(&content), content, "{kept}");
        }
    }

    #[test]
    fn version_less_than_matches_compare_version() {
        assert!(version_less_than("4.4", "4.5"));
        assert!(version_less_than("4.4.9", "4.5"));
        assert!(version_less_than("4.5", "4.5.0"));
        assert!(!version_less_than("4.5", "4.5"));
        assert!(!version_less_than("4.10", "4.5"));
        assert!(!version_less_than("4.5.1", "4.5"));
    }

    #[test]
    fn desc_set_field_replaces_with_continuations() {
        let content = "Package: p\nAuthors@R:\n    person(\"A\")\nLicense: old\n";
        let out = desc_set_field(content, "License", "MIT + file LICENSE");
        assert_eq!(
            out,
            "Package: p\nAuthors@R:\n    person(\"A\")\nLicense: MIT + file LICENSE\n"
        );
        let out = desc_set_field(&out, "NeedsCompilation", "yes");
        assert!(out.ends_with("NeedsCompilation: yes\n"));
        // Replacing a field with continuation lines drops the continuation.
        let out = desc_set_field(content, "Authors@R", "person(\"B\")");
        assert!(out.contains("Authors@R: person(\"B\")\n"));
        assert!(!out.contains("person(\"A\")"));
    }

    #[test]
    fn year_is_plausible() {
        let year: i64 = current_year().parse().unwrap();
        assert!((2026..2200).contains(&year), "year = {year}");
    }

    #[test]
    fn to_rust_name_replaces_dots_and_hyphens() {
        assert_eq!(to_rust_name("my.pkg-x"), "my_pkg_x");
    }
}
