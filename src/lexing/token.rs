use std::fmt;
use std::ops;

pub type Intern = usize;

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub column: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct Token {
    pub span: ops::Range<usize>,
    pub position: Position,
    pub intern: Option<Intern>,
    pub kind: TokenKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Backslash,
    FullStop,
    IllegalCharacter,
    Label,
    LeftParenthesis,
    RightParenthesis,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Backslash => write!(f, "\\"),
            TokenKind::FullStop => write!(f, "."),
            TokenKind::IllegalCharacter => write!(f, "illegal character"),
            TokenKind::Label => write!(f, "label"),
            TokenKind::LeftParenthesis => write!(f, "("),
            TokenKind::RightParenthesis => write!(f, ")"),
        }
    }
}

impl Token {
    pub fn display(&self, source: &str) -> String {
        match self.kind {
            literal @ (TokenKind::Backslash
            | TokenKind::FullStop
            | TokenKind::LeftParenthesis
            | TokenKind::RightParenthesis) => literal.to_string(),

            non_literal @ (TokenKind::IllegalCharacter | TokenKind::Label) => format!("{} {}", non_literal, &source[self.span.clone()]),
        }
    }
}
