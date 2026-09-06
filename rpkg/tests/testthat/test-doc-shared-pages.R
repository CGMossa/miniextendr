test_that("describeIn functions share the named help page in source order", {
  # load_all() registers the source directory as the namespace path; it has
  # generated man pages but no installed help database. Exercise both modes.
  pkg_path <- getNamespaceInfo("miniextendr", "path")
  db <- if (dir.exists(file.path(pkg_path, "man"))) {
    tools::Rd_db(dir = pkg_path)
  } else {
    tools::Rd_db("miniextendr")
  }
  aliases <- unlist(lapply(db, function(page) {
    nodes <- page[vapply(page, function(x) identical(attr(x, "Rd_tag"), "\\alias"), logical(1))]
    unique(unlist(nodes))
  }), use.names = FALSE)
  expect_length(unique(aliases[duplicated(aliases)]), 0L)
  expect_true("doc_shared_topic.Rd" %in% names(db))
  rd <- db[["doc_shared_topic.Rd"]]
  section <- function(tag) {
    nodes <- rd[vapply(rd, function(x) identical(attr(x, "Rd_tag"), tag), logical(1))]
    gsub("[[:space:]]+", " ", paste(unlist(nodes), collapse = ""))
  }
  expect_match(section("\\title"), "Shared documentation in source order.", fixed = TRUE)
  expect_match(section("\\section"), "Doubles each input value while retaining the input order.", fixed = TRUE)
  expect_match(section("\\section"), "Formats each input value with the shared documentation fixture.", fixed = TRUE)
  expect_match(section("\\keyword"), "utilities", fixed = TRUE)
  expect_match(section("\\keyword"), "methods", fixed = TRUE)
  expect_match(section("\\concept"), "shared documentation fixture", fixed = TRUE)
  usage <- section("\\usage")
  positions <- vapply(c("doc_shared_topic", "doc_shared_double", "doc_shared_vector"),
                     function(name) as.integer(regexpr(name, usage, fixed = TRUE)), integer(1))
  expect_true(all(positions > 0L))
  expect_true(all(diff(positions) > 0L))
  # Explicitly routed functions must not leak onto the automatic file-stem page.
  expect_false(any(grepl("doc_shared_", unlist(db[["doc_attr_tests.Rd"]]), fixed = TRUE)))
})

test_that("shared-page functions and the S3 method remain callable", {
  expect_identical(doc_shared_topic(c(1L, 3L)), c(1L, 3L))
  expect_identical(doc_shared_double(c(1L, 3L)), c(2L, 6L))
  expect_identical(format(structure(c(1L, 3L), class = "doc_shared_vector")), c("1", "3"))
})
