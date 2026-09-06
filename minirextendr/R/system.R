# System command execution with stderr/stdout capture

#' Execute system command with stderr/stdout capture
#'
#' Wrapper around system2() that saves stderr/stdout to timestamped log
#' files for later inspection when errors occur.
#'
#' @param command Command to execute
#' @param args Character vector of arguments, without shell quoting.
#' @param log_prefix Prefix for log file (e.g., "autoconf", "configure")
#' @param wd Working directory (uses current if NULL)
#' @param env Environment variables
#' @param log_dir Directory to save log files (defaults to tempdir())
#' @return List with status, output, and log_file path
#' @keywords internal
run_with_logging <- function(command, args = character(),
                              log_prefix = "command",
                              wd = NULL,
                              env = character(),
                              log_dir = tempdir()) {
  # Create timestamp for log file
  timestamp <- format(Sys.time(), "%Y-%m-%d-%H-%M-%S")
  log_file <- file.path(log_dir, sprintf("%s-%s.log", timestamp, log_prefix))

  # Ensure log directory exists
  if (!dir.exists(log_dir)) {
    dir.create(log_dir, recursive = TRUE, showWarnings = FALSE)
  }

  # Normalize env to a named character vector.
  # Accepts either named vector c(FOO = "bar") or "FOO=bar" strings.
  if (length(env) > 0 && is.null(names(env))) {
    eq_pos <- regexpr("=", env, fixed = TRUE)
    nms <- substr(env, 1, eq_pos - 1)
    vals <- substr(env, eq_pos + 1, nchar(env))
    env <- stats::setNames(vals, nms)
  }

  # Run command via system2 with env vars and working directory
  run_fn <- function() {
    if (length(env) > 0) {
      # names = TRUE forces a named result regardless of length(env).
      # Sys.getenv() auto-names only for length > 1, so a single-variable env
      # would otherwise lose its name and the on.exit() restore below would
      # call Sys.setenv("value") with no name -> "all arguments must be named".
      old_env <- Sys.getenv(names(env), unset = NA, names = TRUE)
      do.call(Sys.setenv, as.list(env))
      on.exit({
        # Restore: unset vars that were previously unset, restore others
        to_unset <- names(old_env)[is.na(old_env)]
        to_restore <- old_env[!is.na(old_env)]
        if (length(to_unset) > 0) Sys.unsetenv(to_unset)
        if (length(to_restore) > 0) do.call(Sys.setenv, as.list(to_restore))
      }, add = TRUE)
    }
    # The exit status is captured deterministically via attr(output, "status")
    # below and surfaced through $status/$success. On a non-zero exit with
    # captured output, base R *also* raises a "running command ... had status N"
    # warning -- redundant noise here, since the failure is a handled, expected
    # outcome. Suppress it so callers' logs (and tests) stay clean. (#798)
    suppressWarnings(
      system2(command, args = shQuote(args), stdout = TRUE, stderr = TRUE)
    )
  }

  if (!is.null(wd)) {
    old_wd <- setwd(wd)
    on.exit(setwd(old_wd), add = TRUE)
    output <- run_fn()
  } else {
    output <- run_fn()
  }

  status <- attr(output, "status")
  if (is.null(status)) status <- 0L

  # Save output to log file
  writeLines(c(
    sprintf("Command: %s %s", command, paste(args, collapse = " ")),
    sprintf("Working directory: %s", wd %||% getwd()),
    sprintf("Timestamp: %s", Sys.time()),
    "",
    "=== Output ===",
    output
  ), log_file)

  list(
    status = if (status == 0) NULL else status,
    output = output,
    log_file = log_file,
    success = status == 0
  )
}

#' Check command result and throw informative error if failed
#'
#' Internal helper used by workflow wrappers.
#'
#' @param result Result from run_with_logging()
#' @param context Description of what was being done
#' @keywords internal
check_result <- function(result, context) {
  if (!result$success) {
    cli::cli_alert_danger("{context} failed (status: {result$status %||% 'unknown'})")
    cli::cli_alert_info("Log saved to: {.path {result$log_file}}")

    # Show last 20 lines of output
    tail_output <- utils::tail(result$output, 20)
    if (length(tail_output) > 0) {
      cli::cli_alert_info("Last 20 lines of output:")
      cli::cli_verbatim(paste(tail_output, collapse = "\n"))
    }

    cli::cli_abort(c(
      paste0(context, " failed"),
      "i" = paste0("Full output saved to: ", result$log_file)
    ))
  }
  invisible(result)
}

#' Run a command and capture output
#'
#' Internal helper that runs a command via system2 and returns the combined
#' stdout+stderr as a character vector, with the exit status as an attribute.
#'
#' @param command Command to execute
#' @param args Character vector of arguments
#' @param wd Working directory (NULL for current)
#' @return Character vector of output lines. The exit status is stored as
#'   the \code{"status"} attribute (NULL when 0, matching system2 convention).
#' @keywords internal
run_command <- function(command, args = character(), wd = NULL) {
  if (!is.null(wd)) {
    old_wd <- setwd(wd)
    on.exit(setwd(old_wd), add = TRUE)
  }
  # Callers inspect attr(result, "status") to detect failure (a non-zero exit is
  # an expected, handled outcome -- e.g. probing `git rev-parse` outside a repo).
  # Suppress base R's redundant "running command ... had status N" warning. (#798)
  suppressWarnings(system2(command, args = args, stdout = TRUE, stderr = TRUE))
}
