# Tests for the stale-export drift check in miniextendr_doctor() (#1304/#1305).
#
# miniextendr_build() self-heals a NAMESPACE that still lists an export whose
# backing #[miniextendr] function was removed or renamed (#1288), but
# pkgload::load_all() / testthat only WARN on a superset NAMESPACE and
# proceed, so a load_all()-only dev loop carries the drift green all the way
# to R CMD check / library(). doctor must flag explicit export() directives
# with no backing definition in the union of (a) top-level objects defined in
# R/ sources and (b) the generated R/<pkg>-wrappers.R -- statically, without
# loading or executing any package code -- and advise miniextendr_build().

# Build a minimal fake R-package directory suitable for doctor(). `r_files`
# is a named list: name = file name under R/, value = character vector of
# source lines. The generated-wrappers file is just another entry (e.g.
# "testpkg-wrappers.R") -- the check keys off the *-wrappers.R name only for
# its has-the-package-been-built-yet gate.
make_stale_export_pkg <- function(ns_lines, r_files = list()) {
  tmp <- tempfile("doctor-stale-exports-")
  dir.create(tmp)
  writeLines(
    c("Package: testpkg", "Title: Test", "Version: 0.1.0", ""),
    file.path(tmp, "DESCRIPTION")
  )
  writeLines(ns_lines, file.path(tmp, "NAMESPACE"))

  rust_dir <- file.path(tmp, "src", "rust")
  dir.create(rust_dir, recursive = TRUE)
  writeLines(
    c("[package]", 'name = "testpkg"', "", "[dependencies]", 'miniextendr-api = "*"'),
    file.path(rust_dir, "Cargo.toml")
  )

  r_dir <- file.path(tmp, "R")
  dir.create(r_dir)
  for (name in names(r_files)) {
    writeLines(r_files[[name]], file.path(r_dir, name))
  }

  tmp
}

ns_header <- "useDynLib(testpkg, .registration = TRUE)"

test_that("doctor flags a NAMESPACE export with no backing definition and advises miniextendr_build()", {
  pkg <- make_stale_export_pkg(
    ns_lines = c(ns_header, "export(real_fn)", "export(wrapper_fn)", "export(gone_fn)"),
    r_files = list(
      "code.R" = "real_fn <- function() 1",
      "testpkg-wrappers.R" = 'wrapper_fn <- function() .Call("C_wrapper_fn")'
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  msgs <- testthat::capture_messages(
    result <- miniextendr_doctor(pkg)
  )

  # Exactly the removed function is flagged -- backed exports are not.
  expect_identical(
    grep("^stale NAMESPACE export: ", result$warn, value = TRUE),
    "stale NAMESPACE export: gone_fn"
  )
  # Warn, not fail: a static scan cannot see dynamic definitions.
  expect_false(any(grepl("gone_fn", result$fail, fixed = TRUE)))

  # The console output names the offender and the remediation. cli wraps at
  # the console width, so collapse and normalise whitespace before matching.
  flat <- gsub("\\s+", " ", paste(msgs, collapse = " "))
  expect_match(flat, "export(gone_fn)", fixed = TRUE)
  expect_match(flat, "miniextendr_build()", fixed = TRUE)
})

test_that("doctor passes a clean package, whatever the top-level definition form", {
  pkg <- make_stale_export_pkg(
    ns_lines = c(
      ns_header,
      "export(real_fn)",
      'export("%+%")',
      'export("foo<-")',
      "export(obj)",
      "export(cond_fn)",
      "export(assigned_fn)",
      "export(gen_fn)",
      "export(wrapper_fn)"
    ),
    r_files = list(
      "code.R" = c(
        "real_fn = function() 1",
        "`%+%` <- function(a, b) a + b",
        "`foo<-` <- function(x, value) x",
        "obj <- 1:10",
        "if (getRversion() >= \"4.0.0\") {",
        "  cond_fn <- function() 2",
        "}",
        'assign("assigned_fn", function() 3)',
        'setGeneric("gen_fn", function(x) standardGeneric("gen_fn"))'
      ),
      "testpkg-wrappers.R" = 'wrapper_fn <- function() .Call("C_wrapper_fn")'
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl("no stale NAMESPACE exports", result$pass, fixed = TRUE)))
  expect_false(any(grepl("stale NAMESPACE export", result$warn, fixed = TRUE)))
})

test_that("hand-written R/ exports beyond the wrappers are NOT flagged", {
  # The available set is the UNION of R/ sources and the wrappers file: a
  # package with hand-written exported helpers must come out clean.
  pkg <- make_stale_export_pkg(
    ns_lines = c(ns_header, "export(hand_fn)", "export(wrapper_fn)"),
    r_files = list(
      "helpers.R" = "hand_fn <- function() 42",
      "testpkg-wrappers.R" = 'wrapper_fn <- function() .Call("C_wrapper_fn")'
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl("no stale NAMESPACE exports", result$pass, fixed = TRUE)))
  expect_false(any(grepl("stale NAMESPACE export", result$warn, fixed = TRUE)))
})

test_that("re-exports (importFrom + export) are NOT flagged", {
  pkg <- make_stale_export_pkg(
    ns_lines = c(
      ns_header,
      "importFrom(utils, head)",
      "export(head)",
      "export(wrapper_fn)"
    ),
    r_files = list(
      "testpkg-wrappers.R" = 'wrapper_fn <- function() .Call("C_wrapper_fn")'
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl("no stale NAMESPACE exports", result$pass, fixed = TRUE)))
  expect_false(any(grepl("stale NAMESPACE export", result$warn, fixed = TRUE)))
})

test_that("whole-package imports attribute installed exports and still flag stale names", {
  pkg <- make_stale_export_pkg(
    ns_lines = c(ns_header, "import(stats)", "export(median)", "export(gone_fn)"),
    r_files = list("testpkg-wrappers.R" = "wrapper_fn <- function() 1")
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  msgs <- testthat::capture_messages(result <- miniextendr_doctor(pkg))
  expect_identical(grep("^stale NAMESPACE export: ", result$warn, value = TRUE),
                   "stale NAMESPACE export: gone_fn")
  expect_false(any(grepl("Stale-export check skipped", msgs, fixed = TRUE)))
})

# Installed metadata can be inspected without loading any dependency code.
# Deliberately omit installation machinery and make the R code fail if sourced.
make_doctor_import <- function(package, namespace, metadata = NULL) {
  lib <- withr::local_tempdir(.local_envir = parent.frame())
  root <- file.path(lib, package)
  dir.create(root)
  writeLines(c(paste("Package:", package), "Version: 0.1.0", "Title: Test"),
             file.path(root, "DESCRIPTION"))
  writeLines(namespace, file.path(root, "NAMESPACE"))
  dir.create(file.path(root, "R"))
  writeLines('stop("doctor must not execute dependency code")', file.path(root, "R", "code.R"))
  if (!is.null(metadata)) {
    dir.create(file.path(root, "Meta"))
    saveRDS(metadata, file.path(root, "Meta", "nsInfo.rds"))
  }
  withr::local_libpaths(lib, action = "prefix", .local_envir = parent.frame())
  root
}

test_that("whole imports prefer installed metadata without loading namespaces", {
  make_doctor_import("mxdoctormeta", "this is invalid namespace syntax (((",
                     list(exports = c("available", "shared"), exportPatterns = character()))
  make_doctor_import("mxdoctorsecond", "export(second)",
                     list(exports = "second", exportPatterns = character()))
  pkg <- make_stale_export_pkg(
    c(ns_header, "import(mxdoctormeta, mxdoctorsecond)",
      "export(available, shared, second, gone_fn)"),
    list("testpkg-wrappers.R" = "wrapper_fn <- function() 1")
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  expect_false(any(c("mxdoctormeta", "mxdoctorsecond") %in% loadedNamespaces()))
  res <- stale_namespace_exports(pkg)
  expect_identical(res$status, "ok")
  expect_identical(res$stale, "gone_fn")
  expect_false(any(c("mxdoctormeta", "mxdoctorsecond") %in% loadedNamespaces()))
})

test_that("import except removes only that dependency's contribution", {
  make_doctor_import("mxdoctorexcept", "export(kept, omitted, local_name)",
                     list(exports = c("kept", "omitted", "local_name"), exportPatterns = character()))
  pkg <- make_stale_export_pkg(
    c(ns_header, 'import(mxdoctorexcept, except = c("omitted", "local_name"))',
      "importFrom(utils, head)", "export(kept, omitted, local_name, head)"),
    list("testpkg-wrappers.R" = "local_name <- function() 1")
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  res <- stale_namespace_exports(pkg)
  expect_identical(res$status, "ok")
  expect_identical(res$stale, "omitted")
})

test_that("whole imports fall back to shipped NAMESPACE when metadata is absent or unreadable", {
  for (cache_case in c("absent", "unreadable", "invalid-shape")) {
    root <- make_doctor_import("mxdoctorfallback", "export(fallback_fn)")
    if (cache_case != "absent") {
      dir.create(file.path(root, "Meta"))
      cache <- file.path(root, "Meta", "nsInfo.rds")
      if (cache_case == "unreadable") {
        writeLines("not an RDS file", cache)
      } else {
        saveRDS(42L, cache)
      }
    }
    pkg <- make_stale_export_pkg(
      c(ns_header, "import(mxdoctorfallback)", "export(fallback_fn, gone_fn)"),
      list("testpkg-wrappers.R" = "wrapper_fn <- function() 1")
    )
    on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
    res <- stale_namespace_exports(pkg)
    expect_identical(res$status, "ok")
    expect_identical(res$stale, "gone_fn")
    expect_false("mxdoctorfallback" %in% loadedNamespaces())
  }
})

test_that("unavailable whole imports explain why static attribution skips", {
  make_doctor_import("mxdoctorpattern", 'exportPattern("^dynamic_")',
                     list(exports = "known", exportPatterns = "^dynamic_"))
  make_doctor_import("mxdoctorbroken", "export(unbalanced")
  reasons <- c(mxdoctormissing = "import-not-installed",
               mxdoctorpattern = "import-export-patterns",
               mxdoctorbroken = "import-parse-error")
  for (package in names(reasons)) {
    pkg <- make_stale_export_pkg(
      c(ns_header, sprintf("import(%s)", package), "export(gone_fn)"),
      list("testpkg-wrappers.R" = "wrapper_fn <- function() 1")
    )
    on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
    res <- stale_namespace_exports(pkg)
    expect_identical(res$status, "skip")
    expect_identical(res$reason, unname(reasons[[package]]))
    expect_identical(res$detail, package)
    msgs <- testthat::capture_messages(result <- miniextendr_doctor(pkg))
    expect_false(any(grepl("stale NAMESPACE export", result$warn, fixed = TRUE)))
    flat <- gsub("\\s+", " ", paste(msgs, collapse = " "))
    expect_match(flat, "Stale-export check skipped", fixed = TRUE)
    expect_match(flat, sprintf("import(%s)", package), fixed = TRUE)
    expect_false(grepl("{.code", flat, fixed = TRUE))
  }
})

test_that("exportPattern() does not rescue a stale explicit export, and its presence is noted", {
  pkg <- make_stale_export_pkg(
    ns_lines = c(
      ns_header,
      'exportPattern("^mx_")',
      "export(gone_fn)",
      "export(wrapper_fn)"
    ),
    r_files = list(
      "testpkg-wrappers.R" = c(
        'wrapper_fn <- function() .Call("C_wrapper_fn")',
        "mx_pattern_fn <- function() 1"
      )
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  msgs <- testthat::capture_messages(
    result <- miniextendr_doctor(pkg)
  )

  # Explicit export() names must still resolve at loadNamespace() time, so
  # the stale one is flagged even with a pattern present; pattern-derived
  # exports themselves are not validated, and doctor says so.
  expect_identical(
    grep("^stale NAMESPACE export: ", result$warn, value = TRUE),
    "stale NAMESPACE export: gone_fn"
  )
  flat <- gsub("\\s+", " ", paste(msgs, collapse = " "))
  expect_match(flat, "pattern-derived exports", fixed = TRUE)
})

test_that("the check skips with a note when no generated wrappers file exists yet", {
  # Fresh clone: the wrappers file is gitignored, so NAMESPACE legitimately
  # lists exports whose wrappers are not on disk until the first build.
  # Flagging them all would be a false-positive flood.
  pkg <- make_stale_export_pkg(
    ns_lines = c(ns_header, "export(rust_backed_fn)"),
    r_files = list(
      "code.R" = "internal_helper <- function() 1"
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  msgs <- testthat::capture_messages(
    result <- miniextendr_doctor(pkg)
  )

  expect_false(any(grepl("stale NAMESPACE export", result$warn, fixed = TRUE)))
  expect_false(any(grepl("no stale NAMESPACE exports", result$pass, fixed = TRUE)))
  flat <- gsub("\\s+", " ", paste(msgs, collapse = " "))
  expect_match(flat, "Stale-export check skipped", fixed = TRUE)
  expect_match(flat, "miniextendr_build()", fixed = TRUE)
})

test_that("stale_namespace_exports skips on unparseable sources rather than guessing", {
  # A syntax error anywhere in R/ makes the definition union untrustworthy:
  # skip (reason parse-error, detail = the file) instead of flagging exports
  # whose definitions simply were not collectable.
  pkg <- make_stale_export_pkg(
    ns_lines = c(ns_header, "export(wrapper_fn)"),
    r_files = list(
      "broken.R" = "this is not valid R (((",
      "testpkg-wrappers.R" = 'wrapper_fn <- function() .Call("C_wrapper_fn")'
    )
  )
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  res <- stale_namespace_exports(pkg)
  expect_identical(res$status, "skip")
  expect_identical(res$reason, "parse-error")
  expect_identical(res$detail, file.path("R", "broken.R"))

  # An unparseable NAMESPACE is the same class of skip.
  ns_pkg <- make_stale_export_pkg(
    ns_lines = "export(unbalanced",
    r_files = list(
      "testpkg-wrappers.R" = 'wrapper_fn <- function() .Call("C_wrapper_fn")'
    )
  )
  on.exit(unlink(ns_pkg, recursive = TRUE), add = TRUE)

  ns_res <- stale_namespace_exports(ns_pkg)
  expect_identical(ns_res$status, "skip")
  expect_identical(ns_res$reason, "parse-error")
  expect_identical(ns_res$detail, "NAMESPACE")
})

test_that("r_top_level_definitions collects every static definition form and reports parse failures", {
  src <- withr::local_tempfile(fileext = ".R")
  writeLines(
    c(
      "arrow_fn <- function() 1",
      "eq_fn = function() 2",
      "super_fn <<- function() 3",
      # NB: `function() 4 -> right_fn` would parse as an anonymous function
      # whose *body* is the assignment; assign a value to test `->` itself.
      "4 -> right_fn",
      "`%||%` <- function(a, b) if (is.null(a)) b else a",
      "`val<-` <- function(x, value) x",
      "plain_obj <- letters",
      'assign("assigned_obj", 1)',
      'delayedAssign("delayed_obj", 2)',
      'setGeneric("generic_fn", function(x) standardGeneric("generic_fn"))',
      "if (TRUE) then_fn <- function() 5 else else_fn <- function() 6",
      "{",
      "  braced_fn <- function() 7",
      "}",
      "# dynamic names are deliberately invisible to the scan:",
      "assign(paste0('dyn', '_fn'), function() 8)"
    ),
    src
  )

  scan <- r_top_level_definitions(src)
  expect_setequal(
    scan$names,
    c(
      "arrow_fn", "eq_fn", "super_fn", "right_fn", "%||%", "val<-",
      "plain_obj", "assigned_obj", "delayed_obj", "generic_fn",
      "then_fn", "else_fn", "braced_fn"
    )
  )
  expect_identical(scan$failed, character(0))

  broken <- withr::local_tempfile(fileext = ".R")
  writeLines("not R at all (((", broken)
  broken_scan <- r_top_level_definitions(c(src, broken))
  expect_identical(broken_scan$failed, broken)
  expect_true("arrow_fn" %in% broken_scan$names)
})
