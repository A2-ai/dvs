# proj_dir
# stor_dir

test_that("init works first run [UNI-INI-001]", {
  proj_name <- "first-run-init"
  proj_dir <- create_project(proj_name)

  # run proj_dir
  withr::with_dir(proj_dir, {
    # check stor_dir doesn't exist
    stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
    expect_false(dir.exists(stor_dir))

    # start capturing output
    temp_file <- tempfile()
    temp_connection <- file(temp_file, open = "wt")
    sink(temp_connection)
    #sink(temp_connection, type = "message")
    #print(temp_file)
    #withr::defer(fs::file_delete(temp_file))

    # run dvs_init
    actual_df <- dvs_init(stor_dir)
    withr::defer(fs::dir_delete(stor_dir))

    # stop capturing output
    #sink(temp_connection)
    #sink(temp_connection, type = "message")

    #output <- readLines(temp_file)
    #print(glue::glue("HERE'S THE OUTPUT: {output}"))

    # check output
    #expect_true(any(stringr::str_detect(output, "storage directory doesn't exist")))
    #print(output)

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

test_that("init works second run with same inputs [UNI-INI-002]", {
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

test_that("init doesn't work second run with diff attributes [UNI-INI-003]", {
  dvs <- create_project_and_initialize_dvs("init_second_run", parent.frame())
  withr::with_dir(dvs$proj_dir, {
    # already inited
    expect_true(file.exists("dvs.yaml"))

    # try to init again with ONLY different stor_dir
    new_stor_dir <- file.path(tempdir(), "data/dvs", "try_new_stor_dir")
    expect_error(dvs_init(new_stor_dir), "project already initialized")

    # try again, but this time ONLY change group
    expect_error(dvs_init(dvs$stor_dir, group = "rstudio-superuser-admins"),
                 "project already initialized")

    # try again, but this time ONLY change perms
    expect_error(dvs_init(dvs$stor_dir, permissions = 777),
                 "project already initialized")

    # try again, this time don't change anything
    dvs_init(dvs$stor_dir)
  })
})

test_that("init works with a storage_dir that already exists [UNI-INI-004]", {
  proj_name <- "stor_dir_exists"
  proj_dir <- create_project(proj_name)
  expect_true(dir.exists(proj_dir))

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
    expect_error(dvs_init(new_stor_dir), "project already initialized")

    # try again, but this time ONLY change group
    expect_error(dvs_init(stor_dir, group = "rstudio-superuser-admins"),
                 "project already initialized")

    # try again, but this time ONLY change perms
    expect_error(dvs_init(stor_dir, permissions = 777),
                 "project already initialized")

    # try again, this time don't change anything
    dvs_init(stor_dir)
  })
})

test_that("init doesn’t work when not in a git repo [UNI-INI-005]", {
  proj_name <- "no-git-repo"
  proj_dir <- fs::dir_create(file.path(tempdir(), proj_name))
  withr::defer(fs::dir_delete(proj_dir), envir = parent.frame())
  stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
  withr::with_dir(proj_dir, {
    expect_error(dvs_init(stor_dir), "git repository not found: make sure you're in an active git repository. could not find git repo root; make sure you're in an active git repository:")
  })

})


test_that("init works no defaults [UNI-INI-006]", {
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

test_that("init works after updating inputs in yaml [UNI-INI-007]", {
  dvs <- create_project_and_initialize_dvs("init_update_yaml", parent.frame())
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


    # update yaml manually, check that init can be re-run with new attributes without error,
    # and outputted df reflects the update

    # create new storage dir and defer deletion
    new_stor_dir <- file.path(tempdir(), "new_stor_dir")
    fs::dir_create(new_stor_dir)
    withr::defer(fs::dir_delete(new_stor_dir), envir = parent.frame())

    new_perms <- 777
    new_group <- "rstudio-superuser-admins"

    yaml_data <- yaml::read_yaml("dvs.yaml")
    yaml_data$storage_dir <- new_stor_dir
    yaml_data$permissions <- new_perms
    yaml_data$group <- new_group

    yaml::write_yaml(yaml_data, "dvs.yaml")

    # run dvs_init
    new_actual_df <-  dvs_init(new_stor_dir, new_perms, new_group)

    new_expected_df <- data.frame(storage_directory = new_stor_dir,
                              permissions = new_perms,
                              group = new_group)

    expect_equal(new_actual_df, new_expected_df)

  })
})

test_that("Users are warned if the stor dir isn't empty [UNI-INI-008]", {
  #TODO
})

test_that("Users are warned if the stor dir has a file extension [UNI-INI-009]", {
  #TODO
})

test_that("Users are warned if the stor dir is in the proj dir [UNI-INI-010]", {
  #TODO
})

test_that("Users are told if the stor dir is created [UNI-INI-011]", {
  #TODO
})

test_that("Users are told if the stor dir is already exists [UNI-INI-012]", {
  #TODO
})

test_that("An error occurs if the absolute path of the stor dir can't be created [UNI-INI-013]", {
  #TODO
})

test_that("An error occurs if the config file (dvs.yaml) can't be created [UNI-INI-014]", {
  #TODO
})

test_that("An error occurs if the primary group is invalid [UNI-INI-015]", {
  #TODO
})

test_that("An error occurs if the linux permissions are invalid [UNI-INI-016]", {
  proj_name <- "UNI-INI-019"
  proj_dir <- create_project(proj_name)
  stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
  fs::dir_create(stor_dir)
  withr::defer(fs::dir_delete(stor_dir))
  withr::with_dir(proj_dir, {
    expect_error(dvs_init(stor_dir, permissions = 999), "linux file permissions invalid: linux permissions: 999 not valid. invalid digit found in string")
  })
})

test_that("An error occurs if the stor dir doesn't exist and is unable to be created [UNI-INI-017]", {
  proj_name <- "UNI-INI-017"
  proj_dir <- create_project(proj_name)
  parent_of_stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
  fs::dir_create(parent_of_stor_dir)
  withr::defer(fs::dir_delete(parent_of_stor_dir))
  # change perms for parent_of_stor_dir so stor dir can't be created
  Sys.chmod(parent_of_stor_dir, mode = "0555")
  withr::defer(Sys.chmod(parent_of_stor_dir, mode = "777"))

  withr::with_dir(proj_dir, {
    expect_error(dvs_init(file.path(parent_of_stor_dir, "test")), "storage directory not created")
  })
})

test_that("An error occurs if the stor dir is unable to be verified as empty or non-empty [UNI-INI-018]", {
  proj_name <- "UNI-INI-018"

  proj_dir <- create_project(proj_name)
  stor_dir <- file.path(tempdir(), sprintf("%s_stor_dir", proj_name))
  fs::dir_create(stor_dir)
  withr::defer(fs::dir_delete(stor_dir))

  # change perms so the stor dir is unable to be verified as empty or non-empty
  Sys.chmod(stor_dir, mode = "000")
  withr::defer(Sys.chmod(stor_dir, mode = "777"))

  withr::with_dir(proj_dir, {
    expect_error(dvs_init(stor_dir, permissions = 777), "could not check if storage directory is empty")
  })
})

test_that("An error occurs if the linux file permissions are unable to be set for the stor dir [UNI-INI-019]", {
  #NOTEST
  # I don't think I can do this because I don't have the linux perms
  #Note: this is not the same as invalid perms
})

test_that("If no input is given for the permissions, the default permissions are 664 and
          if no primary group is given, files copied to the stor dir inherit the primary group
          of the original [UNI-INI-020]", {
  dvs <- create_project_and_initialize_real_repo("UNI-INI-020", parent.frame())

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
  expect_true(is_near_time(dvs_json$add_time))

  #check that git ignore is created
  expect_true(file.exists(file.path(data_derived_dir, ".gitignore")))

  # check that a file was added in the stor_dir, but no equality check

  first_two_of_hash <- substring(dvs_json$blake3_checksum, 1, 2)
  rest_of_hash <- substring(dvs_json$blake3_checksum, 3)

  stored_file <- file.path(dvs$stor_dir, first_two_of_hash, rest_of_hash)
  expect_true(file.exists(stored_file))

  # check permissions of stored file
  info <- file.info(stored_file)
  mode <- info$mode
  def_mode <- as.octmode("664")
  expect_equal(mode, def_mode)

  group <- info$grname
  def_group <- "datascience"
  expect_equal(group, def_group)
})

