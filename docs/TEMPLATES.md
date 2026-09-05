# Template System

The minirextendr package provides scaffolding templates for creating new R packages with Rust backends. This document explains how the template system works and how to keep templates in sync with the reference implementation.

## Overview

### Template Sources

Templates are stored in `minirextendr/inst/templates/` and come in two flavors:

- **`rpkg/`** - Standalone R package template (recommended for most users)
- **`monorepo/`** - Workspace-based template with separate Rust crate

### Master Source: example package (`rpkg/`)

**Important:** The templates are **derived from** the example package (`rpkg/`), not the other way around.

- The example package (`rpkg/`) is the **master source** where changes should be tested first
- Templates are **copies** of the example package files with additional logic for standalone use
- Changes flow: `rpkg/` → test → apply to templates → approve delta

## Template Sync System

### Approved Delta

Templates are not exact copies of the example package (`rpkg/`) - they have legitimate differences for standalone projects:

- **Conditional monorepo detection** - Check if miniextendr-api exists before using path overrides
- **Standalone tarball production** - Bootstrap can freeze path-dependency siblings into the release vendor tarball
- **Extra flexibility** - Handle cases where rpkg assumptions don't hold

The approved differences are tracked in `patches/templates.patch`.

### Justfile Commands

```bash
# Check if templates match rpkg (with approved delta)
just templates-check

# Approve current delta as the new baseline
just templates-approve
```

## Workflow for Template Changes

### 1. Making Changes to Templates

**Option A: Change applies to rpkg too (most common)**

```bash
# 1. Make change in rpkg/ first (the master source)
vim rpkg/configure.ac

# 2. Test the change in rpkg
just configure
just devtools-test

# 3. Apply the same change to templates
vim minirextendr/inst/templates/rpkg/configure.ac
vim minirextendr/inst/templates/monorepo/rpkg/configure.ac

# 4. Approve the updated delta
just templates-approve

# 5. Verify sync
just templates-check  # Should pass
```

**Option B: Template-only change (rare)**

If the change only makes sense for templates (e.g., adding fallback logic for missing monorepo):

```bash
# 1. Make change in templates only
vim minirextendr/inst/templates/rpkg/configure.ac

# 2. Approve the new delta
just templates-approve

# 3. Verify
just templates-check
```

### 2. When templates-check Fails

If [`just templates-check`](https://github.com/A2-ai/miniextendr/blob/main/justfile) fails, it means templates have drifted from the example package (`rpkg/`):

```bash
# See what changed
just templates-check  # Shows diff output

# Options:
# A) The drift is intentional (you made changes) - approve it
just templates-approve

# B) The drift is accidental - revert template changes
git restore minirextendr/inst/templates/
```

### 3. Common Scenarios

**Scenario: Updated configure.ac in the example package**

```bash
# Edit and test in rpkg
vim rpkg/configure.ac
cd rpkg && autoconf && bash ./configure

# Copy changes to templates
vim minirextendr/inst/templates/rpkg/configure.ac
vim minirextendr/inst/templates/monorepo/rpkg/configure.ac

# Approve
just templates-approve
```

**Scenario: Updated .Rbuildignore patterns**

```bash
# Edit master
vim rpkg/.Rbuildignore

# Copy to templates
vim minirextendr/inst/templates/rpkg/Rbuildignore
vim minirextendr/inst/templates/monorepo/rpkg/Rbuildignore

# Approve
just templates-approve
```

**Scenario: Updated bootstrap.R**

```bash
# Edit and test
vim rpkg/bootstrap.R
R CMD INSTALL rpkg  # Tests bootstrap

# Copy to templates
vim minirextendr/inst/templates/rpkg/bootstrap.R
vim minirextendr/inst/templates/monorepo/rpkg/bootstrap.R

# Approve
just templates-approve
```

## Template Files

### Files That Need Sync

The following files in rpkg/ have corresponding template versions:

| rpkg/ | Template Location |
|-------|-------------------|
| `.Rbuildignore` | `inst/templates/*/Rbuildignore` |
| `bootstrap.R` | `inst/templates/*/bootstrap.R` |
| `build.rs` | `inst/templates/*/build.rs` |
| `cleanup`, `cleanup.win`, `cleanup.ucrt` | `inst/templates/*/cleanup*` |
| `configure.ac` | `inst/templates/*/configure.ac` |
| `configure.win`, `configure.ucrt` | `inst/templates/*/configure.*` |
| `src/stub.c` | `inst/templates/*/stub.c` |
| `src/Makevars.in` | `inst/templates/*/Makevars.in` |
| `src/rust/build.rs` | `inst/templates/*/build.rs` |
| `src/rust/Cargo.toml` | `inst/templates/*/Cargo.toml` |
| `src/rust/lib.rs` | `inst/templates/*/lib.rs` (with `{{package_rs}}` substitution) |
| `R/rpkg-package.R` | `inst/templates/*/package.R` |
| `inst/include/mx_abi.h` | `inst/templates/*/inst_include/mx_abi.h` |

`src/rust/.cargo/config.toml` is not copied from a template. Each
`configure.ac` writes it directly in its `AC_CONFIG_COMMANDS([cargo-config],
...)` block so the contents can reflect the detected install mode.

### Files That Don't Need Sync

These `rpkg/` files are specific to the example package and don't have template equivalents:

- `DESCRIPTION` - Package metadata
- `R/*.R` - R function implementations
- `src/rust/*.rs` - Example Rust code (except lib.rs template)
- `tests/` - Test files
- `man/` - Generated documentation

## Implementation Details

### templates-check Recipe

The check recipe:

1. Copies rpkg files to a temp directory
2. Applies `patches/templates.patch` to reverse approved differences
3. Compares with actual templates
4. Fails if any unexpected differences found

### templates-approve Recipe

The approve recipe:

1. Copies rpkg files to a temp directory
2. Compares with actual templates
3. Generates a new `patches/templates.patch` with current delta
4. This becomes the new approved baseline

### Patch File

`patches/templates.patch` stores the approved differences between the example package and templates as a unified diff. This allows:

- Intentional differences to be tracked and reviewed
- Unexpected drift to be caught by `templates-check`
- Clear documentation of why templates differ from rpkg

## Best Practices

### Always Test in the example package (`rpkg/`) first

1. ✅ Make change in rpkg/
2. ✅ Test thoroughly (just devtools-test)
3. ✅ Apply to templates
4. ✅ Approve delta
5. ❌ Don't change templates without testing in rpkg first

### Keep Templates Simple

Templates should be as close to rpkg as possible. Only add template-specific logic when absolutely necessary for standalone projects.

### Document Intentional Differences

When adding template-specific logic, add comments explaining why it differs from rpkg:

```r
# bootstrap.R: this runs only while a build frontend is producing a tarball.
# configure.ac never creates inst/vendor.tar.xz.
system2("cargo", c("revendor", "--freeze", "--compress", "inst/vendor.tar.xz"))
```

### Run templates-check Before Committing

Always verify templates are in sync before committing:

```bash
# Your workflow
git add rpkg/configure.ac
git add minirextendr/inst/templates/
git add patches/templates.patch

# Verify
just templates-check  # Must pass before commit
```

## Troubleshooting

### templates-check fails after modifying the example package

**Expected behavior.** You modified the master source, so templates are now out of sync.

**Solution:**
```bash
# Apply changes to templates, then approve
just templates-approve
```

### templates-check fails but I didn't change anything

**Possible causes:**
- Someone committed example package changes without updating templates
- The patch file is out of sync

**Solution:**
```bash
# Review the diff to understand what changed
just templates-check  # Shows differences

# If changes should be in templates, update them
# If rpkg is wrong, revert it
# Then approve
just templates-approve
```

### I modified templates but not the example package

**Generally wrong approach.** Templates derive from the example package (`rpkg/`).

**Solution:**
```bash
# 1. Apply the change to rpkg first
vim rpkg/configure.ac

# 2. Test in rpkg
just devtools-test

# 3. Update templates
vim minirextendr/inst/templates/rpkg/configure.ac

# 4. Approve
just templates-approve
```

## Examples

### Example: Add New .Rbuildignore Pattern

```bash
# 1. Add to master
echo "^new-pattern$" >> rpkg/.Rbuildignore

# 2. Test
R CMD build rpkg
R CMD check rpkg_*.tar.gz

# 3. Add to templates
echo "^new-pattern$" >> minirextendr/inst/templates/rpkg/Rbuildignore
echo "^new-pattern$" >> minirextendr/inst/templates/monorepo/rpkg/Rbuildignore

# 4. Approve and verify
just templates-approve
just templates-check
```

### Example: Fix configure.ac Bug

```bash
# 1. Fix in rpkg
vim rpkg/configure.ac
cd rpkg && autoconf  # Regenerate configure

# 2. Test the fix
just configure
just devtools-test

# 3. Apply to templates
vim minirextendr/inst/templates/rpkg/configure.ac
vim minirextendr/inst/templates/monorepo/rpkg/configure.ac

# 4. Approve
just templates-approve

# 5. Verify everything
just templates-check
just minirextendr-test  # Template tests include scaffolding
```

### Example: Update bootstrap.R Logic

```bash
# 1. Edit master
vim rpkg/bootstrap.R

# 2. Test by triggering bootstrap
rm rpkg/src/Makevars  # Force bootstrap to run
R CMD INSTALL rpkg

# 3. Copy to templates
cp rpkg/bootstrap.R minirextendr/inst/templates/rpkg/bootstrap.R
cp rpkg/bootstrap.R minirextendr/inst/templates/monorepo/rpkg/bootstrap.R

# 4. Approve
just templates-approve
```

## CI Integration

The template sync check runs in CI via minirextendr tests:

```r
test_that("templates patch is in sync with rpkg sources", {
  # Runs `just templates-check` from repo root
  # Fails if templates have drifted from rpkg
})
```

This ensures:
- Templates don't drift over time
- Changes to rpkg are reflected in templates
- Patch file stays up to date

## Summary

**Key Points:**
1. The example package (`rpkg/`) is the master source
2. Templates are derived copies with minimal additions
3. `patches/templates.patch` tracks approved differences
4. Always test in the example package before updating templates
5. Run `just templates-check` before committing
6. Use `just templates-approve` to update the approved delta

**Commands:**
```bash
just templates-check    # Verify templates match rpkg (with approved delta)
just templates-approve  # Approve current delta as new baseline
```
