# rextendr::document()
# devtools::load_all()
# dvs_init("test_directory", group = "datascience")
# dvs_init("test_directory")
library(rlang)

dvs_init <- function(directory, permissions = 511, group = "") {
  res <- dvs_init_r(directory, permissions, group)
  if (!is.null(res$error)) {
    rlang::abort(res$error)
  }
  return(invisible())
}

dvs_add <- function(files, message = "") {
  dvs_add_r(files, message)
}

dvs_get <- function(files) {
  dvs_get_r(files)
}

dvs_status <- function(files = c("")) {
  dvs_status_r(files)
}

