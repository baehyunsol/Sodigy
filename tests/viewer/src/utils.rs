pub fn circle(color: &str, size: &str) -> String {
    format!(r#"<span class="circle-{color} circle-{size}"></span>"#)
}

pub fn render_elapsed_ms(ms: u64) -> String {
    match ms {
        0..1000 => format!("0.{ms:03}s"),
        1_000..20_000 => format!("{}.{:03}s", ms / 1000, ms % 1000),
        20_000..60_000 => format!("{}s", ms / 1000),
        60_000.. => format!("{}m {}s", ms / 60_000, ms / 1_000 % 60),
    }
}

const STYLE: &str = include_str!("style.css");

pub fn html_template(body: &str, show_top_bar: bool) -> String {
    let top_bar = if show_top_bar {
        String::from(r#"<p><a href="../index.html">Home</a></p>"#)
    } else {
        String::new()
    };

    format!(r#"
<!DOCTYPE html>
<html>

<head>
<style>
{STYLE}
</style>
</head>

<body>
{top_bar}

{body}
</body>

</html>
"#)
}

// TODO: these (escape_html, apply_ansi_term_color) are direct copy-paste from crates/driver/src/log.rs
//       I want an independent crate like `html-render`, but I'm not sure if that's worth it
pub fn escape_html(s: &str) -> String {
    let s = s
        .replace("&", "&amp;")
        .replace(">", "&gt;")
        .replace("<", "&lt;");

    apply_ansi_term_color(&s)
}

#[derive(Clone, Copy)]
enum TermColorParseState {
    Text,
    Control,
}

fn apply_ansi_term_color(s: &str) -> String {
    let mut state = TermColorParseState::Text;
    let mut content_buffer: Vec<char> = vec![];
    let mut digits_buffer: Vec<char> = vec![];
    let mut result: Vec<String> = vec![String::from("<span>")];

    for ch in s.chars() {
        match state {
            TermColorParseState::Text => match ch {
                '\u{1b}' => {
                    digits_buffer = vec![];
                    result.push(content_buffer.drain(..).collect());
                    result.push(String::from("</span>"));
                    state = TermColorParseState::Control;
                },
                _ => {
                    content_buffer.push(ch);
                },
            },
            TermColorParseState::Control => match ch {
                '0'..='9' => {
                    digits_buffer.push(ch);
                },
                'm' => {
                    match digits_buffer.iter().collect::<String>().parse::<u32>() {
                        Ok(0) => {
                            result.push(String::from(r#"<span>"#));
                        },
                        Ok(31) => {
                            result.push(String::from(r#"<span class="red">"#));
                        },
                        Ok(32) => {
                            result.push(String::from(r#"<span class="green">"#));
                        },
                        Ok(33) => {
                            result.push(String::from(r#"<span class="yellow">"#));
                        },
                        Ok(34) => {
                            result.push(String::from(r#"<span class="blue">"#));
                        },
                        _ => unreachable!(),
                    };

                    state = TermColorParseState::Text;
                },
                _ => {},
            },
        }
    }

    if !content_buffer.is_empty() {
        result.push(content_buffer.drain(..).collect());
        result.push(String::from("</span>"));
    }

    result.concat()
}

pub fn color_udiff(s: &str) -> String {
    let mut colored = vec![];

    for line in s.lines() {
        if line.starts_with("+") {
            colored.push(format!("\x1b[32m{line}\x1b[0m"));
        }

        else if line.starts_with("-") {
            colored.push(format!("\x1b[31m{line}\x1b[0m"));
        }

        else if line.starts_with("@") {
            colored.push(format!("\x1b[33m{line}\x1b[0m"));
        }

        else {
            colored.push(line.to_string());
        }
    }

    colored.join("\n")
}
