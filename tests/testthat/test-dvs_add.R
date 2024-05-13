test_that("can add a single file", {

  dvs <- create_project_and_initialize_real_repo("add_single_file", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  data_derived_dir <- file.path(tempdir(), "projects/add_single_file/data/derived")
  fs::dir_create(data_derived_dir)

  #check data directory exists
  expect_true(dir.exists(data_derived_dir))

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add(pk_data_path, message = "finished pk data assembly")
  })

  # check that metadata file exists
  dvs_file_path <- file.path(dvs$proj_dir, "data/derived/pk_data.csv.dvs")
  dvs_json <- jsonlite::fromJSON(dvs_file_path)

  expect_true(file.exists(dvs_file_path))

  # check that it was added recently (not necessary?)
  expect_true(is_near_time(dvs_json$time_stamp))

  # check that a file was added in the stor_dir, but no equality check

  first_two_of_hash <- substring(dvs_json$blake3_checksum, 1, 2)
  rest_of_hash <- substring(dvs_json$blake3_checksum, 3)

  expect_true(file.exists(file.path(dvs$stor_dir, first_two_of_hash, rest_of_hash)))

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })


  # print("")
  # print(sprintf("tempdir(): %s", tempdir()))
  # print(sprintf("proj_dir: %s", dvs$proj_dir))
  # print(sprintf("stor_dir: %s", dvs$stor_dir))
  # print(sprintf("data_derived_dir: %s", data_derived_dir))

})
