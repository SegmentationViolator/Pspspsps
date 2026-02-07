use super::lexing;

pub struct Ast {
    pub expressions: Vec<Expression>,
}

pub enum Error {
    IncorrectToken {
        actual: lexing::Token,
        expected: lexing::TokenKind,
    },

    TokenStreamExhausted {
        position: lexing::Position,
    },

    UndefinedLabel {
        position: lexing::Position,
    },

    UnexpectedToken {
        token: lexing::Token,
    },
}

pub enum Expression {
    Abstraction { expression: usize },

    Application { function: usize, argument: usize },

    Variable { index: usize },
}

pub struct ParsingContext<'s> {
    token_stream: lexing::TokenStream<'s>,
    variables: ahash::AHashMap<usize, usize>,
    expressions: Vec<Expression>,
    current_depth: usize,
    unmatched_tokens: usize,
}

impl<'s> ParsingContext<'s> {
    fn add_expression(&mut self, expression: Expression) -> usize {
        let index = self.expressions.len();
        self.expressions.push(expression);

        index
    }

    fn expect(&mut self, expected: lexing::TokenKind) -> Result<lexing::Token, Error> {
        match self.token_stream.next() {
            None => Err(Error::TokenStreamExhausted {
                position: self.token_stream.position,
            }),
            Some(token) if token.kind == expected => Ok(token),
            Some(token) => Err(Error::IncorrectToken {
                actual: token,
                expected,
            }),
        }
    }

    pub fn new(source: &'s str) -> Self {
        Self {
            current_depth: 0,
            expressions: Vec::with_capacity(16),
            variables: ahash::AHashMap::with_capacity(16),
            token_stream: lexing::TokenStream::new(source),
            unmatched_tokens: 0,
        }
    }

    pub fn parse(mut self) -> Result<Ast, Error> {
        let root = self.parse_expression()?;
        self.add_expression(root);

        Ok(Ast {
            expressions: self.expressions,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, Error> {
        let mut expression = self.parse_subexpression()?;

        while let Some(token) = self.token_stream.peek()
            && (self.unmatched_tokens == 0 || token.kind != lexing::TokenKind::RightParenthesis)
        {
            let function = self.add_expression(expression);
            let argument = self.parse_subexpression()?;
            let argument = self.add_expression(argument);

            expression = Expression::Application { function, argument };
        }

        Ok(expression)
    }

    fn parse_subexpression(&mut self) -> Result<Expression, Error> {
        match self.token_stream.next() {
            Some(lexing::Token {
                kind: lexing::TokenKind::Backslash,
                ..
            }) => {
                let intern = self.expect(lexing::TokenKind::Label)?.intern.unwrap();

                self.expect(lexing::TokenKind::FullStop)?;

                let previous_reference = self.variables.insert(intern, self.current_depth);
                self.current_depth += 1;

                let expression = self.parse_expression()?;
                let expression = self.add_expression(expression);

                self.current_depth -= 1;
                if let Some(reference) = previous_reference {
                    self.variables.insert(intern, reference);
                } else {
                    self.variables.remove(&intern);
                }

                Ok(Expression::Abstraction { expression })
            }

            Some(
                token @ lexing::Token {
                    kind: lexing::TokenKind::Label,
                    intern: Some(intern),
                    ..
                },
            ) => {
                let Some(depth) = self.variables.get(&intern).copied() else {
                    return Err(Error::UndefinedLabel {
                        position: token.position,
                    });
                };

                Ok(Expression::Variable {
                    index: self.current_depth - depth,
                })
            }

            Some(lexing::Token {
                kind: lexing::TokenKind::LeftParenthesis,
                ..
            }) => {
                self.unmatched_tokens += 1;
                let expression = self.parse_expression()?;
                self.expect(lexing::TokenKind::RightParenthesis)?;
                self.unmatched_tokens -= 1;

                Ok(expression)
            }

            Some(token) => Err(Error::UnexpectedToken { token }),
            None => Err(Error::TokenStreamExhausted {
                position: self.token_stream.position,
            }),
        }
    }
}
