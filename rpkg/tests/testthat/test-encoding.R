# Encoding / locale assertion tests for package_init.
#
# `package_init` (miniextendr-api/src/init.rs) calls
# `miniextendr_assert_utf8_locale()` once during `R_init_*`. That assertion
# rejects sessions whose locale isn't UTF-8, so unmarked/native CHARSXPs have
# an unambiguous UTF-8 interpretation. Explicit Latin-1 CHARSXPs can still
# occur in such a session and must be translated per string.
#
# These tests cover three layers:
#   1. The FFI lookup of `l10n_info()[["UTF-8"]]` works.
#   2. The assertion's success path (UTF-8 locale).
#   3. The assertion's failure path (non-UTF-8 locale) — exercised both
#      mid-session via `Sys.setlocale()` and at fresh load via `callr::r()`
#      with explicit `LC_ALL` / `LC_CTYPE` overrides.
#
# R's docs warn that changing `LC_CTYPE` mid-session may not take effect on
# every platform. Each mid-session test guards with the actual `l10n_info()`
# return so the test only runs when the platform actually flipped the locale.

# region: cached snapshot

test_that("encoding_info_available returns logical", {
  result <- encoding_info_available()
  expect_type(result, "logical")
  # Standard R package builds: encoding_init is disabled (symbols not
  # exported from libR), so this is FALSE. Embedded / nonapi builds may
  # populate the snapshot.
})

# endregion

# region: l10n_info baseline

test_that("test session reports UTF-8 (R >= 4.2 default)", {
  # The miniextendr package literally cannot have loaded if this were FALSE
  # (package_init would have thrown), so the test session must report UTF-8.
  expect_true(isTRUE(l10n_info()[["UTF-8"]]))
})

# endregion

# region: per-string encoding tags

latin1_facade <- function() {
  value <- rawToChar(as.raw(c(0x66, 0x61, 0xe7, 0x61, 0x64, 0x65)))
  Encoding(value) <- "latin1"
  value
}

test_that("owned String conversion translates Latin-1 in a UTF-8 session", {
  value <- latin1_facade()
  expect_identical(Encoding(value), "latin1")
  expect_identical(conv_string_arg(value), enc2utf8(value))
})

test_that("borrowed str conversion translates Latin-1 in a UTF-8 session", {
  value <- latin1_facade()
  expect_equal(miniextendr:::str_borrow_len(value), 6L)
})

test_that("Cow strings own only elements that require translation", {
  value <- latin1_facade()
  expect_true(zero_copy_cow_str_is_borrowed("façade"))
  expect_false(zero_copy_cow_str_is_borrowed(value))
  expect_false(zero_copy_vec_cow_str_all_borrowed(c("ascii", value)))
})

test_that("bytes-encoded strings are rejected as non-text", {
  value <- rawToChar(as.raw(0xff))
  Encoding(value) <- "bytes"
  expect_error(conv_string_arg(value), "marked as bytes")
  expect_error(miniextendr:::str_borrow_len(value), "marked as bytes")
  expect_error(zero_copy_cow_str_is_borrowed(value), "marked as bytes")
})

test_that("malformed strings tagged as UTF-8 are rejected before creating str", {
  value <- rawToChar(as.raw(0xff))
  Encoding(value) <- "UTF-8"
  expect_error(conv_string_arg(value), "marked as UTF-8 contains invalid UTF-8")
})

# endregion

# region: assertion success path

test_that("assert_utf8_locale_now succeeds in a UTF-8 test session", {
  expect_true(assert_utf8_locale_now())
})

test_that("assert_utf8_locale_now is idempotent", {
  for (i in seq_len(5)) {
    expect_true(assert_utf8_locale_now())
  }
})

# endregion

# region: assertion failure path (mid-session locale flip)

# Helper: temporarily switch LC_CTYPE, restoring on exit. Returns the value
# Sys.setlocale() actually returned ("" on failure) so callers can skip when
# the platform refused the requested locale.
with_lc_ctype <- function(loc, code) {
  original <- Sys.getlocale("LC_CTYPE")
  applied <- suppressWarnings(Sys.setlocale("LC_CTYPE", loc))
  on.exit(suppressWarnings(Sys.setlocale("LC_CTYPE", original)), add = TRUE)
  list(applied = applied, value = if (nzchar(applied)) force(code) else NULL)
}

test_that("assert_utf8_locale_now errors after Sys.setlocale to C", {
  res <- with_lc_ctype("C", {
    # Some platforms keep `l10n_info()[['UTF-8']]` TRUE even after
    # `LC_CTYPE=C` (R caches `utf8locale` early). Only assert the failure
    # path when l10n_info actually flipped — otherwise the assertion is
    # correctly returning success.
    if (isTRUE(l10n_info()[["UTF-8"]])) {
      skip("Platform did not flip l10n_info()[['UTF-8']] under LC_CTYPE=C")
    }
    expect_error(
      assert_utf8_locale_now(),
      regexp = "UTF-8 locale",
      fixed = FALSE
    )
  })
  if (!nzchar(res$applied)) {
    skip("Platform refused Sys.setlocale('LC_CTYPE', 'C')")
  }
})

test_that("assert_utf8_locale_now errors after Sys.setlocale to a Latin-1 locale", {
  # Try a few well-known Latin-1 locale names; skip if none take.
  candidates <- c("en_US.ISO8859-1", "en_US.iso88591", "C.ISO-8859-1")
  applied <- ""
  original <- Sys.getlocale("LC_CTYPE")
  for (cand in candidates) {
    applied <- suppressWarnings(Sys.setlocale("LC_CTYPE", cand))
    if (nzchar(applied)) break
  }
  on.exit(suppressWarnings(Sys.setlocale("LC_CTYPE", original)), add = TRUE)

  if (!nzchar(applied)) {
    skip("No Latin-1 locale available on this platform")
  }
  if (isTRUE(l10n_info()[["UTF-8"]])) {
    skip("Latin-1 locale did not flip l10n_info()[['UTF-8']]")
  }
  expect_error(
    assert_utf8_locale_now(),
    regexp = "UTF-8 locale"
  )
})

test_that("assert_utf8_locale_now succeeds again after restoring UTF-8", {
  original <- Sys.getlocale("LC_CTYPE")
  on.exit(suppressWarnings(Sys.setlocale("LC_CTYPE", original)), add = TRUE)

  flipped <- suppressWarnings(Sys.setlocale("LC_CTYPE", "C"))
  if (!nzchar(flipped) || isTRUE(l10n_info()[["UTF-8"]])) {
    skip("Could not flip locale away from UTF-8 to test restore")
  }
  # Restore something UTF-8.
  utf8 <- ""
  for (cand in c("en_US.UTF-8", "C.UTF-8", "en_GB.UTF-8")) {
    utf8 <- suppressWarnings(Sys.setlocale("LC_CTYPE", cand))
    if (nzchar(utf8)) break
  }
  if (!nzchar(utf8) || !isTRUE(l10n_info()[["UTF-8"]])) {
    skip("No UTF-8 locale available to restore")
  }
  expect_true(assert_utf8_locale_now())
})

# endregion

# region: package load under explicit locale (subprocess)

# These tests spawn a fresh R via callr::r() with the requested locale env
# vars and observe whether `library(miniextendr)` succeeds. This is the only
# way to exercise the assertion at *load* time — its real production path.

skip_on_os("windows")  # Windows locale names + processx orphan handling are
                      # both finicky; the locale-load gate is platform-
                      # independent in implementation, so Linux + macOS cover
                      # the contract.

run_load_in_locale <- function(env) {
  callr::r(
    function() {
      tryCatch(
        {
          library(miniextendr)
          # Cheap call — exercises a real .Call to confirm load succeeded.
          list(ok = TRUE, utf8 = isTRUE(l10n_info()[["UTF-8"]]))
        },
        error = function(e) list(ok = FALSE, message = conditionMessage(e))
      )
    },
    env = env,
    show = FALSE
  )
}

# Helper: pick a locale from candidates that the spawned subprocess actually
# accepts. Returns "" if none work.
probe_locale <- function(var, candidates) {
  for (cand in candidates) {
    env <- c(callr::rcmd_safe_env(), setNames(cand, var))
    res <- tryCatch(
      callr::r(
        function() Sys.getlocale("LC_CTYPE"),
        env = env,
        show = FALSE
      ),
      error = function(e) ""
    )
    if (nzchar(res) && !identical(res, "C") && !identical(res, "POSIX")) {
      # Subprocess successfully picked up the locale.
      return(cand)
    }
    if (var == "LC_ALL" && identical(cand, "C")) {
      # We *want* C here.
      return(cand)
    }
  }
  ""
}

test_that("library(miniextendr) succeeds under LC_ALL=C.UTF-8", {
  cand <- probe_locale("LC_ALL", c("C.UTF-8", "en_US.UTF-8"))
  if (!nzchar(cand)) skip("No UTF-8 locale available in subprocess")
  env <- c(callr::rcmd_safe_env(), LC_ALL = cand)
  res <- run_load_in_locale(env)
  expect_true(res$ok, info = res$message %||% "")
  expect_true(res$utf8)
})

test_that("library(miniextendr) succeeds under LC_CTYPE=en_US.UTF-8 with LANG=C", {
  cand <- probe_locale("LC_CTYPE", c("en_US.UTF-8", "C.UTF-8"))
  if (!nzchar(cand)) skip("No UTF-8 LC_CTYPE candidate available")
  env <- c(callr::rcmd_safe_env(), LANG = "C", LC_CTYPE = cand)
  res <- run_load_in_locale(env)
  expect_true(res$ok, info = res$message %||% "")
  expect_true(res$utf8)
})

test_that("library(miniextendr) errors under LC_ALL=C", {
  env <- c(callr::rcmd_safe_env(), LC_ALL = "C")

  # First confirm that LC_ALL=C actually produces a non-UTF-8 session in the
  # subprocess; some glibc builds default to C.UTF-8 even when asked for C.
  utf8_in_sub <- tryCatch(
    callr::r(
      function() isTRUE(l10n_info()[["UTF-8"]]),
      env = env,
      show = FALSE
    ),
    error = function(e) NA
  )
  if (isTRUE(utf8_in_sub)) {
    skip("Subprocess kept UTF-8 even under LC_ALL=C")
  }

  res <- run_load_in_locale(env)
  expect_false(res$ok)
  expect_match(res$message, "UTF-8 locale")
})

test_that("library(miniextendr) errors under LC_ALL=POSIX", {
  env <- c(callr::rcmd_safe_env(), LC_ALL = "POSIX")
  utf8_in_sub <- tryCatch(
    callr::r(
      function() isTRUE(l10n_info()[["UTF-8"]]),
      env = env,
      show = FALSE
    ),
    error = function(e) NA
  )
  if (isTRUE(utf8_in_sub)) {
    skip("Subprocess kept UTF-8 even under LC_ALL=POSIX")
  }
  res <- run_load_in_locale(env)
  expect_false(res$ok)
  expect_match(res$message, "UTF-8 locale")
})

# endregion
