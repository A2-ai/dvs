# proj_dir
# stor_dir

test_that("init works first run", {
  proj_name <- "first-run-init"
  proj_dir <- create_project(proj_name)

  # run proj_dir
  withr::with_dir(proj_dir, {
    # check stor_dir doesn't exist
    stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
    expect_false(dir.exists(stor_dir))

    # run dvs_init
    actual_df <- dvs_init(stor_dir)
    withr::defer(fs::dir_delete(stor_dir))

    # check stor_dir created
    expect_true(dir.exists(stor_dir))

    # check yaml created
    expect_true(file.exists("dvs.yaml"))

    # Check dvs_init output
    default_perms <- 664
    expected_df <- data.frame(storage_directory = stor_dir,
                              file_permissions = default_perms,
                              group = "")

    expect_equal(actual_df, expected_df)
  })

})

test_that("init doesn't work second run", {
 dvs <- create_project_and_initialize_dvs("init_second_run", parent.frame())
  withr::with_dir(dvs$proj_dir, {
    expect_true(file.exists("dvs.yaml"))
    expect_error(dvs_init(dvs$stor_dir), "user function panicked")
  })
})




