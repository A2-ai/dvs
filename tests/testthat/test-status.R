test_that("status works base case", {
  temp_dir <- "temp"
  dir.create(temp_dir)

  yaml::write_yaml(data.frame(
    storage_dir = normalizePath(temp_dir),
    permissions = as.integer(664),
    group = ""
    ),
    file.path(temp_dir, "dvs.yaml"))

  dir.create("temp/.git")




  # expected <- setNames(data.frame(matrix(ncol = 11, nrow = 0)), c("relative_path", "status", "file_size_bytes",
  #                                                                    "blake3_checksum", "time_stamp", "saved_by",
  #                                                                    "message", "absolute_path", "error",
  #                                                                    "error_message", "input"
  #                                                                    )) %>%
  #   dplyr::mutate_if(is.logical, as.character)
  #
  # expected$file_size_bytes <- as.numeric(expected$file_size_bytes)
  # dir.create(file.path(temp_dir, ".git"))
  # actual <- dvs_status()
  # expect_equal(actual, expected)
  #
  #
  #
  # unlink(temp_dir, recursive = TRUE)
})

test_that("status doesn't work when uninitialized", {
  expect_error(dvs_status(), "user function panicked")
})

test_that("status works for current file", {
  dir.create("storage_dir")

  dir.create("project_dir")
  setwd("project_dir")
  temp_dir <- getwd()

  dir.create(".git")

  yaml::write_yaml(data.frame(
    storage_dir = normalizePath(temp_dir),
    permissions = as.integer(664),
    group = ""
  ),
  "dvs.yaml")

  file <- file.create("test.txt")

  digest::digest(file = "test.txt", algo = "blake3")



  # setwd("..")
  # unlink("storage_dir", recursive = TRUE)
  # unlink("project_dir", recursive = TRUE)
})
#
# test_that("status works for absent file", {
#   dir.create("temp")
#   temp_dir <- setwd("temp")
#
#   dir.create(.git)
#
#   file.create("test.txt.dvs")
#
# })
#
# test_that("status works for unsynced file", {
#   dir.create("temp")
#   temp_dir <- setwd("temp")
#
#   dir.create(.git)
#
#   file.create("test.txt.dvs")
#
# })
#
# test_that("status works for non-existing file", {
#
# })
#
# test_that("status works for file glob", {
#
# })



