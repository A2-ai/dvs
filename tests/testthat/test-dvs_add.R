test_that("can add a single file", {

  dvs <- create_project_and_initialize_real_repo("add_single_file", parent.frame())

  expect_true(dir.exists(dvs$proj_dir))
  expect_true(dir.exists(dvs$stor_dir))
  expect_true(dir.exists(file.path(dvs$proj_dir, ".git")))

  derived_dir <- file.path(dvs$proj_dir, "data/derived")

  fs::dir_create(derived_dir)

  expect_true(dir.exists(derived_dir))



  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  pk_data_path <- file.path(derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add(pk_data_path, message = "finished pk data assembly")
  })

  # added_files <- dvs_add(pk_data_path, message = "finished pk data assembly")
  # expect_equal(1,1)

  print("")
  print(dvs$proj_dir)
  print(dvs$stor_dir)
  print(derived_dir)
  print(pk_data_path)
  print(added_files)
  withr::with_dir(dvs$stor_dir, {
    print(sprintf("Listing files in %s", dvs$stor_dir))
    print(list.files())
  })
})
