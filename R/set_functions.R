normalize_paths <- function(files) {
  # normalize if file path has ~
  path.expand(files)
}

#' copy files to the storage directory
#'
#' @details after initializing the storage directory with [dvs_init], this command
#' copies files to storage directory for other collaborators to version and retrieve with [dvs_get]
#' If an explicitly inputted file doesn't exist, the command returns an error .
#'
#' For any other error retrieving a particular file, the function itself will
#' indicate the error type and message in the data frame output; the function
#' itself will not return an error.
#'
#' @param files file paths or glob patterns to add to the storage directory
#' @param message optional: a message associated with the file(s) for versioning
#' context to appear in dvs metadata files
#' @param split_output optional: when `TRUE`, a list of two data frames is returned:
#' `successes` and `failures`.
#' - rows in `successes` are successfully added file inputs
#' - rows in `failures` are inputs that returned errors
#'
#' when `FALSE`, the output is a single data frame whose rows are the files
#' attempted to add in the given operation, the successes or failures of which are indicated in the
#' `outcome` column
#'
#' @return one or two data frames whose rows are the files attempted to add in the given operation.
#'
#' @examples
#' \dontrun{
#' # would add all csv files in data/derived to the initialized storage directory
#' dvs_add("data/derived/*.csv")
#'
#' # would add model/nonmem/1001/1001.ext and all files in data/derived
#' # to the initialized storage directory (excluding dvs metadata and .gitignore files)
#' dvs_add(c("model/nonmem/1001/1001.ext", "data/derived/*"))
#' }
#'
#' @export
dvs_add <- function(files, message = NULL, split_output = FALSE) {
  files <- normalize_paths(files)
  files <- parse_files_from_globs_add_impl(files)
  strict = TRUE
  dvs_add_impl(files, message, strict, split_output)
}


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
  dvs_status_impl(files, split_output)
}
#
# dvs_info <- function(files, split_output = FALSE) {
#   files <- clean_paths(files)
#   get_file_info_impl(files, split_output)
# }

