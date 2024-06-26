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
                              permissions = default_perms,
                              group = "")

    expect_equal(actual_df, expected_df)
  })

})

test_that("init works second run with same inputs", {
 dvs <- create_project_and_initialize_dvs("init_second_run", parent.frame())
  withr::with_dir(dvs$proj_dir, {
    # check dvs.yaml exists
    expect_true(file.exists("dvs.yaml"))
    # get yaml modification time
    yaml_time1 <- file.info("dvs.yaml")$mtime
    # check stor_dir exists
    expect_true(dir.exists(dvs$stor_dir))
    # run dvs_init
    actual_df <-  dvs_init(dvs$stor_dir)
    # check yaml not modified
    yaml_time2 <- file.info("dvs.yaml")$mtime
    expect_equal(yaml_time1, yaml_time2)

    # Check dvs_init output
    default_perms <- 664
    expected_df <- data.frame(storage_directory = dvs$stor_dir,
                              permissions = default_perms,
                              group = "")

    expect_equal(actual_df, expected_df)
  })
})

test_that("init doesn't work second run with different attributes", {
  dvs <- create_project_and_initialize_dvs("init_second_run", parent.frame())
  withr::with_dir(dvs$proj_dir, {
    # already inited
    expect_true(file.exists("dvs.yaml"))

    # try to init again with ONLY different stor_dir
    new_stor_dir <- file.path(tempdir(), "data/dvs", "try_new_stor_dir")
    expect_error(dvs_init(new_stor_dir), "user function panicked")

    # try again, but this time ONLY change group
    expect_error(dvs_init(dvs$stor_dir, group = "rstudio-superuser-admins"), "user function panicked")

    # try again, but this time ONLY change perms
    expect_error(dvs_init(dvs$stor_dir, permissions = 777), "user function panicked")

    # try again, this time don't change anything
    dvs_init(dvs$stor_dir)
  })
})

test_that("init works with a storage_dir that already exists", {
  proj_name <- "stor_dir_exists"
  proj_dir <- create_project(proj_name)

  # run proj_dir
  withr::with_dir(proj_dir, {
    # create stor_dir
    stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
    fs::dir_create(stor_dir)
    withr::defer(fs::dir_delete(stor_dir))
    expect_true(dir.exists(stor_dir))

    # get stor_dir perms
    stor_dir_perms1 <- file.info(stor_dir)

    # run dvs_init
    actual_df <- dvs_init(stor_dir)

    # check stor_dir created
    expect_true(dir.exists(stor_dir))

    # check yaml created
    expect_true(file.exists("dvs.yaml"))

    # check dvs_init output
    default_perms <- 664
    expected_df <- data.frame(storage_directory = stor_dir,
                              permissions = default_perms,
                              group = "")

    expect_equal(actual_df, expected_df)

    # check stor_dir perms didn't change
    stor_dir_perms2 <- file.info(stor_dir)
    expect_equal(stor_dir_perms1, stor_dir_perms2)

    # try to init again with ONLY different stor_dir
    new_stor_dir <- file.path(tempdir(), "data/dvs", "try_new_stor_dir2")
    expect_error(dvs_init(new_stor_dir), "user function panicked")

    # try again, but this time ONLY change group
    expect_error(dvs_init(stor_dir, group = "rstudio-superuser-admins"), "user function panicked")

    # try again, but this time ONLY change perms
    expect_error(dvs_init(stor_dir, permissions = 777), "user function panicked")

    # try again, this time don't change anything
    dvs_init(stor_dir)
  })
})

test_that("init doesn’t work when not in a git repo", {
  proj_name <- "no-git-repo"
  proj_dir <- fs::dir_create(file.path(tempdir(), proj_name))
  withr::defer(fs::dir_delete(proj_dir), envir = parent.frame())
  stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
  withr::with_dir(proj_dir, {
    expect_error(dvs_init(stor_dir), "user function panicked")
  })

})


test_that("init works no defaults", {
  proj_name <- "no-defaults"
  proj_dir <- create_project(proj_name)

  # run proj_dir
  withr::with_dir(proj_dir, {
    # check stor_dir doesn't exist
    stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
    expect_false(dir.exists(stor_dir))

    # run dvs_init
    perms <- 776
    group <- "datascience"
    actual_df <- dvs_init(stor_dir, perms, group)
    withr::defer(fs::dir_delete(stor_dir))

    # check stor_dir created
    expect_true(dir.exists(stor_dir))

    # check yaml created
    expect_true(file.exists("dvs.yaml"))

    # Check dvs_init output
    expected_df <- data.frame(storage_directory = stor_dir,
                              permissions = perms,
                              group = group)

    expect_equal(actual_df, expected_df)
  })

})

