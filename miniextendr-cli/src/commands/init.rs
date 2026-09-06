//! `miniextendr init` — scaffold packages from the canonical minirextendr
//! templates.
//!
//! Every file comes from the embedded copy of `minirextendr/inst/templates/`
//! (see [`crate::scaffold`]), so a CLI-generated package uses exactly the
//! same build system as `minirextendr::create_miniextendr_package()` /
//! `create_miniextendr_monorepo()` / `use_miniextendr()`: the two-mode
//! install latch keyed on `inst/vendor.tar.xz` presence, configure-written
//! `.cargo/config.toml`, wrapper + wasm-registry generation, and no `just`
//! requirement (#1351).

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::bridge::{has_program, run_command};
use crate::cli::InitCmd;
use crate::project::{ProjectContext, parse_description_field};
use crate::scaffold::{self, MONOREPO_ROOT_PLAN, RPKG_PLAN, RUST_SYSTEM_REQUIREMENT, TemplateData};

pub fn dispatch(cmd: &InitCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        InitCmd::Package { dest } => init_package(dest, quiet),
        InitCmd::Monorepo {
            dest,
            package,
            crate_name,
            rpkg_name,
            local_path: _,
            miniextendr_version: _,
        } => init_monorepo(
            dest,
            package.as_deref(),
            crate_name.as_deref(),
            rpkg_name.as_deref(),
            quiet,
        ),
        InitCmd::Use {
            template_type,
            rpkg_name,
            miniextendr_version: _,
            local_path: _,
        } => init_use(ctx, template_type, rpkg_name.as_deref(), quiet),
    }
}

// region: init package

/// Create a new standalone R package
/// (~ `minirextendr::create_miniextendr_package()`).
fn init_package(path: &str, quiet: bool) -> Result<()> {
    let root = Path::new(path);
    if root.exists() {
        bail!("Directory already exists: {path}");
    }

    let pkg_name = package_name_from_path(root)?;
    validate_package_name(&pkg_name)?;

    if !quiet {
        eprintln!("Creating R package: {pkg_name}");
    }

    let data = TemplateData::new(&pkg_name);
    scaffold_rpkg_fresh(root, &data)?;
    write_miniextendr_yml(root, quiet)?;
    run_autoconf(root, quiet);

    if !quiet {
        eprintln!("\nPackage created at: {path}");
        eprintln!("Next steps:");
        eprintln!("  cd {path}");
        eprintln!("  # edit src/rust/lib.rs, then:");
        eprintln!("  miniextendr workflow build");
    }
    Ok(())
}

// endregion

// region: init monorepo

/// Create a Rust workspace with an embedded R package
/// (~ `minirextendr::create_miniextendr_monorepo()`).
fn init_monorepo(
    path: &str,
    package: Option<&str>,
    crate_name: Option<&str>,
    rpkg_name: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let root = Path::new(path);
    if root.exists() {
        bail!("Directory already exists: {path}");
    }

    // Defaults mirror create_miniextendr_monorepo(): package from the
    // directory name, crate name = package with dots as dashes, R package
    // subdirectory named after the package.
    let pkg_name = match package {
        Some(p) => p.to_string(),
        None => package_name_from_path(root)?,
    };
    validate_package_name(&pkg_name)?;
    let crate_name = crate_name
        .map(str::to_string)
        .unwrap_or_else(|| pkg_name.replace('.', "-"));
    let rpkg_name = rpkg_name.map(str::to_string).unwrap_or_else(|| {
        // rpkg_name defaults to the package name; when that collides with the
        // crate directory (dot-less package names), fall back to "rpkg"
        // instead of erroring like R does — same layout the templates
        // document, one less required flag.
        if pkg_name == crate_name {
            "rpkg".to_string()
        } else {
            pkg_name.clone()
        }
    });
    if rpkg_name == crate_name {
        bail!(
            "--rpkg-name and --crate-name must be different (both are \"{crate_name}\" — \
             they are sibling directories under the project root).\n\
             Pass e.g. --rpkg-name {crate_name}-rpkg"
        );
    }

    if !quiet {
        eprintln!("Creating monorepo: {}", root.display());
        eprintln!("  package:   {pkg_name}");
        eprintln!("  crate:     {crate_name}/");
        eprintln!("  R package: {rpkg_name}/");
    }

    let data = TemplateData::new(&pkg_name)
        .with_crate(&crate_name)
        .with_rpkg(&rpkg_name);

    // Workspace root: Cargo.toml, .gitignore, tools/bump-version.R, core crate.
    scaffold::apply_plan(root, "templates/monorepo", MONOREPO_ROOT_PLAN, &data, true)?;

    // Embedded R package (~ create_rpkg_subdirectory()).
    let rpkg_root = root.join(&rpkg_name);
    scaffold_rpkg_at(&rpkg_root, "templates/monorepo/rpkg", &data)?;
    run_autoconf(&rpkg_root, quiet);

    if !quiet {
        eprintln!("\nMonorepo created at: {path}");
        eprintln!("Next steps:");
        eprintln!("  1. Edit {crate_name}/src/lib.rs for your core Rust library");
        eprintln!("  2. Edit {rpkg_name}/src/rust/lib.rs for R-exposed functions");
        eprintln!("  3. cd {path}/{rpkg_name} && miniextendr workflow build");
    }
    Ok(())
}

// endregion

// region: init use

/// Add miniextendr scaffolding to an existing project
/// (~ `minirextendr::use_miniextendr()`), auto-detecting standalone-package
/// vs monorepo layout.
fn init_use(
    ctx: &ProjectContext,
    template_type: &str,
    rpkg_name: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let root = &ctx.root;

    let template_type = match template_type {
        "auto" => {
            // Mirror detect_project_type(): a Cargo.toml at the project root
            // means "Rust project, embed an R package"; a DESCRIPTION means
            // "standalone R package".
            if root.join("Cargo.toml").is_file() {
                "monorepo"
            } else if root.join("DESCRIPTION").is_file() {
                "rpkg"
            } else {
                bail!(
                    "Could not detect project type: no Cargo.toml or DESCRIPTION in {}.\n\
                     Use `miniextendr init package <path>` to create a new package, or pass\n\
                     --template-type rpkg|monorepo explicitly.",
                    root.display()
                );
            }
        }
        t @ ("rpkg" | "monorepo") => t,
        other => bail!("Unknown template type: {other} (expected auto, rpkg, or monorepo)"),
    };

    match template_type {
        "monorepo" => init_use_monorepo(root, rpkg_name, quiet),
        _ => init_use_rpkg(root, quiet),
    }
}

/// `init use` in a Rust workspace: create the R package subdirectory.
fn init_use_monorepo(root: &Path, rpkg_name: Option<&str>, quiet: bool) -> Result<()> {
    // Mirror get_package_name_from_cargo(): crate name from the root
    // Cargo.toml, hyphens become dots for the R package name.
    let cargo_toml = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;
    let crate_name = content
        .lines()
        .find_map(|line| {
            let rest = line.strip_prefix("name")?.trim_start().strip_prefix('=')?;
            rest.trim().strip_prefix('"')?.split('"').next()
        })
        .with_context(|| {
            format!(
                "could not find a package name in {} — the monorepo template reads the crate \
                 name from the workspace Cargo.toml",
                cargo_toml.display()
            )
        })?
        .to_string();
    let pkg_name = crate_name.replace('-', ".");
    let rpkg_name = rpkg_name.map(str::to_string).unwrap_or_else(|| {
        if pkg_name == crate_name {
            "rpkg".to_string()
        } else {
            pkg_name.clone()
        }
    });

    if !quiet {
        eprintln!("Detected Rust project — creating R package in {rpkg_name}/");
        eprintln!("Using package name: {pkg_name}");
    }

    let data = TemplateData::new(&pkg_name)
        .with_crate(&crate_name)
        .with_rpkg(&rpkg_name);
    let rpkg_root = root.join(&rpkg_name);
    scaffold_rpkg_at(&rpkg_root, "templates/monorepo/rpkg", &data)?;
    run_autoconf(&rpkg_root, quiet);
    write_miniextendr_yml(root, quiet)?;

    if !quiet {
        eprintln!("\nminiextendr scaffolding added.");
        eprintln!("Next: cd {rpkg_name} && miniextendr workflow build");
    }
    Ok(())
}

/// `init use` in an R package directory: add scaffolding in place.
fn init_use_rpkg(root: &Path, quiet: bool) -> Result<()> {
    let desc_path = root.join("DESCRIPTION");
    let pkg_name = if desc_path.is_file() {
        let content = std::fs::read_to_string(&desc_path)?;
        parse_description_field(&content, "Package")
            .context("Could not read Package field from DESCRIPTION")?
    } else {
        // ~ use_miniextendr_description(): seed a minimal DESCRIPTION when
        // none exists, deriving the package name from the directory.
        let pkg_name = package_name_from_path(root)?;
        validate_package_name(&pkg_name)?;
        std::fs::write(&desc_path, scaffold::description_content(&pkg_name))?;
        if !quiet {
            eprintln!("No DESCRIPTION found; created a minimal one");
        }
        pkg_name
    };

    if !quiet {
        eprintln!("Adding miniextendr to: {pkg_name}");
    }

    update_description(root)?;

    // LICENSE if missing (required by License: MIT + file LICENSE).
    let license_path = root.join("LICENSE");
    if !license_path.is_file() {
        std::fs::write(&license_path, scaffold::license_content(&pkg_name))?;
    }

    // Minimal NAMESPACE if missing (~ use_miniextendr_namespace()): without
    // it the first build fails on the configure-time guard.
    let namespace_path = root.join("NAMESPACE");
    if !namespace_path.is_file() {
        std::fs::write(&namespace_path, scaffold::minimal_namespace(&pkg_name))?;
        if !quiet {
            eprintln!(
                "No NAMESPACE found; created a minimal one with useDynLib({pkg_name}, .registration = TRUE)"
            );
        }
    }

    let data = TemplateData::new(&pkg_name);
    scaffold::apply_plan(root, "templates/rpkg", RPKG_PLAN, &data, false)?;
    scaffold::write_config_scripts(root)?;
    std::fs::create_dir_all(root.join("vendor"))?;

    run_autoconf(root, quiet);
    write_miniextendr_yml(root, quiet)?;

    if !quiet {
        eprintln!("\nminiextendr scaffolding added.");
        eprintln!("Next: edit src/rust/lib.rs, then run: miniextendr workflow build");
    }
    Ok(())
}

// endregion

// region: shared scaffold steps

/// Write the full fresh standalone-package surface into `root`.
fn scaffold_rpkg_fresh(root: &Path, data: &TemplateData) -> Result<()> {
    scaffold_rpkg_at(root, "templates/rpkg", data)
}

/// Write the fresh R-package scaffold surface: the canonical DESCRIPTION
/// literal, LICENSE, NAMESPACE, the template plan, autoconf helper scripts,
/// and the `vendor/` directory (~ `create_rpkg_subdirectory()`).
fn scaffold_rpkg_at(root: &Path, prefix: &str, data: &TemplateData) -> Result<()> {
    scaffold::write_file(
        &root.join("DESCRIPTION"),
        &scaffold::description_content(&data.package),
        false,
    )?;
    scaffold::write_file(
        &root.join("LICENSE"),
        &scaffold::license_content(&data.package),
        false,
    )?;
    scaffold::write_file(
        &root.join("NAMESPACE"),
        &scaffold::minimal_namespace(&data.package),
        false,
    )?;
    scaffold::apply_plan(root, prefix, RPKG_PLAN, data, true)?;
    scaffold::write_config_scripts(root)?;
    std::fs::create_dir_all(root.join("vendor"))?;
    Ok(())
}

/// Merge the miniextendr DESCRIPTION fields into an existing file
/// (~ `use_miniextendr_description()`).
fn update_description(root: &Path) -> Result<()> {
    let desc_path = root.join("DESCRIPTION");
    let mut content = std::fs::read_to_string(&desc_path)?;

    for (field, value) in scaffold::CONFIG_BUILD_FIELDS {
        content = scaffold::desc_set_field(&content, field, value);
    }

    // Depends: merge the miniextendr R version floor (raise a lower declared
    // floor, keep a higher one) — this path runs against pre-existing
    // DESCRIPTIONs (~ mx_desc_ensure_r_floor()).
    content = scaffold::desc_ensure_r_floor(&content);

    // License: fill in when unset or still a usethis placeholder
    // ("`use_mit_license()`, `use_gpl3_license()` or friends ...").
    let license = parse_description_field(&content, "License").unwrap_or_default();
    if license.is_empty() || license.contains("_license()") {
        content = scaffold::desc_set_field(&content, "License", "MIT + file LICENSE");
    }

    // SystemRequirements: append the Rust toolchain requirement if absent.
    let sys_req = parse_description_field(&content, "SystemRequirements").unwrap_or_default();
    if !sys_req.to_lowercase().contains("rust") {
        let merged = if sys_req.is_empty() {
            RUST_SYSTEM_REQUIREMENT.to_string()
        } else {
            format!("{sys_req}, {RUST_SYSTEM_REQUIREMENT}")
        };
        content = scaffold::desc_set_field(&content, "SystemRequirements", &merged);
    }

    std::fs::write(&desc_path, content)?;
    Ok(())
}

/// Copy the default `miniextendr.yml` unless one exists
/// (~ `use_miniextendr_config()`).
fn write_miniextendr_yml(root: &Path, quiet: bool) -> Result<()> {
    let target = root.join("miniextendr.yml");
    if target.is_file() {
        if !quiet {
            eprintln!("miniextendr.yml already exists, skipping");
        }
        return Ok(());
    }
    scaffold::write_file(
        &target,
        scaffold::embedded("templates/miniextendr.yml"),
        false,
    )
}

/// Run autoconf when available and make `configure` executable
/// (~ `miniextendr_autoconf()`). Failure is a warning, not an error — the
/// user can install autoconf and rerun `miniextendr workflow autoconf`.
fn run_autoconf(dir: &Path, quiet: bool) {
    if !has_program("autoconf") {
        if !quiet {
            eprintln!(
                "autoconf not found; run `miniextendr workflow autoconf` after installing it"
            );
        }
        return;
    }
    match run_command("autoconf", &["-v", "-i", "-f"], dir, quiet) {
        Ok(_) => {
            let configure = dir.join("configure");
            #[cfg(unix)]
            if configure.is_file() {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(&configure, std::fs::Permissions::from_mode(0o755));
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("autoconf failed: {e:#}");
                eprintln!("Run `miniextendr workflow autoconf` manually later");
            }
        }
    }
}

// endregion

// region: validation helpers

fn package_name_from_path(root: &Path) -> Result<String> {
    root.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_string)
        .with_context(|| format!("could not derive a package name from {}", root.display()))
}

/// Validate an R package name (~ `create_miniextendr_package()`): ASCII
/// letters, digits, and dots; starts with a letter; does not end with a dot;
/// at least 2 characters.
fn validate_package_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    let valid = name.len() >= 2
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '.')
        && !name.ends_with('.');
    if !valid {
        let suggestion: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '.')
            .collect();
        bail!(
            "\"{name}\" is not a valid R package name.\n\
             R package names must start with a letter, contain only ASCII letters, digits,\n\
             and dots, and not end with a dot. Try: \"{suggestion}\" \
             (or pass --package for monorepos)"
        );
    }
    Ok(())
}

// endregion

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;
    use crate::scaffold::{Dest, EMBEDDED, Render, dest_rel};

    // region: test harness

    /// Fresh scratch directory per test (removed by `Scratch::drop`).
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "miniextendr-cli-init-{}-{}-{tag}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create scratch dir");
            Scratch(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Independent re-implementation of the substitution used to compute the
    /// EXPECTED file content from the on-disk template. Deliberately naive so
    /// a bug in `scaffold::render` cannot cancel itself out in the test.
    fn naive_substitute(template: &str, data: &[(&str, &str)], triple_only: bool) -> String {
        let mut out = template.to_string();
        for (key, value) in data {
            out = out.replace(&format!("{{{{{{{key}}}}}}}"), value);
            if !triple_only {
                out = out.replace(&format!("{{{{{key}}}}}"), value);
            }
        }
        out
    }

    /// Read a template from the repo checkout (NOT the embedded copy).
    fn disk_template(rel: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../minirextendr/inst")
            .join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn read(root: &Path, rel: &str) -> String {
        std::fs::read_to_string(root.join(rel))
            .unwrap_or_else(|e| panic!("read {rel} from scaffold: {e}"))
    }

    fn walk_scaffold(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable scaffold dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk_scaffold(&path, out);
            } else {
                out.push(path);
            }
        }
    }

    // endregion

    /// The #1351 regression test: the standalone scaffold's build-system
    /// surface must equal the canonical minirextendr templates modulo the
    /// documented substitutions, for every file in the plan — expected
    /// content is computed from the templates ON DISK, so both the embedded
    /// copies and the renderer are checked against the source of truth.
    #[test]
    fn package_scaffold_matches_canonical_templates() {
        let scratch = Scratch::new("pkg-surface");
        let root = scratch.path().join("my.pkg");
        let data = TemplateData::new("my.pkg");
        scaffold_rpkg_fresh(&root, &data).unwrap();

        let subs: Vec<(&str, &str)> = data.pairs().iter().map(|(k, v)| (*k, v.as_str())).collect();

        let mut problems = Vec::new();
        for entry in RPKG_PLAN {
            let dest = dest_rel(entry, &data).unwrap();
            let got = read(&root, &dest.to_string_lossy());
            let template = disk_template(&format!("templates/rpkg/{}", entry.template));
            let expected = match entry.render {
                Render::Mustache => naive_substitute(&template, &subs, false),
                Render::TripleOnly => naive_substitute(&template, &subs, true),
                Render::Verbatim => template,
                Render::IgnoreFilter => {
                    let patterns: Vec<&str> = template
                        .lines()
                        .filter(|l| !l.is_empty() && !l.starts_with('#'))
                        .collect();
                    patterns.join("\n") + "\n"
                }
            };
            if got != expected {
                problems.push(format!(
                    "{}: scaffolded content differs from canonical template {}",
                    dest.display(),
                    entry.template
                ));
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));

        // config.guess / config.sub come from the bundled scripts.
        for (src, dest) in scaffold::CONFIG_SCRIPTS {
            assert_eq!(read(&root, dest), disk_template(src), "{dest}");
        }
    }

    /// The obsolete four-mode build model must be gone from every scaffolded
    /// file: no PREPARE_CRAN / NOT_CRAN / BUILD_CONTEXT, no
    /// cargo-config.toml.in indirection, and no unresolved placeholders.
    #[test]
    fn no_obsolete_build_system_markers() {
        let scratch = Scratch::new("pkg-markers");
        let root = scratch.path().join("my.pkg");
        let data = TemplateData::new("my.pkg");
        scaffold_rpkg_fresh(&root, &data).unwrap();
        scaffold::write_file(
            &root.join("miniextendr.yml"),
            scaffold::embedded("templates/miniextendr.yml"),
            false,
        )
        .unwrap();

        let mut files = Vec::new();
        walk_scaffold(&root, &mut files);
        assert!(files.len() > 20, "scaffold unexpectedly small: {files:?}");

        let mut problems = Vec::new();
        for file in &files {
            let name = file.file_name().unwrap().to_string_lossy();
            if name == "cargo-config.toml.in" {
                problems.push(format!("{}: retired file scaffolded", file.display()));
            }
            let content = std::fs::read_to_string(file).expect("scaffolded files are UTF-8");
            for marker in ["PREPARE_CRAN", "NOT_CRAN", "BUILD_CONTEXT"] {
                if content.contains(marker) {
                    problems.push(format!("{}: contains retired {marker}", file.display()));
                }
            }
            for placeholder in ["{{package", "{{crate_name", "{{rpkg_name", "{{year"] {
                if content.contains(placeholder) {
                    problems.push(format!(
                        "{}: unresolved placeholder {placeholder}",
                        file.display()
                    ));
                }
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));

        // The canonical two-mode latch and its consumers must be present.
        let configure_ac = read(&root, "configure.ac");
        assert!(
            configure_ac.contains("inst/vendor.tar.xz"),
            "install-mode latch missing"
        );
        assert!(
            configure_ac.starts_with("AC_INIT([my.pkg]"),
            "package name not substituted"
        );
        let cargo_toml = read(&root, "src/rust/Cargo.toml");
        assert!(cargo_toml.contains("name = \"my_pkg\""));
        assert!(
            cargo_toml.contains("miniextendr-api = { git ="),
            "source mode must declare the git dependency"
        );
        assert!(
            !cargo_toml.contains("../../vendor/miniextendr-api"),
            "retired hardcoded vendor path dependency"
        );
        let desc = read(&root, "DESCRIPTION");
        for (field, value) in scaffold::CONFIG_BUILD_FIELDS {
            assert!(
                desc.contains(&format!("{field}: {value}")),
                "{field} missing"
            );
        }
        assert!(desc.contains(&format!("SystemRequirements: {RUST_SYSTEM_REQUIREMENT}")));
        assert!(
            desc.contains(&format!("Depends: {}\n", scaffold::r_depends_entry())),
            "fresh DESCRIPTION missing the R version floor: {desc}"
        );
        assert!(read(&root, "NAMESPACE").contains("useDynLib(my.pkg, .registration = TRUE)"));
        assert!(root.join("vendor").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn build_hooks_are_executable() {
        use std::os::unix::fs::PermissionsExt;
        let scratch = Scratch::new("pkg-exec");
        let root = scratch.path().join("my.pkg");
        scaffold_rpkg_fresh(&root, &TemplateData::new("my.pkg")).unwrap();
        for rel in [
            "cleanup",
            "cleanup.win",
            "cleanup.ucrt",
            "configure.win",
            "configure.ucrt",
            "tools/config.guess",
            "tools/config.sub",
        ] {
            let mode = std::fs::metadata(root.join(rel))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(
                mode & 0o111,
                0o111,
                "{rel} must be executable (mode {mode:o})"
            );
        }
    }

    #[test]
    fn monorepo_scaffold_matches_canonical_templates() {
        let scratch = Scratch::new("monorepo");
        let root = scratch.path().join("proj");
        init_monorepo(
            &root.to_string_lossy(),
            Some("my.proj"),
            None, // crate_name defaults to my-proj
            None, // rpkg_name defaults to my.proj
            true,
        )
        .unwrap();

        let data = TemplateData::new("my.proj")
            .with_crate("my-proj")
            .with_rpkg("my.proj");
        let subs: Vec<(&str, &str)> = data.pairs().iter().map(|(k, v)| (*k, v.as_str())).collect();

        // Workspace root surface vs disk templates.
        for entry in MONOREPO_ROOT_PLAN {
            let dest = dest_rel(entry, &data).unwrap();
            let got = read(&root, &dest.to_string_lossy());
            let template = disk_template(&format!("templates/monorepo/{}", entry.template));
            let expected = match entry.render {
                Render::TripleOnly => naive_substitute(&template, &subs, true),
                _ => naive_substitute(&template, &subs, false),
            };
            assert_eq!(got, expected, "{} differs from canonical", dest.display());
        }

        // Embedded R package uses the monorepo/rpkg templates (sibling path
        // dependency present, monorepo-flavored configure.ac).
        let rpkg_cargo = read(&root, "my.proj/src/rust/Cargo.toml");
        assert!(rpkg_cargo.contains("my-proj = { path = \"../../../my-proj\" }"));
        let configure_ac = read(&root, "my.proj/configure.ac");
        assert_eq!(
            configure_ac,
            naive_substitute(
                &disk_template("templates/monorepo/rpkg/configure.ac"),
                &subs,
                false
            )
        );
        assert_eq!(
            read(&root, "my.proj/src/Makevars.win"),
            disk_template("templates/monorepo/rpkg/Makevars.win")
        );
        // bump-version.R keeps copy_template() semantics: {{{rpkg_name}}}
        // substituted, nothing else touched, no placeholder residue.
        let bump = read(&root, "tools/bump-version.R");
        assert!(bump.contains("tools/bump-version.R my.proj"));
        assert!(!bump.contains("{{{"));
        // create_miniextendr_monorepo() does not write miniextendr.yml.
        assert!(!root.join("miniextendr.yml").exists());
    }

    #[test]
    fn monorepo_rejects_colliding_names() {
        let scratch = Scratch::new("monorepo-collide");
        let root = scratch.path().join("proj");
        let err = init_monorepo(
            &root.to_string_lossy(),
            Some("mypkg"),
            Some("mypkg"),
            Some("mypkg"),
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("must be different"), "{err}");
    }

    #[test]
    fn init_use_merges_into_existing_package() {
        let scratch = Scratch::new("use-existing");
        let root = scratch.path().to_path_buf();
        std::fs::write(
            root.join("DESCRIPTION"),
            "Package: existing.pkg\n\
             Title: Existing\n\
             Version: 1.0.0\n\
             License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a license\n\
             SystemRequirements: GNU make\n\
             Encoding: UTF-8\n",
        )
        .unwrap();
        std::fs::write(root.join("NAMESPACE"), "export(existing_fn)\n").unwrap();
        std::fs::write(root.join(".Rbuildignore"), "^custom$\n").unwrap();

        init_use_rpkg(&root, true).unwrap();

        let desc = read(&root, "DESCRIPTION");
        assert!(desc.contains("Package: existing.pkg"));
        assert!(desc.contains("Version: 1.0.0"), "existing fields preserved");
        for (field, value) in scaffold::CONFIG_BUILD_FIELDS {
            assert!(desc.contains(&format!("{field}: {value}")), "{field}");
        }
        assert!(
            desc.contains("License: MIT + file LICENSE"),
            "placeholder replaced"
        );
        assert!(
            desc.contains(&format!(
                "SystemRequirements: GNU make, {RUST_SYSTEM_REQUIREMENT}"
            )),
            "Rust appended to existing SystemRequirements: {desc}"
        );
        assert!(
            desc.contains(&format!("Depends: {}\n", scaffold::r_depends_entry())),
            "R version floor added to a DESCRIPTION without Depends: {desc}"
        );

        // Existing NAMESPACE untouched; ignore file merged with dedupe.
        assert_eq!(read(&root, "NAMESPACE"), "export(existing_fn)\n");
        let rbuildignore = read(&root, ".Rbuildignore");
        assert!(
            rbuildignore.starts_with("^custom$\n"),
            "existing patterns kept"
        );
        assert!(rbuildignore.contains("^src/rust/target$"));
        // Idempotence: rerunning must not duplicate patterns.
        init_use_rpkg(&root, true).unwrap();
        let again = read(&root, ".Rbuildignore");
        assert_eq!(again.matches("^src/rust/target$").count(), 1);

        // Build-system files landed and are canonical.
        assert!(read(&root, "configure.ac").starts_with("AC_INIT([existing.pkg]"));
        assert!(root.join("src/rust/lib.rs").is_file());
        assert!(root.join("tools/lock-shape-check.R").is_file());
        assert!(root.join("miniextendr.yml").is_file());
    }

    /// `init use` must never lower an existing R floor — merge semantics are
    /// unit-tested on `desc_ensure_r_floor`; this checks the wiring end to end.
    #[test]
    fn init_use_keeps_higher_r_floor() {
        let scratch = Scratch::new("use-higher-floor");
        let root = scratch.path().to_path_buf();
        std::fs::write(
            root.join("DESCRIPTION"),
            "Package: floor.pkg\n\
             Version: 1.0.0\n\
             Depends: R (>= 4.6), methods\n\
             License: MIT + file LICENSE\n",
        )
        .unwrap();

        init_use_rpkg(&root, true).unwrap();

        let desc = read(&root, "DESCRIPTION");
        assert!(
            desc.contains("Depends: R (>= 4.6), methods\n"),
            "existing higher floor must survive init use: {desc}"
        );
    }

    #[test]
    fn init_use_detects_monorepo_from_cargo_toml() {
        let scratch = Scratch::new("use-monorepo");
        let root = scratch.path().to_path_buf();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"my-core\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();

        init_use_monorepo(&root, None, true).unwrap();

        // Package name derived from the crate name (dashes -> dots), R
        // package scaffolded into a sibling subdirectory.
        let desc = read(&root, "my.core/DESCRIPTION");
        assert!(desc.contains("Package: my.core"));
        let cargo = read(&root, "my.core/src/rust/Cargo.toml");
        assert!(cargo.contains("my-core = { path = \"../../../my-core\" }"));
        assert!(root.join("miniextendr.yml").is_file());
    }

    #[test]
    fn validate_package_name_mirrors_r() {
        for good in ["my.pkg", "abc", "a2", "A.b.c"] {
            assert!(validate_package_name(good).is_ok(), "{good}");
        }
        for bad in ["a", "1pkg", "my-pkg", "pkg.", "my_pkg", ".pkg"] {
            assert!(validate_package_name(bad).is_err(), "{bad}");
        }
    }

    /// Guard: every plan template resolves inside the embedded manifest for
    /// both template-type prefixes, and destinations are well-formed.
    #[test]
    fn plans_are_consistent_with_manifest() {
        for (prefix, plan) in [
            ("templates/rpkg", RPKG_PLAN),
            ("templates/monorepo/rpkg", RPKG_PLAN),
            ("templates/monorepo", MONOREPO_ROOT_PLAN),
        ] {
            for entry in plan {
                let rel = format!("{prefix}/{}", entry.template);
                assert!(
                    EMBEDDED.iter().any(|(p, _)| *p == rel),
                    "plan references unembedded template {rel}"
                );
                if let Dest::Path(p) = entry.dest {
                    assert!(!p.starts_with('/'), "absolute dest {p}");
                }
            }
        }
    }
}
