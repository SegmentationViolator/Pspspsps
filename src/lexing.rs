use std::fmt::Display;
use std::ops;

#[derive(Clone, Copy)]
pub struct Position {
    pub column: usize,
    pub line: usize,
}

pub struct Token {
    pub span: ops::Range<usize>,
    pub position: Position,
    pub kind: TokenKind,
}

pub enum TokenKind {
    Backslash,
    FullStop,
    IllegalCharacter,
    Integer,
    Label,
    LeftParenthesis,
    RightParenthesis,
}

pub struct TokenStream<'s> {
    source: &'s str,
    position: Position,
    index: usize,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Backslash => "'\\'",
            Self::FullStop => "'.'",
            Self::IllegalCharacter => "illegal character",
            Self::Integer => "{integer}",
            Self::Label => "{label}",
            Self::LeftParenthesis => "'('",
            Self::RightParenthesis => "')'",
        })
    }
}

impl Iterator for TokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut characters = self.source[self.index..].chars();
        let mut character = characters.next()?;

        while character.is_whitespace() {
            if character == '\n' {
                self.position.column = 0;
                self.position.line += 1;
            }

            self.index += character.len_utf8();
            self.position.column += 1;

            character = characters.next()?;
        }

        let (span, kind) = match character {
            '\\' => (self.index..self.index+1, TokenKind::Backslash),
            '.' => (self.index..self.index+1, TokenKind::FullStop),
            '(' => (self.index..self.index+1, TokenKind::LeftParenthesis),
            ')' => (self.index..self.index+1, TokenKind::RightParenthesis),

            '0'..='9' => {
                let mut end = self.index + character.len_utf8();
                let start = self.index;

                while let Some(character) = characters.next() && character.is_ascii_digit() {
                    self.position.column += 1;
                    end += character.len_utf8();
                }

                self.position.column -= 1;

                (start..end, TokenKind::Integer)
            }

            _ if character.is_alphabetic() => {
                let mut end = self.index + character.len_utf8();
                let start = self.index;

                while let Some(character) = characters.next() && character.is_alphanumeric() {
                    self.position.column += 1;
                    end += character.len_utf8();
                }

                self.position.column -= 1;

                (start..end, TokenKind::Label)
            }


            _ => (self.index..self.index+character.len_utf8(), TokenKind::IllegalCharacter),
        };

        let position = self.position;

        self.index = span.end;
        self.position.column += 1;

        Some(Token { span, position, kind })
    }
}

impl<'s> TokenStream<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            position: Position { column: 1, line: 1 },
            index: 0,
        }
    }
}
