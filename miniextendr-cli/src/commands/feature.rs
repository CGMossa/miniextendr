use std::collections::BTreeMap;

use anyhow::{Result, bail};

use crate::bridge::{rscript_eval, rscript_eval_args, run_command};
use crate::cli::{FeatureCmd, FeatureDetectCmd, FeatureRuleCmd};
use crate::output::print_json;
use crate::project::ProjectContext;

pub fn dispatch(cmd: &FeatureCmd, ctx: &ProjectContext, quiet: bool, json: bool) -> Result<()> {
    match cmd {
        FeatureCmd::Enable { name } => feature_enable(ctx, name, quiet),
        FeatureCmd::List => feature_list(ctx, json),
        FeatureCmd::Detect { cmd: detect_cmd } => match detect_cmd {
            FeatureDetectCmd::Init => feature_detect_init(ctx, quiet),
            FeatureDetectCmd::Update => feature_detect_update(ctx, quiet),
        },
        FeatureCmd::Rule { cmd: rule_cmd } => match rule_cmd {
            FeatureRuleCmd::Add {
                feature,
                detect,
                cargo_spec,
                optional_dep,
            } => feature_rule_add(
                ctx,
                feature,
                detect,
                cargo_spec.as_deref(),
                *optional_dep,
                quiet,
            ),
            FeatureRuleCmd::Remove { feature } => feature_rule_remove(ctx, feature, quiet),
            FeatureRuleCmd::List => feature_rule_list(ctx, json),
        },
    }
}

/// Enable a named feature by adding the cargo dependency/feature.
fn feature_enable(ctx: &ProjectContext, name: &str, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let manifest_str = manifest.to_string_lossy().to_string();

    match name {
        "r6" => {
            // R6 is an R-side class system, just need to suggest R6 in DESCRIPTION
            add_r_suggests(ctx, "R6", quiet)?;
            if !quiet {
                println!("Enabled R6 class system. Add `#[miniextendr(r6)]` to impl blocks.");
            }
        }
        "s4" => {
            add_r_depends(ctx, "methods", quiet)?;
            if !quiet {
                println!("Enabled S4 class system. Add `#[miniextendr(s4)]` to impl blocks.");
            }
        }
        "s7" => {
            add_r_suggests(ctx, "S7", quiet)?;
            if !quiet {
                println!("Enabled S7 class system. Add `#[miniextendr(s7)]` to impl blocks.");
            }
        }
        "serde" => {
            run_command(
                "cargo",
                &[
                    "add",
                    "--manifest-path",
                    &manifest_str,
                    "serde",
                    "--features",
                    "derive",
                ],
                &ctx.root,
                quiet,
            )?;
            // Enable miniextendr-api serde feature
            enable_cargo_feature(ctx, "serde", quiet)?;
            if !quiet {
                println!("Enabled serde. Use `#[derive(Serialize, Deserialize)]` on structs.");
            }
        }
        "vctrs" => {
            add_r_suggests(ctx, "vctrs", quiet)?;
            enable_cargo_feature(ctx, "vctrs", quiet)?;
            if !quiet {
                println!("Enabled vctrs integration.");
            }
        }
        "rayon" => {
            run_command(
                "cargo",
                &["add", "--manifest-path", &manifest_str, "rayon"],
                &ctx.root,
                quiet,
            )?;
            enable_cargo_feature(ctx, "rayon", quiet)?;
            if !quiet {
                println!("Enabled rayon parallelism.");
            }
        }
        "build-rs" => {
            let build_rs = ctx.root.join("src/rust/build.rs");
            if !build_rs.exists() {
                std::fs::write(
                    &build_rs,
                    "fn main() {\n    println!(\"cargo::rerun-if-changed=lib.rs\");\n}\n",
                )?;
            }
            if !quiet {
                println!("Created src/rust/build.rs");
            }
        }
        "feature-detection" => feature_detect_init(ctx, quiet)?,
        "knitr" | "rmarkdown" | "quarto" => {
            if !quiet {
                println!(
                    "Feature '{name}' requires R-side setup.\n\
                     Use `Rscript -e 'minirextendr::use_miniextendr_{name}()'` or set up manually."
                );
            }
        }
        other => {
            // Try as a cargo feature name
            enable_cargo_feature(ctx, other, quiet)?;
        }
    }
    Ok(())
}

/// List cargo features from Cargo.toml.
fn feature_list(ctx: &ProjectContext, json: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let content = std::fs::read_to_string(manifest)?;
    let toml: toml::Value = content.parse()?;

    let features = toml
        .get("features")
        .and_then(|f| f.as_table())
        .cloned()
        .unwrap_or_default();

    // Also gather optional deps
    let optional_deps: Vec<String> = toml
        .get("dependencies")
        .and_then(|d| d.as_table())
        .map(|deps| {
            deps.iter()
                .filter(|(_, v)| v.get("optional").and_then(|o| o.as_bool()).unwrap_or(false))
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default();

    if json {
        let mut map = serde_json::Map::new();
        for (name, deps) in &features {
            let deps_vec: Vec<String> = deps
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            map.insert(name.clone(), serde_json::json!(deps_vec));
        }
        if !optional_deps.is_empty() {
            map.insert("_optional_deps".into(), serde_json::json!(optional_deps));
        }
        print_json(&map)?;
    } else {
        println!("Cargo features:");
        for (name, deps) in &features {
            let deps_str = deps
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            if deps_str.is_empty() {
                println!("  {name}");
            } else {
                println!("  {name} = [{deps_str}]");
            }
        }
        if !optional_deps.is_empty() {
            println!("\nOptional dependencies:");
            for dep in &optional_deps {
                println!("  {dep}");
            }
        }
    }
    Ok(())
}

/// Set up the canonical configure-time script and regenerate configure.
fn feature_detect_init(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    rscript_eval(
        "minirextendr::use_configure_feature_detection()",
        &ctx.root,
        quiet,
    )?;
    Ok(())
}

/// Update feature detection by calling the R helper.
fn feature_detect_update(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    rscript_eval("minirextendr::update_feature_detection()", &ctx.root, quiet)?;
    if !quiet {
        println!("Updated feature detection helpers.");
    }
    Ok(())
}

/// Add a rule through the R helper, including its Cargo dependency options.
fn feature_rule_add(
    ctx: &ProjectContext,
    feature: &str,
    detect: &str,
    cargo_spec: Option<&str>,
    optional_dep: bool,
    quiet: bool,
) -> Result<()> {
    let detect_file = ctx.root.join("tools/detect-features.R");
    if !detect_file.exists() {
        feature_detect_init(ctx, quiet)?;
    }
    parse_feature_rules(&std::fs::read_to_string(&detect_file)?)?;

    let optional_dep = if optional_dep { "TRUE" } else { "FALSE" };
    rscript_eval_args(
        "args <- commandArgs(trailingOnly = TRUE); \
         minirextendr::add_feature_rule(args[[1]], detect = args[[2]], \
         cargo_spec = if (nzchar(args[[3]])) args[[3]] else NULL, \
         optional_dep = as.logical(args[[4]]))",
        &[
            feature,
            detect,
            cargo_spec.unwrap_or_default(),
            optional_dep,
        ],
        &ctx.root,
        quiet,
    )?;
    Ok(())
}

/// Remove a rule through the same marker-aware editor used from R.
fn feature_rule_remove(ctx: &ProjectContext, feature: &str, quiet: bool) -> Result<()> {
    let detect_file = ctx.root.join("tools/detect-features.R");
    if !detect_file.exists() {
        bail!("tools/detect-features.R not found. Run `miniextendr feature detect init` first.");
    }
    parse_feature_rules(&std::fs::read_to_string(&detect_file)?)?;
    rscript_eval_args(
        "minirextendr::remove_feature_rule(commandArgs(trailingOnly = TRUE)[[1]])",
        &[feature],
        &ctx.root,
        quiet,
    )?;
    Ok(())
}

/// Read the single-line rules written by minirextendr without evaluating R.
fn parse_feature_rules(content: &str) -> Result<BTreeMap<String, String>> {
    let mut lines = content.lines();
    if !lines.any(|line| line.starts_with("## BEGIN RULES")) {
        bail!(
            "tools/detect-features.R has no BEGIN RULES marker; replace the legacy script and run `miniextendr feature detect init`."
        );
    }
    let mut rules = BTreeMap::new();
    for line in lines {
        if line.starts_with("## END RULES") {
            return Ok(rules);
        }
        if let Some(rule) = line.strip_prefix("rules[[\"")
            && let Some((feature, detect)) = rule.split_once("\"]] <- function() ")
        {
            rules.insert(feature.to_owned(), detect.to_owned());
        }
    }
    bail!(
        "tools/detect-features.R has no END RULES marker; restore the rules section before editing it."
    )
}

/// List canonical rules; JSON is an object mapping feature names to expressions.
fn feature_rule_list(ctx: &ProjectContext, json: bool) -> Result<()> {
    let detect_file = ctx.root.join("tools/detect-features.R");
    let rules = if detect_file.exists() {
        parse_feature_rules(&std::fs::read_to_string(&detect_file)?)?
    } else {
        BTreeMap::new()
    };
    if json {
        print_json(&rules)?;
    } else if rules.is_empty() {
        println!("No feature detection rules defined.");
    } else {
        println!("Feature detection rules:");
        for (feature, detect) in rules {
            println!("  {feature}: {detect}");
        }
    }
    Ok(())
}

// region: Helpers

/// Enable a feature in the `[features]` section by adding it to default or as standalone.
fn enable_cargo_feature(ctx: &ProjectContext, feature: &str, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let content = std::fs::read_to_string(manifest)?;
    let mut toml: toml::Value = content.parse()?;

    // Check if feature already exists
    let has_feature = toml
        .get("features")
        .and_then(|f| f.as_table())
        .is_some_and(|t| t.contains_key(feature));

    if has_feature {
        if !quiet {
            println!("Feature '{feature}' already defined in Cargo.toml");
        }
        return Ok(());
    }

    // Add feature if it maps to miniextendr-api
    let feature_def = format!("miniextendr-api/{feature}");
    if let Some(features) = toml.get_mut("features").and_then(|f| f.as_table_mut()) {
        features.insert(
            feature.to_string(),
            toml::Value::Array(vec![toml::Value::String(feature_def)]),
        );
    }

    std::fs::write(manifest, toml.to_string())?;
    if !quiet {
        println!("Added feature '{feature}' to Cargo.toml");
    }
    Ok(())
}

/// Add a package to Suggests in DESCRIPTION.
fn add_r_suggests(ctx: &ProjectContext, pkg: &str, quiet: bool) -> Result<()> {
    add_desc_field(ctx, "Suggests", pkg, quiet)
}

/// Add a package to Depends in DESCRIPTION.
fn add_r_depends(ctx: &ProjectContext, pkg: &str, quiet: bool) -> Result<()> {
    add_desc_field(ctx, "Depends", pkg, quiet)
}

fn add_desc_field(ctx: &ProjectContext, field: &str, pkg: &str, quiet: bool) -> Result<()> {
    let desc_path = ctx.root.join("DESCRIPTION");
    if !desc_path.exists() {
        return Ok(());
    }

    // Check if field exists and if pkg is already listed (DCF-aware, so a
    // value wrapped across continuation lines is still searched in full).
    let existing = ctx.description_field(field);
    if let Some(value) = &existing
        && value.contains(pkg)
    {
        if !quiet {
            println!("{pkg} already in {field}");
        }
        return Ok(());
    }

    let content = std::fs::read_to_string(&desc_path)?;
    let prefix = format!("{field}:");
    let new_content = if existing.is_some() {
        // Append to existing field
        content.replace(&prefix.to_string(), &format!("{prefix} {pkg},"))
    } else {
        // Add new field
        format!("{content}{field}: {pkg}\n")
    };

    std::fs::write(&desc_path, new_content)?;
    if !quiet {
        println!("Added {pkg} to {field} in DESCRIPTION");
    }
    Ok(())
}

// endregion

#[cfg(test)]
mod tests {
    use super::parse_feature_rules;

    #[test]
    fn canonical_rules_are_scoped_and_keep_detection_expressions() {
        let content = r#"rules[["outside"]] <- function() FALSE
## BEGIN RULES (do not edit this line)
rules[["alpha"]] <- function() requireNamespace("pkg", quietly = TRUE)
rules[["beta"]] <- function() FALSE
## END RULES (do not edit this line)
rules[["after"]] <- function() TRUE
"#;
        let rules = parse_feature_rules(content).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules["alpha"], r#"requireNamespace("pkg", quietly = TRUE)"#);
        assert_eq!(rules["beta"], "FALSE");
    }

    #[test]
    fn legacy_or_incomplete_rules_are_rejected() {
        assert!(parse_feature_rules("detect_features <- function() character()").is_err());
        assert!(parse_feature_rules("## BEGIN RULES\n").is_err());
        assert!(
            parse_feature_rules("## BEGIN RULES\n## END RULES\n")
                .unwrap()
                .is_empty()
        );
    }
}
