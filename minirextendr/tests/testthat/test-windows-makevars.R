# Missing Makevars.win omits the Windows system libraries required by Rust.
for (layout in c("rpkg", "monorepo")) {
  test_that(paste("Windows Makevars is scaffolded for", layout), {
    tmp <- withr::local_tempdir()
    usethis::local_project(tmp, force = TRUE, setwd = FALSE)
    old_type <- minirextendr:::get_template_type()
    withr::defer(minirextendr:::set_template_type(old_type))
    minirextendr:::set_template_type(layout)
    subdir <- if (layout == "monorepo") "rpkg" else NULL

    minirextendr:::use_miniextendr_makevars(subdir = subdir)
    dest <- file.path(tmp, "src", "Makevars.win")
    expect_true(file.exists(dest))
    expect_identical(readLines(dest), readLines(
      minirextendr:::template_path("Makevars.win", subdir = subdir)
    ))

    # Upgrading replaces a stale tracked copy with the current template.
    writeLines("# old Windows settings", dest)
    minirextendr:::use_miniextendr_makevars(subdir = subdir)
    expect_identical(readLines(dest), readLines(
      minirextendr:::template_path("Makevars.win", subdir = subdir)
    ))
  })
}
