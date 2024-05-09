test_that("init works first run", {
  expected_df <- data.frame(storage_directory = c("/data/dvs/testthat"),
                            file_permissions = c(664),
                            group = c(""))

  actual_df <- dvs_init("/data/dvs/testthat")

  expect_equal(expected_df, actual_df)

  # delete dvs.yaml and storage_dir
  file.remove("~/Projects/Rdevious/dvs.yaml")
  unlink("/data/dvs/testthat", recursive = TRUE)
})

init


test_that("init works with input perms and group", {
  expected_df <- data.frame(storage_directory = c("/data/dvs/testthat"),
                            file_permissions = c(777),
                            group = c("datascience"))

  actual_df <- dvs_init("/data/dvs/testthat", 777, "datascience")

  expect_equal(expected_df, actual_df)

  # delete dvs.yaml and storage_dir
  file.remove("~/Projects/Rdevious/dvs.yaml")
  unlink("/data/dvs/testthat", recursive=TRUE)
})

