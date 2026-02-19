use std::iter;
use std::ops;
use std::str;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Intern(usize);

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

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Backslash,
    FullStop,
    IllegalCharacter,
    Label,
    LeftParenthesis,
    RightParenthesis,
}

pub struct TokenStream<'s> {
    symbols: ahash::AHashMap<&'s str, Intern>,
    peeked: Option<Token>,
    characters: iter::Peekable<str::CharIndices<'s>>,
    source: &'s str,
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

                let label = &self.source[start..end];
                let intern = Intern(self.symbols.len());
                let intern = *self.symbols.entry(label).or_insert(intern);

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
            symbols: ahash::AHashMap::with_capacity(16),
            source,
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
