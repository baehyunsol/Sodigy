use crate::SAVE_IRS_AT;
use crate::result::CompilerOutput;
use sodigy_files::{
    basename,
    create_dir_all,
    exists,
    is_dir,
    parent,
    read_dir,
    remove_dir_all,
    FileError,
    FileErrorContext,
};
use sodigy_error::UniversalError;

pub fn clean_irs(path: &str, compiler_output: &mut CompilerOutput) {
    if let Ok(contents) = read_dir(path) {
        for content in contents.iter() {
            if is_dir(content) {
                match basename(content) {
                    Ok(c) if c == SAVE_IRS_AT => {
                        if let Err(mut e) = remove_dir_all(&content) {
                            compiler_output.push_error(e.set_context(FileErrorContext::CleaningIr).to_owned().into());
                        }
                    },
                    Ok(_) => {
                        clean_irs(content, compiler_output);
                    },
                    Err(e) => {
                        compiler_output.push_error(e.into());
                    },
                }
            }
        }
    }
}

pub fn try_make_intermediate_paths(
    is_file: bool, path: &String,
) -> Result<(), UniversalError> {
    if exists(path) {
        if is_dir(path) {
            // we have to make a file named X,
            // but there exists a dir named X
            if is_file {
                Err(FileError::cannot_create_file(true /* there exists a dir */, path).into())
            }

            // we have to make a dir named X,
            // and it's already there
            else {
                Ok(())
            }
        }

        // this branch is for files
        // the compiler ignores anything other than dirs and files
        else {
            // we have to make a file named X,
            // and it's already there
            // we'll truncate it
            if is_file {
                Ok(())
            }

            // we have to make a dir named X,
            // but there exists a file named X
            else {
                Err(FileError::cannot_create_file(false /* there exists a file */, path).into())
            }
        }
    }

    else {
        let dir_to_create = if is_file {
            parent(path).map_err(|e| UniversalError::from(e))?
        } else {
            path.to_string()
        };

        create_dir_all(&dir_to_create).map_err(|e| e.into())
    }
}
