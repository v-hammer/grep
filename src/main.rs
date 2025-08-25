pub mod parser;

fn report_error(e: parser::ParseError, input: &str) {
    let pos = match e {
        parser::ParseError::DanglingQuantifier(_, p)
        | parser::ParseError::InvalidEscape(p)
        | parser::ParseError::LackEscape(p)
        | parser::ParseError::UnterminatedCharClass(p)
        | parser::ParseError::UnterminatedGroup(p) => p,
    };

    eprintln!("error: {}", e);
    eprintln!("       {}", input);
    let caret_pos = if pos <= input.len() { pos } else { input.len() };
    eprintln!("       {}^", " ".repeat(caret_pos));
}

fn main() {
    // let test_re = "\\d\\w[anc]";
    // let re = parser::parse(test_re);

    let raw_re = "([^abc]+|(cat|dog123)[123]12)\\w";
    let re = parser::parse(raw_re);
    if let Err(e) = re {
        report_error(e, raw_re);
    } else {
        // dbg!(&re);
        println!("{re:?}");
    }
}
