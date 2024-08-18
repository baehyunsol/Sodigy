use crate::SAVE_IRS_AT;
use log::info;
use sodigy_files::{
    basename,
    is_dir,
    read_dir,
    remove_dir_all,
    FileErrorContext,
};
use sodigy_output::CompilerOutput;

// TODO: no more `--clean`, so remove this func
pub fn clean_irs(path: &str, compiler_output: &mut CompilerOutput, count: &mut usize) {
    info!("sodigy::clean_irs() path: {path:?}");

    compiler_output.show_overall_result = true;

    if let Ok(contents) = read_dir(path) {
        for content in contents.iter() {
            if is_dir(content) {
                match basename(content) {
                    Ok(c) if c == SAVE_IRS_AT => {
                        if let Err(mut e) = remove_dir_all(&content) {
                            compiler_output.push_error(e.set_context(FileErrorContext::CleaningIr).to_owned().into());
                        }

                        else {
                            *count += 1;
                        }
                    },
                    Ok(_) => {
                        clean_irs(content, compiler_output, count);
                    },
                    Err(e) => {
                        compiler_output.push_error(e.into());
                    },
                }
            }
        }
    }
}
