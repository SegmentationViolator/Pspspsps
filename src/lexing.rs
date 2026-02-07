use std::iter;
use std::ops;
use std::str;

#[derive(Clone, Copy)]
pub struct Position {
    pub column: usize,
    pub line: usize,
}

pub struct Token {
    pub span: ops::Range<usize>,
    pub intern: Option<usize>,
    pub position: Position,
    pub kind: TokenKind,
}

#[derive(PartialEq, Eq)]
pub enum TokenKind {
    Backslash,
    FullStop,
    IllegalCharacter,
    Label,
    LeftParenthesis,
    RightParenthesis,
}

pub struct TokenStream<'s> {
    characters: iter::Peekable<str::CharIndices<'s>>,
    interns: ahash::AHashMap<&'s str, usize>,
    original: &'s str,
    peeked: Option<Token>,
    pub position: Position,
}

impl Iterator for TokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }

        while let Some((_, character)) = self.characters.peek().copied()
            && character.is_whitespace()
        {
            if character == '\n' {
                self.position.column = 0;
                self.position.line += 1;
            }

            self.position.column += 1;
            self.characters.next();
        }

        let position = self.position;

        let (start, character) = self.characters.next()?;

        let (end, kind, intern) = match character {
            '\\' => (start + 1, TokenKind::Backslash, None),
            '.' => (start + 1, TokenKind::FullStop, None),
            '(' => (start + 1, TokenKind::LeftParenthesis, None),
            ')' => (start + 1, TokenKind::RightParenthesis, None),

            _ if character.is_alphabetic() => {
                let mut end = start + character.len_utf8();

                while let Some((_, character)) = self.characters.peek().copied()
                    && character.is_alphanumeric()
                {
                    self.characters.next();
                    end += character.len_utf8();
                    self.position.column += 1;
                }

                let label = &self.original[start..end];
                let intern = self.interns.len();
                let intern = *self.interns.entry(label).or_insert(intern);

                (end, TokenKind::Label, Some(intern))
            }

            _ => (
                start + character.len_utf8(),
                TokenKind::IllegalCharacter,
                None,
            ),
        };

        self.position.column += 1;

        Some(Token {
            span: start..end,
            position,
            kind,
            intern,
        })
    }
}

impl<'s> TokenStream<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            characters: source.char_indices().peekable(),
            interns: ahash::AHashMap::with_capacity(16),
            original: source,
            peeked: None,
            position: Position { column: 1, line: 1 },
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            let peeked = self.next()?;
            let _ = self.peeked.insert(peeked);
        }

        self.peeked.as_ref()
    }
}
