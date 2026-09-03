#[derive(Clone, Copy)]
enum AnsiParseState {
    Text,
    Escape,
}

pub fn remove_ansi_characters(s: &str) -> String {
    let mut state = AnsiParseState::Text;
    let mut result = vec![];

    for ch in s.chars() {
        match state {
            AnsiParseState::Text => match ch {
                '\x1b' => {
                    state = AnsiParseState::Escape;
                },
                _ => {
                    result.push(ch);
                },
            },
            AnsiParseState::Escape => match ch {
                'm' => {
                    state = AnsiParseState::Text;
                },
                _ => {},
            },
        }
    }

    result.iter().collect()
}
