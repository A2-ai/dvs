# rextendr::document()
# devtools::load_all()
library(rlang)
library(purrr)

#' initialize devious
#'
#' @param directory path to storage directory
#' @param permissions optional argument to set linux file permissions (in octal format)
#' @param group optional argument to set group of files stored
#'
#' @return nothing, or errors initializing
#'
#' @export
dvs_init <- function(storage_directory, permissions = 664, group = "") {
  storage_directory <- normalizePath(storage_directory, mustWork = FALSE)
  res <- dvs_init_impl(storage_directory, permissions, group)
  if (!is.null(res$error)) {
    rlang::abort(res$error)
  }
  return(invisible())
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
  files <- files |> map_chr(normalizePath, mustWork = FALSE)
  dvs_add_impl(files, message)
}

#' get files previously added to the storage directory
#'
#' @param files files to get from the storage directory
#'
#' @return a data frame with the states of success of retrieved files
#'
#' @export
dvs_get <- function(files) {
  files <- files |> map_chr(normalizePath, mustWork = FALSE)
  dvs_get_impl(files)
}

#' get the status of any file previously added to the storage directory
#'
#' @param files optional argument, when specified, returns data frame with only these specified files
#'
#' @return a data frame with the statuses of files previously added
#'
#' @export
dvs_status <- function(files = c()) {
    files <- files |> map_chr(normalizePath, mustWork = FALSE)
  dvs_status_impl(files)
}

