use super::{CnrContext, CompileAndRun, Status};
use sodigy_fs_api::{
    WriteMode,
    basename,
    join,
    parent,
    read_dir,
    read_bytes,
    remove_file,
    write_bytes,
};

impl CnrContext {
    pub fn incremental_compilation_test(&self, result: &CompileAndRun) -> Result<(), String> {
        self.clean()?;

        // TODO: turn on incremental compilation
        //       as of now, there's no flag that enable/disable incremental compilation
        //       and it's enabled by default. I want to add an explicity flag.

        // It chooses 2 files in `src/`.
        // It tests the incremental compilation by updating the 2 files and compiling it again.
        let src = join(&self.project_dir, "src").map_err(|e| format!("{e:?}"))?;
        let mut lib_file = None;
        let mut test_file_1 = None;
        let mut test_file_2 = None;

        for file in read_dir(&src, true).map_err(|e| format!("{e:?}"))? {
            if basename(&file).map_err(|e| format!("{e:?}"))? == "lib.sdg" {
                lib_file = Some(file.to_string());
            }

            else if file.ends_with(".sdg") {
                match (&test_file_1, &test_file_2) {
                    (None, _) => {
                        test_file_1 = Some(file.to_string());
                        continue;
                    },
                    (_, None) => {
                        test_file_2 = Some(file.to_string());
                        break;
                    },
                    _ => {},
                }
            }
        }

        let (lib_file, test_file_1, test_file_2) = match (lib_file, test_file_1, test_file_2) {
            (Some(f1), Some(f2), Some(f3)) => (f1, f2, f3),
            _ => {
                // We don't have to run incremental_compilation_test for every cases.
                // There must be other cases that are suitable to test.
                return Ok(());
            },
        };
        let tmp_file = join(
            &parent(&test_file_1).map_err(|e| format!("{e:?}"))?,
            "a_very_long_name_that_is_not_likely_to_be_used_by_the_test_case.sdg",
        ).map_err(|e| format!("{e:?}"))?;
        let lib_file_content = read_bytes(&lib_file).map_err(|e| format!("{e:?}"))?;
        let test_file_1_content = read_bytes(&test_file_1).map_err(|e| format!("{e:?}"))?;
        let test_file_2_content = read_bytes(&test_file_2).map_err(|e| format!("{e:?}"))?;

        // step 1. remove `test_file_1` and run
        //
        // It should fail to compile, but should create caches for the other files.
        remove_file(&test_file_1).map_err(|e| format!("{e:?}"))?;
        self.run_sodigy(Status::CompileFail)?;

        // step 2. restore `test_file_1` and run
        //
        // It should compile, and parsing stages for files other than `test_file_1`
        // must be skipped.
        write_bytes(
            &test_file_1,
            &test_file_1_content,
            WriteMode::AlwaysCreate,
        ).map_err(|e| format!("{e:?}"))?;
        let run_result = self.run_sodigy(Status::RunPass)?;

        // TODO: make sure that
        //   1. parsing `lib_file` is skipped
        //   2. parsing `test_file_1` is not skipped
        //   3. parsing `test_file_2` is skipped

        // step 3. add erroneous statements to `test_file_2` and run
        write_bytes(
            &test_file_2,
            // name collision error in hir
            b"\n\nlet x = 100; let x = 100;",
            WriteMode::AlwaysAppend,
        ).map_err(|e| format!("{e:?}"))?;
        let run_result = self.run_sodigy(Status::CompileFail)?;

        // TODO: make sure that
        //   1. parsing `lib_file` is skipped
        //   2. parsing `test_file_1` is skipped
        //   3. parsing `test_file_2` is not skipped

        // step 4. don't touch anything and run again
        //
        // Everything has to be cached.
        // Step 3 and step 4 must emit the same error.
        let run_result = self.run_sodigy(Status::CompileFail)?;

        // TODO: make sure that
        //   1. parsing `lib_file` is skipped
        //   2. parsing `test_file_1` is skipped
        //   3. parsing `test_file_2` is skipped

        // TODO: make sure that step 3 and step 4 emitted the same errors

        // step 5. make everything back to normal and run
        write_bytes(
            &test_file_2,
            &test_file_2_content,
            WriteMode::CreateOrTruncate,
        ).map_err(|e| format!("{e:?}"))?;
        let run_result = self.run_sodigy(Status::RunPass)?;

        // TODO: make sure that
        //   1. parsing `lib_file` is skipped
        //   2. parsing `test_file_1` is skipped
        //   3. parsing `test_file_2` is not skipped

        // step 6. don't touch anything and run again
        let run_result = self.run_sodigy(Status::RunPass)?;

        // TODO: make sure that
        //   1. parsing `lib_file` is skipped
        //   2. parsing `test_file_1` is skipped
        //   3. parsing `test_file_2` is skipped

        // step 7. add a new file (with no errors) and run
        write_bytes(
            &tmp_file,
            b"let a_very_long_name_that_is_not_likely_to_be_used_by_the_test_case = 100;",
            WriteMode::AlwaysCreate,
        ).map_err(|e| format!("{e:?}"))?;
        write_bytes(
            &lib_file,
            b"\n\nmod a_very_long_name_that_is_not_likely_to_be_used_by_the_test_case;",
            WriteMode::AlwaysAppend,
        ).map_err(|e| format!("{e:?}"))?;
        let run_result = self.run_sodigy(Status::RunPass)?;

        // TODO: make sure that
        //   1. parsing `lib_file` is not skipped
        //   2. parsing `test_file_1` is skipped
        //   3. parsing `test_file_2` is skipped
        //   4. parsing `tmp_file` is not skipped

        write_bytes(
            &lib_file,
            &lib_file_content,
            WriteMode::CreateOrTruncate,
        ).map_err(|e| format!("{e:?}"))?;

        Ok(())
    }
}
