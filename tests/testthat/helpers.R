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

# input: character of ISO8601
near_system_time <- function(time_string) {
  # Convert the ISO8601 string to a POSIXct date-time object
  time_object <- as.POSIXct(time_string, tz = "UTC", format = "%Y-%m-%dT%H:%M:%OS")
  print(paste("inputted time:", time_object))

  # Get the current system time in the same time zone as the time object
  current_time <- Sys.time()
  print(paste("current time:", time_object))

  # Set the time zone to UTC for consistency
  attr(current_time, "tzone") <- "UTC"

  # Calculate the difference in seconds (or any other unit you prefer)
  time_difference <- difftime(current_time, time_object, units = "secs")
  print(paste("time_difference:", time_difference))

  # true if time difference < 1
  time_difference < 1
}


