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
#' @return nothing, or initializing errors
#'
#' @export
dvs_init <- function(storage_directory, permissions = 664, group = "") {
  storage_directory <- normalizePath(storage_directory, mustWork = FALSE)
  dvs_init_impl(storage_directory, permissions, group)
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
#'
#' }
#'
#' @return a data frame with the states of success of added files
#'
#' @export
dvs_add <- function(files, message = "") {
  files <- clean_paths(files)

  strict = TRUE
  res <- dvs_add_impl(files, message, strict)

  list(successes = res[[1]], errors = res[[2]])
} # dvs_add

#' get files previously added to the storage directory
#'
#' @param files file paths or glob patterns to get from the storage directory
#'
#' @return a data frame with the states of success of retrieved files
#'
#' @import purrr
#'
#' @export
dvs_get <- function(files) {
  files <- files |> purrr::map_chr(normalizePath, mustWork = FALSE)
  dvs_get_impl(files)
}

#' get the status of any file previously added to the storage directory
#'
#' @param files optional argument: when specified, returns data frame with only these specified file paths or glob patterns
#'
#' @return a data frame with the statuses of all previously added files
#'
#' @import purrr
#'
#' @export
dvs_status <- function(files = c()) {
    files <- files |> purrr::map_chr(normalizePath, mustWork = FALSE)
  dvs_status_impl(files)
}

