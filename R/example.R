# rextendr::document()
# devtools::load_all()
# dvs_init("test_directory", group = "datascience")
# dvs_init("test_directory")
library(rlang)

dvs_init <- function(directory, permissions = 666, group = "") {
  res <- dvs_init_r(directory, permissions, group)
  if (!is.null(res$error)) {
    rlang::abort(res$error)
  }
  return(invisible())
}

dvs_add <- function(files, message = "") {
  res <- dvs_add_r(files, message)
  # if (!is.null(res$error)) {
  #   rlang::abort(res$error)
  # }
  res
}

