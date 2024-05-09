test_that("dvs_init writes files", {

  # Temporary directory for testing
  dir.create("temp")
  setwd("temp")

  temp_dir <- getwd()

  dir.create(".git")

  dvs_init("dvs")

  dvs_yaml <- yaml::read_yaml("dvs.yaml")

  expected_yaml <- list(
    storage_dir = normalizePath("dvs"),
    permissions = 664,
    group = ""
  )

  expect_equal(dvs_yaml, expected_yaml)

  expect_true(dir.exists("dvs"))

  setwd("../")

  unlink(temp_dir, recursive = TRUE)
})

test_that("init doesn't work second run", {
  dir.create("temp")
  setwd("temp")
  temp_dir <- getwd()
  dir.create(".git")
  dir.create("dvs")

  yaml::write_yaml(list(), "dvs.yaml")

  expect_error(dvs_init("dvs"))

  setwd("..")

  unlink(temp_dir, recursive = TRUE)
})

#test_that("dvs_init error 1", )
