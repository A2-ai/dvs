create_fake_git_repo <- function(dir, env) {
  git_dir <- file.path(dir, ".git")
  print(paste("git_dir", git_dir))
  fs::dir_create(git_dir)
}

create_dvs_project <- function(dir, proj_name, env) {
  proj_directory <- file.path(tempdir(), proj_name)
  storage_dir <- file.path(tempdir(), sprintf("%s_storage_dir", proj_name))
  create_fake_git_repo(proj_directory)
  withr::defer(print(sprintf("cleaning up dir: %s", proj_directory)), envir = env)
  withr::defer(unlink(proj_directory, recursive = TRUE), environ = env)
  withr::with_dir(proj_directory, {
    dvs_init(storage_dir)
  })
}
