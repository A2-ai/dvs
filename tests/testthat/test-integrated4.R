test_that("get unsynced [INT-GET-002]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("update", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # create file
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(file)

    # set file contents
    writeLines("This is an original line.", file)

    # add file
    added_file1 <- dvs_add(file)
    # check that data frame added it
    expect_equal(added_file1$outcome, "copied")

    # check that status is current
    status <- dvs_status(file)
    expect_equal(status$status, "current")

    # update file contents
    file_content <- readLines(file)
    new_content <- c("This is an updated line.")
    writeLines("This is an updated line.", file)

    # check that file updated
    contents <- readLines(file)
    expect_true(stringr::str_detect(contents, "This is an updated line."))

    # check that status is unsynced
    status <- dvs_status(file)
    expect_equal(status$status, "unsynced")

    # get old version of file
    getted_file <- dvs_get(file)
    # check that outcome is copied
    expect_equal(getted_file$outcome, "copied")

    # check that contents doesn't have new line
    contents <- readLines(file)
    expect_false(stringr::str_detect(contents, "This is an updated line."))

    # check that status is current
    status2 <- dvs_status(file)
    expect_equal(status2$status, "current")
  })
})
