test_that("init works first run", {
  proj_directory <- file.path(tempdir(), "test_init")
  storage_dir <- file.path(tempdir(), "test_init_storage_dir")
  create_fake_git_repo(proj_directory)
  withr::defer(print(sprintf("cleaning up dir: %s", proj_directory)))
  withr::defer(unlink(proj_directory, recursive = TRUE))
  # run init

  withr::with_dir(proj_directory, {
    expect_false(dir.exists(storage_dir))
    actual_df <- dvs_init(storage_dir)
    withr::defer(fs::dir_delete(storage_dir))
    # check storage_dir created
    expect_true(dir.exists(storage_dir))

    # check yaml created
    expect_true(file.exists("dvs.yaml"))

    # Check dvs_init output
    default_perms <- 664
    expected_df <- data.frame(storage_directory = storage_dir,
                              file_permissions = default_perms,
                              group = "")

    expect_equal(actual_df, expected_df)
  })

  # TODO: check dvs.yaml contents
  #deserialized <- yaml::read_yaml(yaml)

})

