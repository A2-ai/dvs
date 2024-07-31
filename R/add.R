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
  if (inherits(files, "extendr_error")) {
    rlang::abort(files$value,"dvs_glob_error", parent = NA)
  }
  val_or_err <- dvs_add_impl(files, message, strict, split_output)
  if (inherits(val_or_err, "extendr_error")) {
    rlang::abort(val_or_err$value,"dvs_get_error", parent = NA)
  }
  return(val_or_err)
}

