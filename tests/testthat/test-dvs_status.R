test_that("dvs status shows ....", {
  proj_directory <- file.path(tempdir(), "test_status")
  storage_dir <- file.path(tempdir(), "test_status_storage_dir")
  create_fake_git_repo(proj_directory)
  withr::defer(print(sprintf("cleaning up dir: %s", proj_directory)))
  withr::defer(unlink(proj_directory, recursive = TRUE))
  create_dvs_project("test_status", parent.frame())
  # run init

  withr::with_dir(proj_directory, {
    status <- dvs_status()
    expect_equal(nrow(status), 0)
  })

  # TODO: check dvs.yaml contents
  #deserialized <- yaml::read_yaml(yaml)

})

test_that("dvs status shows ....", {

  create_dvs_project("test_status2")
  withr::with_dir(proj_directory, {
    dvs_add(...)
    status <- dvs_status()
    expect_equal(nrow(status), 1)
  })

  # TODO: check dvs.yaml contents
  #deserialized <- yaml::read_yaml(yaml)

})


