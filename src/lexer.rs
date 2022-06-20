use std::collections::HashSet;

use m6lexerkit::{
    aqstr_m, aux_strlike_m, dqstr_m, lazy_static, lit_regex_m,
    make_token_matcher_rules, sqstr_m, tokenize as tokenize__, SrcFileInfo,
    TokenMatchResult, TokenizeResult,
};
use maplit::hashset;

make_token_matcher_rules! {
    id        => "[[:alpha:]_][[:alnum:]_]*",

    // Lit
    lit_int => r"[+|-]?(([0-9]+)|(0x[0-9a-f]+))",
    lit_float => r"[+|-]?([0-9]+\.[0-9])",
    sqstr,
    dqstr,
    aqstr,
    cmd,
    lit_regex,

    // Comment
    sharp_line_comment  => r"#.*",

    // space
    sp      => "[[:blank:]]+",
    newline => r#"\n\r?"#,

    // Bracket
    lparen => r"\(",
    rparen => r"\)",
    lbracket => r"\[",
    rbracket => r"\]",
    lbrace => r"\{",
    rbrace => r"\}",

    // Delimiter
    colon => ":",
    question => r"\?",
    rarrow => "->",
    rdarrow  => "=>",
    semi   => ";",
    comma  => ",",

    // Assign
    assign => "=",

    // Unary Operation
    inc => r"\+\+",
    dec => r"--",
    not => "!",

    // Binary Operation
    sub    => "-",
    add    => r"\+[^\+]",
    mul    => r"\*",
    div    => "/",
    dot    => r"\.",
    ge     => ">=",
    le     => "<=",
    lt     => "<",
    gt     => ">",
    neq    => "!=",
    eq     => "==",
    percent=> "%",
    and    => "&&",
    or     => r"\|\|"

}

fn cmd_m(source: &str, from: usize) -> Option<TokenMatchResult> {
    aux_strlike_m(source, from, "!(", ")", '\\')
        .and_then(|res| Some(res.and_then(|tok| Ok(tok.rename("cmd")))))
}


lazy_static::lazy_static! {
    static ref BLANK_TOK_SET: HashSet<&'static str> = hashset! {
        "sp",
        "newline",
        "sharp_line_comment"
    };
    static ref KEY_SET: Vec<&'static str> = vec! {
        "fn",
        "return",
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
            .map(|tok| {
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


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use super::{display_pure_tokval, tokenize};

    #[test]
    fn test_lexer() {
        let path = PathBuf::from("./examples/exp0.bath");
        let srcfile = SrcFileInfo::new(&path).unwrap();

        match tokenize(&srcfile) {
            Ok(tokens) => {
                display_pure_tokval(&tokens, &srcfile);
            }
            Err(err) => println!("{}", err),
        }
    }
}
