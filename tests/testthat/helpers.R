create_fake_git_repo <- function(dir, env) {
  git_dir <- file.path(dir, ".git")
  print(paste("git_dir", git_dir))
  fs::dir_create(git_dir)
}

# input: project name
# function: creates fake git repo
# output: project directory
create_project <- function(proj_name) {
  proj_dir <- file.path(tempdir(), proj_name)
  create_fake_git_repo(proj_dir)
  proj_dir
}

# input: project name
# function: creates project and initializes dvs
# output: project directory and storage directory
create_project_and_initialize_dvs <- function(proj_name, env) {
  proj_dir <- create_project(proj_name)
  stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
  withr::defer(print(sprintf("cleaning up proj_dir: %s", proj_dir)), envir = env)
  withr::defer(fs::dir_delete(proj_dir), envir = env)
  withr::with_dir(proj_dir, {
    dvs_init(stor_dir)
    withr::defer(fs::dir_delete(stor_dir), envir = env)
  })

  list(proj_dir = proj_dir, stor_dir = stor_dir)
}

