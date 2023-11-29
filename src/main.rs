use sodigy::{compile_file, compile_input};
use sodigy_files::get_all_sdg;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "FULL");
    // tests

    compile_file("./samples/easy.sdg".to_string()).unwrap().print_results();

    return;

    compile_input("
        let korean = \"한글 테스트 하나둘 하나둘\" <> \"셋넷\";
    ".as_bytes().to_vec()).print_results();

    for file in get_all_sdg(
        "./samples/err", false, "in"
    ).unwrap().iter().chain(
        get_all_sdg("./samples", true, "sdg").unwrap().iter()
    ) {
        compile_file(file.to_string()).unwrap().print_results();
    }
}
