# rextendr::document()
# devtools::load_all()
library(rlang)

#' initialize devious
#'
#' @param directory path to storage directory
#' @param permissions optional argument to set permissions of files stored
#' @param group optional argument to set group of files stored
#'
#' @return nothing, or errors initializing
#'
#' @export
dvs_init <- function(directory, permissions = 777, group = "") {
  res <- dvs_init_impl(directory, permissions, group)
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
  dvs_get_impl(files)
}

#' get the status of any file previously added to the storage directory
#'
#' @param files optional argument, when specified, returns data frame with only these specified files
#'
#' @return a data frame with the statuses of files previously added
#'
#' @export
dvs_status <- function(files = c("")) { # this is shoddy, but it won't accept c()
  dvs_status_impl(files)
}

