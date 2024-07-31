#' status report for added files
#'
#' @details gives the statuses of previously added files (`current`, `unsynced`,
#' `absent`, or `error`) to make users aware if files stored in the storage
#' directory don't exist in their local repository or have been updated.
#' If no file paths or glob patterns are inputted, `dvs_status` gives the status
#' of all previously added files.
#' If there an error getting the status of a particular file, the function
#' itself will not return an error, rather, a given error will be indicated in
#' the data frame output.
#'
#'
#' @param files optional: when specified, returns data frame with only these
#' specified file paths or glob patterns.
#' @param split_output when split_output is true, a list of two data frames -
#' `successes` and `failures` - is returned.
#' Rows in `successes` are files with successfully curated statuses, and rows in
#' `failures` are inputs that returned errors.
#' When split_output is false, the output is a single data frame with all
#' attempted file status reports, and whose success or failure is indicated as
#' such in the outcome column.
#'
#' @return a data frame with the statuses of previously added files
#'
#' @examples
#' \dontrun{
#'   # would give the status of all previously added files
#'   dvs_status()
#'
#'   # would attempt to get the status of all files in data/derived
#'   # (except for .gitignore files)
#'   dvs_status("data/derived/*")
#' }
#'
#' @export
dvs_status <- function(files = c(""), split_output = FALSE) {
  files <- normalize_paths(files)
  files <- parse_files_from_globs_status_impl(files)
  if (inherits(files, "extendr_error")) {
    rlang::abort(files$value,"dvs_glob_error", parent = NA)
  }
  val_or_err <- dvs_status_impl(files, split_output)
  if (inherits(val_or_err, "extendr_error")) {
    rlang::abort(val_or_err$value,"dvs_get_error", parent = NA)
  }
  return(val_or_err)
}

