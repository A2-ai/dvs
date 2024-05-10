test_that("init works first run", {
  old_project <- getwd()
  # create project dir one level up
  proj_dir_parent <- tempdir()
  proj_dir <- file.path(proj_dir_parent, "proj_dir")
  dir.create(proj_dir)
  print(paste("proj_dir", proj_dir))
  withr::defer(fs::dir_delete(proj_dir), parent.frame())

  # make git dir (fake being in git repo)
  git_dir <- file.path(proj_dir, ".git")
  print(paste("git_dir", git_dir))
  dir.create(git_dir)
  #withr::defer(fs::dir_delete(git_dir), parent.frame())

  # set up dir for storage_dir
  storage_dir_parent <- tempdir()
  storage_dir <- file.path(storage_dir_parent, "storage_dir")
  print(paste("storage_dir", storage_dir))

  # change wd to temp proj dir
  setwd(proj_dir)
  withr::defer(setwd(old_project), envir = parent.frame())
  print(paste("current wd", getwd()))

  # run init
  actual_df <- dvs_init(storage_dir)
  withr::defer(fs::dir_delete(storage_dir), parent.frame())

  # check storage_dir exists
  expect_true(dir.exists(storage_dir))

  # check yaml exists
  yaml <- file.path(proj_dir, "dvs.yaml")
  expect_true(file.exists(yaml))

  default_perms <- 664
  expected_df <- data.frame(storage_directory = storage_dir,
                                        file_permissions = default_perms,
                                        group = "")

  expect_equal(actual_df, expected_df)
  expect_equal(yaml$permissions, 664)


})




# test_that("init works with input perms and group", {
#   expected_df <- data.frame(storage_directory = c("/data/dvs/testthat"),
#                             file_permissions = c(777),
#                             group = c("datascience"))
#
#   actual_df <- dvs_init("/data/dvs/testthat", 777, "datascience")
#
#   expect_equal(expected_df, actual_df)
#
#   # delete dvs.yaml and storage_dir
#   file.remove("~/Projects/Rdevious/dvs.yaml")
#   unlink("/data/dvs/testthat", recursive=TRUE)
# })

