use m6lexerkit::{
    lazy_static,
    make_token_matcher_rules, tokenize as tokenize__, SrcFileInfo,
    TokenMatchResult, TokenizeResult,
    prelude::*
};


make_token_matcher_rules! {
    tag       => "[[:alpha:]_][[:alnum:]_]*#",
    id        => "[[:alpha:]_][[:alnum:]_]*",
    attr      => r#"@\w+"#,

    // Lit
    lit_int => r"[+|-]?(([0-9]+)|(0x[0-9a-f]+))",
    lit_float => r"[+|-]?([0-9]+\.[0-9])",
    sqstr,
    dqstr,
    aqstr,
    cmd,
    lit_regex,

    // Comment
    sharp_line_comment,

    // White characters
    sp,
    newline,

    // Bracket
    lparen,
    rparen,
    lbracket,
    rbracket,
    lbrace,
    rbrace,

    // Delimiter
    colon,
    question,
    rarrow,
    rdarrow,
    semi,
    comma,

    // Assign
    assign,

    // Unary Operation
    inc,
    dec,
    not,

    // Binary Operation
    sub,
    add,
    mul,
    div,
    dot,
    ge,
    le,
    lt,
    gt,
    neq,
    eq,
    percent,
    and,
    or
}

fn cmd_m(source: &str, from: usize) -> Option<TokenMatchResult> {
    aux_strlike_m(source, from, "!(", ")", '\\')
        .and_then(|res| Some(res.and_then(|tok| Ok(tok.rename("cmd")))))
}


lazy_static::lazy_static! {
    static ref BLANK_TOK_SET: Vec<&'static str> = vec! [
        "sp",
        "newline",
        "sharp_line_comment"
    ];
    static ref KEY_SET: Vec<&'static str> = vec! {
        "fn",
        "return",
        "ret",
        "if",
        "else",
        "loop",
        "while",
        "break",
        "continue",
        "let"
    };
}


pub(crate) fn tokenize(source: &SrcFileInfo) -> TokenizeResult {
    tokenize_(source).and_then(|toks| {
        Ok(toks
            .into_iter()
            .filter(|tok| !BLANK_TOK_SET.contains(&tok.name_string().as_str()))
            .map(|mut tok| {
                if tok.check_name("attr") {
                    tok = tok.mapval(&tok.value_string()[1..]);
                }
                if tok.check_name("tag") {
                    let s = &tok.value_string();
                    tok = tok.mapval(&s[..s.len() - 1]);
                }

                tok.rename_by_value(&KEY_SET)
            })
            // .chain([Token::eof()])
            .collect::<Vec<Token>>())
    })
}


#[inline]
fn tokenize_(source: &SrcFileInfo) -> TokenizeResult {
    tokenize__(source, &MATCHERS[..])
}




#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::{SrcFileInfo, Token};

    use super::tokenize;

    fn display_pure_tokval(tokens: &[Token], src: &SrcFileInfo) {
        for token in tokens.iter() {
            println!(
                "<{}>: {}: {}",
                token.name_string(),
                token.value_string(),
                src.boffset2srcloc(token.span().from)
            )
        }
    }

    #[test]
    fn test_lexer() {
        let path = PathBuf::from("./examples/exp0.bath");
        let srcfile = SrcFileInfo::new(&path).unwrap();

        match tokenize(&srcfile) {
            Ok(tokens) => {
                display_pure_tokval(&tokens[..], &srcfile);
            }
            Err(err) => println!("{}", err),
        }
    }
}
