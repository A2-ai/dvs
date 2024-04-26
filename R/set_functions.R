# rextendr::document()
# devtools::load_all()

#' initialize devious
#'
#' @param storage_directory path to the desired storage directory
#' @param permissions optional: linux file permissions to automatically set for files added to the storage directory (in octal format)
#' @param group optional: group to automatically set for files added to the storage directory
#'
#' @examples
#' \dontrun{
#' dvs_init("/data/project_x") # would initialize the project's storage directory at /data/project_x
#' dvs_init("/data/project_x", 777, "project_x_group") # would initialize the project's storage directory at /data/project_x and configure the linux permissions "777" and group "project_x_group" for all files added to the storage directory
#' }
#'
#' @return nothing unless an initializing error occurs
#'
#' @export
dvs_init <- function(storage_directory, permissions = 664, group = "") {
  storage_directory <- normalizePath(storage_directory, mustWork = FALSE)
  dvs_init_impl(storage_directory, permissions, group)
  return(invisible())
}

#' @import stringr
clean_paths <- function(files) {
  for (i in seq_along(files)) {
    if (stringr::str_detect(files[i], "~")) {
      files[i] <- normalizePath(files[i], mustWork = FALSE)
    }
  }
  files
}

#' add files to the storage directory
#'
#' @param files file paths or glob patterns to add to the storage directory
#' @param message optional argument to add a message to future data frame rows associated with these files
#'
#' @examples
#' \dontrun{
#' dvs_add("data/derived/*.csv") # would add all csv files in data/derived to the initialized storage directory
#' dvs_add(c("data/derived/*", "model/nonmem/1001/1001.ext")) would add all files in data/derived (excluding .dvsmeta and .gitignore files) and model/nonmem/1001/1001.ext to the initialized storage directory
#' }
#'
#' @return a data frame with the states of success of added files
#'
#' @export
dvs_add <- function(files, message = "", one_df = TRUE) {
  files <- clean_paths(files)
  strict = TRUE
  dvs_add_impl(files, message, strict, one_df)
} # dvs_add

#' get added files
#'
#' @details retrieves files previously added with [dvs_add] to the storage directory (initialized by [dvs_init]).
#' If there's an error retrieving a particular file, the function itself will not return an error, rather the error
#' will be indicated in the returned data frame.
#'
#' @param files file paths or glob patterns to get from the storage directory
#'
#'@examples
#' \dontrun{
#' # would get all previously added files in data/derived from
#' # the initialized storage directory
#' dvs_get("data/derived/*.dvsmeta")
#'
#' # would get all files in data/derived (excluding
#' # .gitignore files) and ~Projects/project_x/large_file.pdf
#' # from the initialized storage directory
#' dvs_get(c("data/derived/*", "~Projects/project_x/large_file.pdf"))
#'}
#'
#' @return a data frame with the states of success of retrieved files
#'
#' @import purrr
#'
#' @export
dvs_get <- function(files, one_df = TRUE) {
  files <- clean_paths(files)
  dvs_get_impl(files, one_df)
}

#' status report for added files
#'
#' @details gives the statuses of previously added files (`up-to-date`, `out-of-sync`, or `not-present`)
#' to make users aware if files stored in the storage directory don't exist in their local repository or have been updated.
#' If no file paths or glob patterns are inputted, `dvs_status` gives the status of all previously added files.
#' If there an error getting the status of a particular file, the function itself will not return an error, rather, a given error will be indicated in the data frame output.
#'
#'
#' @param files optional: when specified, returns data frame with only these specified file paths or glob patterns.
#'
#' @return a data frame with the statuses of previously added files
#'
#' @examples
#' \dontrun{
#'   # would give the status of all previously added files
#'   dvs_status()
#'
#'   # would attempt to get the status of all files in data/derived (except for .gitignore files)
#'   dvs_status("data/derived/*")
#' }
#'
#' @import purrr
#'
#' @export
dvs_status <- function(files = c()) {
  files <- clean_paths(files)
  dvs_status_impl(files)
}

dvs_info <- function(files = c(), one_df = TRUE) {
  files <- clean_paths(files)
  get_file_info_impl(files, one_df)
}

