use crate::TestHarnessSummary;
use shev::{Color, Entry, EntryState, Graphic, TextBox};
use sodigy_compiler_test::{CompileAndRun, find_root};
use sodigy_fs_api::{exists, join3, join4, read_string};

pub fn render_cnr(entry: &Entry, entry_state: EntryState) -> Result<Vec<Graphic>, String> {
    let cnr: CompileAndRun = serde_json::from_str(entry.content.as_ref().unwrap()).unwrap();

    match entry_state {
        EntryState(0) => {
            let s = format!(
                "# stdout\n\n```\n{}\n```\n\n# stderr\n\n```\n{}\n```{}",
                cnr.stdout_colored,
                cnr.stderr_colored,
                if let Some(error) = &cnr.error { format!("\n\n# test error\n\n```\n{error}\n```") } else { String::new() },
            );
            let (s, colors) = apply_ansi_term_color(&s);
            Ok(TextBox::new(
                &s,
                16.0,
                Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                [20.0, 20.0, 1080.0, 2000.0],
            ).with_color_map(colors).render())
        },
        EntryState(1) => {
            let root = find_root().map_err(|e| format!("{e:?}"))?;
            let test_files_at = join4(&root, "tests", "log", "test_files").map_err(|e| format!("{e:?}"))?;
            let hash = &cnr.hash;
            let test_file_at = join3(
                &test_files_at,
                hash.get(0..2).ok_or(format!("Corrupted hash: {hash:?}"))?,
                hash.get(2..).ok_or(format!("Corrupted hash: {hash:?}"))?,
            ).map_err(|e| format!("{e:?}"))?;

            if exists(&test_file_at) {
                Ok(TextBox::new(
                    &read_string(&test_file_at).map_err(|e| format!("{e:?}"))?,
                    16.0,
                    Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                    [20.0, 20.0, 1080.0, 2000.0],
                ).render())
            }

            else {
                Err(format!("File not found: {test_file_at}\nTry running `create-index` command!"))
            }
        },
        _ => unreachable!(),
    }
}

pub fn render_harness(entry: &Entry, _: EntryState) -> Result<Vec<Graphic>, String> {
    let entry: TestHarnessSummary = serde_json::from_str(entry.content.as_ref().unwrap()).unwrap();
    Ok(TextBox::new(
        &format!(
            "crates: {}/{}\ncompile-and-run: {}/{}\ntested at: {}",
            entry.crates_pass,
            entry.crates_pass + entry.crates_fail,
            entry.cnr_pass,
            entry.cnr_pass + entry.cnr_fail,
            entry.started_at,
        ),
        16.0,
        Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        [30.0, 30.0, 840.0, 540.0],
    ).render())
}

#[derive(Clone, Copy)]
enum TermColorParseState {
    Text,
    Control,
}

fn apply_ansi_term_color(s: &str) -> (String, Vec<Color>) {
    let mut chars: Vec<char> = vec![];
    let mut colors = vec![];
    let mut curr_color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    let mut state = TermColorParseState::Text;
    let mut digits_buffer = vec![];

    for ch in s.chars() {
        match state {
            TermColorParseState::Text => match ch {
                '\u{1b}' => {
                    digits_buffer = vec![];
                    state = TermColorParseState::Control;
                },
                _ => {
                    chars.push(ch);
                    colors.push(curr_color);
                },
            },
            TermColorParseState::Control => match ch {
                '0'..='9' => {
                    digits_buffer.push(ch);
                },
                'm' => {
                    match digits_buffer.iter().collect::<String>().parse::<u32>() {
                        Ok(0) => {
                            curr_color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
                        },
                        Ok(31) => {
                            curr_color = Color { r: 0.9, g: 0.3, b: 0.3, a: 1.0 };
                        },
                        Ok(32) => {
                            curr_color = Color { r: 0.3, g: 0.9, b: 0.3, a: 1.0 };
                        },
                        Ok(33) => {
                            curr_color = Color { r: 0.9, g: 0.9, b: 0.3, a: 1.0 };
                        },
                        Ok(34) => {
                            curr_color = Color { r: 0.3, g: 0.3, b: 0.9, a: 1.0 };
                        },
                        _ => {},
                    };

                    state = TermColorParseState::Text;
                },
                _ => {},
            },
        }
    }

    (chars.iter().collect(), colors)
}
