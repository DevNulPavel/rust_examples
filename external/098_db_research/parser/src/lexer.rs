use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token<'a> {
    // Операторы
    #[token("AND")]
    And,
    #[token("OR")]
    Or,
    #[token("NOT")]
    Not,
    #[token("NOW")]
    Now,
    #[token("AGE")]
    Age,
    #[token("EXISTS")]
    Exists,
    #[token("AMONG")]
    Among,
    #[token("ANY")]
    Any,
    #[token("ALL")]
    All,
    #[token("LEN")]
    Len,
    #[token("MOD")]
    Mod,
    #[token("&")]
    BitAnd,

    // Время
    #[token("seconds")]
    #[token("second")]
    Second,
    #[token("minutes")]
    #[token("minute")]
    Minute,
    #[token("hours")]
    #[token("hour")]
    Hour,
    #[token("days")]
    #[token("day")]
    Day,

    // Символы
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    // Компараторы
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("CONTAINS")]
    Contains,
    #[token("ICONTAINS")]
    Icontains,

    // JSON Литералы
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    // Строки
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLit(String),

    // Идентификаторы (теперь любой JSON ключ)
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),

    // Числа
    #[regex(r"-?[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),
}
