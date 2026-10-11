use super::lower_to_single_module;
use sodigy_error::{Error, ErrorKind};
use sodigy_fs_api::{
    WriteMode,
    exists,
    join,
    join3,
    write_string,
};
use sodigy_object_file::ObjectFile;
use sodigy_subprocess as subprocess;

// TODO: I'm not sure if these unwraps are okay...
//       My current policy is "compiler errors are for errors in the
//       user code", so these file system errors are just ICE.
//       But, still, there are too many unwraps!
//       By the way, if `cargo` or `rustc` is missing in the user's
//       machine, that's a compile error because that's the user's
//       fault!
pub fn lower_to_crates(
    object_file: ObjectFile,
    intermediate_dir: &str,
    errors: &mut Vec<Error>,
) -> Result<String, ()> {  // when successful, it returns the path of the top-level crate
    if let Err(e) = check_dependencies() {
        errors.push(e);
        return Err(());
    }

    let crate_at = join(intermediate_dir, "rust-crate").unwrap();

    if !exists(&crate_at) {
        subprocess::run(
            "cargo",
            &["new", "rust-crate"],
            intermediate_dir,
            5.0,
            false,
            true,
        ).unwrap();
    }

    // TODO: I want to split it into multiple crates, so that
    //       1. The sodigy compiler can run in parallel
    //       2. Cargo can run in parallel
    //       3. Cargo can benefit from incremental compilation
    //       But I'm too lazy to implement that...
    let module = lower_to_single_module(object_file).to_string();
    let src = join3(&crate_at, "src", "main.rs").unwrap();
    write_string(
        &src,
        &module,
        WriteMode::CreateOrTruncate,
    ).unwrap();
    Ok(crate_at)
}

fn check_dependencies() -> Result<(), Error> {
    if let Err(_) = subprocess::run(
        "cargo",
        &["--version"],
        ".",
        3.0,
        false,
        true,
    ) {
        Err(Error {
            kind: ErrorKind::DependencyNotInstalled {
                dependency: String::from("cargo"),
            },
            spans: vec![],
            note: None,
        })
    } else if let Err(_) = subprocess::run(
        "rustc",
        &["--version"],
        ".",
        3.0,
        false,
        true,
    ) {
        Err(Error {
            kind: ErrorKind::DependencyNotInstalled {
                dependency: String::from("rustc"),
            },
            spans: vec![],
            note: None,
        })
    } else { Ok(()) }
}

