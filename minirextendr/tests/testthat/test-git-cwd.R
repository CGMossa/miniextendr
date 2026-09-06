# Git calls must run in the resolved project dir, not getwd()
# (audit 2026-07-06 finding #2). `with_project()` activates the usethis project
# without changing the working directory (setwd = FALSE), so a git call with no
# working-dir argument inspects the caller's cwd -- an unrelated repo, or none.

# Initialise a git repo at `dir` with a single committed file at `rel_path`.
init_repo_with_committed_file <- function(dir, rel_path, contents = "clean") {
  target <- file.path(dir, rel_path)
  dir.create(dirname(target), recursive = TRUE, showWarnings = FALSE)
  writeLines(contents, target)
  git <- function(...) {
    system2("git", c("-C", dir, ...), stdout = TRUE, stderr = TRUE)
  }
  git("init", "-q")
  git("config", "user.email", "test@example.com")
  git("config", "user.name", "minirextendr test")
  git("config", "commit.gpgsign", "false")
  git("add", "-A")
  git("commit", "-q", "-m", "init")
  invisible(dir)
}

# Initialise a git repo whose Cargo.toml declares a `[workspace]`.
init_workspace_repo <- function(dir) {
  writeLines(c("[workspace]", "members = []"), file.path(dir, "Cargo.toml"))
  system2("git", c("-C", dir, "init", "-q"), stdout = TRUE, stderr = TRUE)
  invisible(dir)
}

test_that("check_scaffolding_clean guards the active project, not getwd()", {
  # This is the guard-bypass regression: with the active usethis project set to
  # a dirty package but the working directory pointing elsewhere, the dirty
  # check must still fire. On the unfixed code the no-arg call probes getwd()
  # (a non-repo dir), silently returns early, and lets the upgrade overwrite the
  # target's uncommitted files.
  skip_if(!nzchar(Sys.which("git")), "git not available")

  pkg <- withr::local_tempdir()
  init_repo_with_committed_file(pkg, "src/stub.c")
  writeLines("dirty", file.path(pkg, "src", "stub.c"))

  usethis::local_project(pkg, force = TRUE, setwd = FALSE)

  # Working directory is neither the target nor a git repo.
  foreign <- withr::local_tempdir()
  withr::local_dir(foreign)

  expect_error(
    minirextendr:::check_scaffolding_clean(),
    "uncommitted changes"
  )
})

test_that("check_scaffolding_clean(proj_dir) inspects the given dir from any cwd", {
  skip_if(!nzchar(Sys.which("git")), "git not available")

  pkg <- withr::local_tempdir()
  init_repo_with_committed_file(pkg, "src/stub.c")
  writeLines("dirty", file.path(pkg, "src", "stub.c"))

  foreign <- withr::local_tempdir()
  withr::local_dir(foreign)

  expect_error(
    minirextendr:::check_scaffolding_clean(pkg),
    "uncommitted changes"
  )
})

test_that("check_scaffolding_clean passes a clean repo from a foreign cwd", {
  skip_if(!nzchar(Sys.which("git")), "git not available")

  pkg <- withr::local_tempdir()
  init_repo_with_committed_file(pkg, "src/stub.c")

  foreign <- withr::local_tempdir()
  withr::local_dir(foreign)

  expect_no_error(minirextendr:::check_scaffolding_clean(pkg))
})

test_that("find_workspace_root resolves its path argument, not getwd()", {
  skip_if(!nzchar(Sys.which("git")), "git not available")

  target <- withr::local_tempdir()
  init_workspace_repo(target)

  # A *different* workspace repo used as the foreign cwd. On the unfixed code
  # the git probe runs here and returns this root instead of `target`.
  foreign <- withr::local_tempdir()
  init_workspace_repo(foreign)
  withr::local_dir(foreign)

  expect_identical(
    minirextendr:::find_workspace_root(target),
    normalizePath(target)
  )
})

test_that("check_scaffolding_clean guards modified Windows linker settings", {
  skip_if(!nzchar(Sys.which("git")), "git not available")
  pkg <- withr::local_tempdir()
  init_repo_with_committed_file(pkg, "src/Makevars.win", "include Makevars")
  writeLines("PKG_LIBS = custom-windows-libs", file.path(pkg, "src", "Makevars.win"))

  expect_error(
    minirextendr:::check_scaffolding_clean(pkg),
    "uncommitted changes"
  )
})
