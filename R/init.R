#' initialize devious to add/get versioned files from the storage directory
#'
#' @param storage_directory path to the desired storage directory for versioned files
#' @param permissions optional: linux file permissions to set
#' for files added to the storage directory (in octal format)
#' @param group optional: primary group to set for files added to the
#' storage directory
#'
#' @examples
#' \dontrun{
#' # would initialize the project's storage directory at /data/project_x
#' dvs_init("/data/project_x")
#'
#' # would initialize the project's storage directory at /data/project_x and
#' # configure the linux permissions "777" and primary group "project_x_group" for all
#' # files added to the storage directory
#' dvs_init("/data/project_x", 777, "project_x_group")
#' }
#'
#' @return A data frame with the storage directory, permissions, and group
#' @export
dvs_init <- function(storage_directory,
                     permissions = NULL,
                     group = NULL) {
  storage_directory <- normalizePath(storage_directory, mustWork = FALSE)
  val_or_err <- dvs_init_impl(storage_directory, permissions, group)
  if (inherits(val_or_err, "extendr_error")) {
    rlang::abort(val_or_err$value, class = "dvs_init_error")
  }
  return(val_or_err)
}
