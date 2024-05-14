test_that("can add a single file", {

  dvs <- create_project_and_initialize_real_repo("add_single_file", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
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
  dvs_file_path <- file.path(data_derived_dir, "pk_data.csv.dvs")
  dvs_json <- jsonlite::fromJSON(dvs_file_path)

  expect_true(file.exists(dvs_file_path))

  # check that it was added recently (not necessary?)
  expect_true(is_near_time(dvs_json$time_stamp))

  #check that git ingore is created
  expect_true(file.exists(file.path(data_derived_dir, ".gitignore")))

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

test_that("can add multiple files - same directory", {
  dvs <- create_project_and_initialize_real_repo("add_multiple_file", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  #check data directory exists
  expect_true(dir.exists(data_derived_dir))

  # create two data files for testing
  pk_data_1 <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  pk_data_2 <- data.frame(
    USUBJID = c(2, 2, 2),
    NTFD = c(0.4, 2, 3),
    DV = c(359.44, 540.213, 1)
  )

  pk_data_path_1 <- file.path(data_derived_dir, "pk_data_1.csv")
  pk_data_path_2 <- file.path(data_derived_dir, "pk_data_2.csv")
  write.csv(pk_data_1, pk_data_path_1)
  write.csv(pk_data_2, pk_data_path_2)

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add("data/derived/*", message = "finished pk data assembly")
  })

  # check that metadata files exist
  dvs_file_path_1 <- file.path(data_derived_dir, "pk_data_1.csv.dvs")
  dvs_file_path_2 <- file.path(data_derived_dir, "pk_data_2.csv.dvs")
  dvs_json_1 <- jsonlite::fromJSON(dvs_file_path_1)
  dvs_json_2 <- jsonlite::fromJSON(dvs_file_path_2)

  expect_true(file.exists(dvs_file_path_1))
  expect_true(file.exists(dvs_file_path_2))


  #check that git ingore is created
  expect_true(file.exists(file.path(data_derived_dir, ".gitignore")))

  # check that both files were added in the stor_dir, but no equality check
  first_two_of_hash_1 <- substring(dvs_json_1$blake3_checksum, 1, 2)
  first_two_of_hash_2 <- substring(dvs_json_2$blake3_checksum, 1, 2)
  rest_of_hash_1 <- substring(dvs_json_1$blake3_checksum, 3)
  rest_of_hash_2 <- substring(dvs_json_2$blake3_checksum, 3)

  expect_true(file.exists(file.path(dvs$stor_dir, first_two_of_hash_1, rest_of_hash_1)))
  expect_true(file.exists(file.path(dvs$stor_dir, first_two_of_hash_2, rest_of_hash_2)))

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })
})

test_that("can add two files in different directories", {
  dvs <- create_project_and_initialize_real_repo("add_two_diff_directories", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  #check data directory exists
  expect_true(dir.exists(data_derived_dir))

  # create two data files for testing
  pk_data_1 <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  pk_data_2 <- data.frame(
    USUBJID = c(2, 2, 2),
    NTFD = c(0.4, 2, 3),
    DV = c(359.44, 540.213, 1)
  )

  #create other path
  new_path <- file.path(dvs$proj_dir, "model/nonmem/")
  fs::dir_create(new_path)
  expect_true(dir.exists(new_path))

  pk_data_path_1 <- file.path(data_derived_dir, "pk_data_1.csv")
  pk_data_path_2 <- file.path(new_path, "pk_data_2.csv")
  write.csv(pk_data_1, pk_data_path_1)
  write.csv(pk_data_2, pk_data_path_2)

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add(c(pk_data_path_1, pk_data_path_2), message = "finished pk data assembly")
  })

  # check that metadata files exist
  dvs_file_path_1 <- file.path(data_derived_dir, "pk_data_1.csv.dvs")
  dvs_file_path_2 <- file.path(dvs$proj_dir, "model/nonmem/pk_data_2.csv.dvs")
  dvs_json_1 <- jsonlite::fromJSON(dvs_file_path_1)
  dvs_json_2 <- jsonlite::fromJSON(dvs_file_path_2)

  expect_true(file.exists(dvs_file_path_1))
  expect_true(file.exists(dvs_file_path_2))


  #check that git ignore is created
  expect_true(file.exists(file.path(data_derived_dir, ".gitignore")))

  # check that both files were added in the stor_dir, but no equality check
  first_two_of_hash_1 <- substring(dvs_json_1$blake3_checksum, 1, 2)
  first_two_of_hash_2 <- substring(dvs_json_2$blake3_checksum, 1, 2)
  rest_of_hash_1 <- substring(dvs_json_1$blake3_checksum, 3)
  rest_of_hash_2 <- substring(dvs_json_2$blake3_checksum, 3)

  expect_true(file.exists(file.path(dvs$stor_dir, first_two_of_hash_1, rest_of_hash_1)))
  expect_true(file.exists(file.path(dvs$stor_dir, first_two_of_hash_2, rest_of_hash_2)))

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })
})

test_that("dvs_add filters .dvs and .gitignore files", {
  dvs <- create_project_and_initialize_real_repo("no_add_meta_gitignore", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  #check data directory exists
  expect_true(dir.exists(data_derived_dir))

  # create two data files for testing
  pk_data_1 <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  pk_data_2 <- data.frame(
    USUBJID = c(2, 2, 2),
    NTFD = c(0.4, 2, 3),
    DV = c(359.44, 540.213, 1)
  )

  pk_data_path_1 <- file.path(data_derived_dir, "pk_data_1.csv")
  pk_data_path_2 <- file.path(data_derived_dir, "pk_data_2.csv")
  write.csv(pk_data_1, pk_data_path_1)
  write.csv(pk_data_2, pk_data_path_2)

  # create artificial metadata files and a .gitignore before adding
  file.create(file.path(data_derived_dir, "pk_data_1.csv.dvs"))
  file.create(file.path(data_derived_dir, ".gitignore"))

  withr::with_dir(data_derived_dir, {
    expect_true(file.exists("pk_data_1.csv.dvs"))
    expect_true(file.exists(".gitignore"))
  })

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add("data/derived/*", message = "finished pk data assembly")
  })

  # check that only two subdirectories were created in the storage directory
  subdirectories <- list.dirs(dvs$stor_dir, full.names = TRUE, recursive = FALSE)
  expect_equal(length(subdirectories), 2)

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })
})

test_that("errors when file DNE", {

  dvs <- create_project_and_initialize_real_repo("file_dne", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  #check data directory exists
  expect_true(dir.exists(data_derived_dir))

  # try to add a file that doesn't exist
  withr::with_dir(dvs$proj_dir, {
    expect_error(dvs_add(file.path(data_derived_dir, "no_such_file.csv"), message = "finished pk data assembly"))
  })

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })

})

test_that("errors when not in a git repo", {

  dvs <- create_project_and_initialize_real_repo("no_repo", parent.frame())

  # check that directories exist after dvs_init
  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  # data directory NOT in a git repo
  data_derived_dir <- file.path(tempdir(), "no_repo/data/derived")
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

  # add a file
  withr::with_dir(tempdir(), {
    expect_error(dvs_add(pk_data_path, message = "finished pk data assembly"))
  })

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })

})

test_that("errors when not initialized", {

  proj_dir <- create_project_no_dvs_init("no_init", parent.frame())
  expect_true(dir.exists(proj_dir))
  expect_true(dir.exists(file.path(proj_dir, ".git")))

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

  expect_true(file.exists(pk_data_path))

  # dvs_add
  withr::with_dir(proj_dir, {
    expect_error(dvs_add(pk_data_path, message = "finished pk data assembly"))
  })

  # withr::with_dir(tempdir(), {
  #   fs::dir_tree(all = TRUE)
  # })
})
