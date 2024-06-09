#' get or update files if they are in shared storage to add to the local repo
#'
#' @details retrieves files previously added with [dvs_add] to the storage
#' directory (initialized by [dvs_init]).
#' If a file is explicitly inputted without a corresponding dvs metadata file i.e.
#' hasn't been added yet with [dvs_add], the command returns an error.
#'
#' For any other error retrieving a particular file, the function itself will
#' indicate the error type and message in the data frame output; the function
#' itself will not return an error.
#'
#' @param files file paths or glob patterns to get from the storage directory
#' @param split_output when split_output is true, a list of two data frames -
#' `successes` and `failures` - is returned.
#' Rows in `successes` are files successfully retrieved, and rows in `failures`
#' are inputs that returned errors.
#' When split_output is false, the output is a single data frame with all files
#' attempted to recopy to the project directory,
#' and whose success or failure is indicated as such in the outcome column.
#'
#'@examples
#' \dontrun{
#' # would get all previously added files in data/derived from
#' # the initialized storage directory
#' dvs_get("data/derived/*.dvs")
#'
#' # would get all files in data/derived (excluding
#' # .gitignore files) and ~Projects/project_x/large_file.pdf
#' # from the initialized storage directory
#' dvs_get(c("data/derived/*", "~Projects/project_x/large_file.pdf"))
#'}
#'
#' @return one or two data frames whose rows are the files attempted to get in the given operation.
#'
#' @export
dvs_get <- function(files, split_output = FALSE) {
  files <- normalize_paths(files)
  files <- parse_files_from_globs_get_impl(files)
  if (inherits(files, "extendr_error")) {
    rlang::abort(files$value,"dvs_glob_error", parent = NA)
  }
  val_or_err <-   dvs_get_impl(files, split_output)
  if (inherits(val_or_err, "extendr_error")) {
    rlang::abort(val_or_err$value,"dvs_get_error", parent = NA)
  }
  return(val_or_err)
}
