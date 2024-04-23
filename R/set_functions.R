# rextendr::document()
# devtools::load_all()

#' initialize devious
#'
#' @param storage_directory path to storage directory
#' @param permissions optional argument to set linux file permissions (in octal format)
#' @param group optional argument to set group of files stored
#'
#'
#' @return nothing, or errors initializing
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
#' @param files files to add to the storage directory
#' @param message optional argument to add a message to future data frame rows associated with these files
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
#' @param files files to get from the storage directory
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
#' @param files optional argument, when specified, returns data frame with only these specified files
#'
#' @return a data frame with the statuses of files previously added
#'
#' @import purrr
#'
#' @export
dvs_status <- function(files = c()) {
    files <- files |> purrr::map_chr(normalizePath, mustWork = FALSE)
  dvs_status_impl(files)
}

