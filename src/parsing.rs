use std::ops;

use super::lexing;

#[derive(Debug)]
pub enum Error {
    IncorrectToken {
        actual: lexing::Token,
        expected: lexing::TokenKind,
    },

    TokenStreamExhausted {
        position: lexing::Position,
    },

    UnexpectedToken {
        token: lexing::Token,
    },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expression {
    Abstraction {
        body: ExpressionId,
    },

    Application {
        function: ExpressionId,
        argument: ExpressionId,
    },

    Indirection {
        target: ExpressionId,
    },

    Symbol {
        span: ops::Range<usize>,
    },

    Variable {
        index: usize,
    },
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ExpressionGraph(Vec<Expression>);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq)]
pub struct ExpressionId(usize);

pub struct ParsingContext<'s> {
    abstractions: ahash::AHashMap<ExpressionId, ExpressionId>,
    applications: ahash::AHashMap<(ExpressionId, ExpressionId), ExpressionId>,
    depths: ahash::AHashMap<lexing::Intern, usize>,
    token_stream: lexing::TokenStream<'s>,
    expressions: ExpressionGraph,
    current_depth: usize,
    unmatched_tokens: usize,
}

impl ops::Index<ExpressionId> for ExpressionGraph {
    type Output = Expression;

    fn index(&self, index: ExpressionId) -> &Self::Output {
        self.0.index(index.0)
    }
}

impl ops::IndexMut<ExpressionId> for ExpressionGraph {
    fn index_mut(&mut self, index: ExpressionId) -> &mut Self::Output {
        self.0.index_mut(index.0)
    }
}

impl Error {
    pub fn display(self, source: &str) -> String {
        match self {
            Error::IncorrectToken { actual, expected } => {
                format!(
                    "at line {}, column {}: expected {} found {}\n",
                    actual.position.line,
                    actual.position.column,
                    expected,
                    actual.display(source)
                )
            }
            Error::TokenStreamExhausted { position } => {
                format!(
                    "at line {}, column {}: unexpected end-of-stream\n",
                    position.line, position.column
                )
            }
            Error::UnexpectedToken { token } => {
                format!(
                    "at line {}, column {}: unexpected {}\n",
                    token.position.line,
                    token.position.column,
                    token.display(source)
                )
            }
        }
    }
}

impl ExpressionGraph {
    pub fn add(&mut self, expression: Expression) -> ExpressionId {
        let index = self.0.len();
        self.0.push(expression);

        ExpressionId(index)
    }

    pub fn clone_expression(&mut self, expression: ExpressionId) -> ExpressionId {
        match self[expression] {
            Expression::Abstraction { body } => {
                let body = self.clone_expression(body);
                self.add(Expression::Abstraction { body })
            }
            Expression::Application { function, argument } => {
                let function = self.clone_expression(function);
                let argument = self.clone_expression(argument);
                self.add(Expression::Application { function, argument })
            }
            Expression::Indirection { .. } => expression,
            Expression::Symbol { ref span } => self.add(Expression::Symbol { span: span.clone() }),
            Expression::Variable { index } => self.add(Expression::Variable { index }),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn root(&self) -> ExpressionId {
        ExpressionId(self.len() - 1)
    }
}

impl ExpressionId {
    pub const NULL: Self = Self(usize::MAX);
}

impl<'s> ParsingContext<'s> {
    fn expect(&mut self, expected: lexing::TokenKind) -> Result<lexing::Token, Error> {
        match self.token_stream.next() {
            Some(token) if token.kind == expected => Ok(token),

            None => Err(Error::TokenStreamExhausted {
                position: self.token_stream.position,
            }),
            Some(token) => Err(Error::IncorrectToken {
                actual: token,
                expected,
            }),
        }
    }

    pub fn new(source: &'s str) -> Self {
        Self {
            abstractions: ahash::AHashMap::with_capacity(16),
            applications: ahash::AHashMap::with_capacity(16),
            current_depth: 0,
            depths: ahash::AHashMap::with_capacity(16),
            expressions: ExpressionGraph(Vec::with_capacity(16)),
            token_stream: lexing::TokenStream::new(source),
            unmatched_tokens: 0,
        }
    }

    pub fn parse(mut self) -> Result<ExpressionGraph, Error> {
        self.parse_expression()?;

        Ok(self.expressions)
    }

    fn parse_expression(&mut self) -> Result<ExpressionId, Error> {
        let mut function = self.parse_subexpression()?;

        while let Some(token) = self.token_stream.peek()
            && (self.unmatched_tokens == 0 || token.kind != lexing::TokenKind::RightParenthesis)
        {
            let argument = self.parse_subexpression()?;

            if let Some(expression) = self.applications.get(&(function, argument)).copied() {
                function = expression;
                continue;
            }

            let expression = self
                .expressions
                .add(Expression::Application { function, argument });
            self.applications.insert((function, argument), expression);

            function = expression;
        }

        Ok(function)
    }

    fn parse_subexpression(&mut self) -> Result<ExpressionId, Error> {
        match self.token_stream.next() {
            Some(lexing::Token {
                kind: lexing::TokenKind::Backslash,
                ..
            }) => {
                let intern = self.expect(lexing::TokenKind::Label)?.intern.unwrap();

                self.expect(lexing::TokenKind::FullStop)?;

                let previous_depth = self.depths.insert(intern, self.current_depth);
                self.current_depth += 1;

                let body = self.parse_expression()?;

                self.current_depth -= 1;
                if let Some(depth) = previous_depth {
                    self.depths.insert(intern, depth);
                } else {
                    self.depths.remove(&intern);
                }

                if let Some(expression) = self.abstractions.get(&body).copied() {
                    return Ok(expression);
                }

                let expression = self.expressions.add(Expression::Abstraction { body });
                self.abstractions.insert(body, expression);

                Ok(expression)
            }

            Some(lexing::Token {
                kind: lexing::TokenKind::Label,
                intern: Some(intern),
                span,
                ..
            }) => {
                let Some(depth) = self.depths.get(&intern).copied() else {
                    return Ok(self.expressions.add(Expression::Symbol { span }));
                };

                Ok(self.expressions.add(Expression::Variable {
                    index: self.current_depth - depth,
                }))
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
