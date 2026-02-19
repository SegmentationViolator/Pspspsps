use std::{alloc, mem, ptr};

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

    UndefinedLabel {
        position: lexing::Position,
    },

    UnexpectedToken {
        token: lexing::Token,
    },
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Expression {
    Abstraction {
        body: ExpressionId,
    },

    Application {
        function: ExpressionId,
        argument: ExpressionId,
    },

    Variable {
        index: usize,
    },
}

pub struct ExpressionGraph {
    ptr: *const Expression,
    len: usize, 
    cap: usize,
}

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ExpressionId(pub usize);

pub struct ParsingContext<'s> {
    abstractions: ahash::AHashMap<ExpressionId, ExpressionId>,
    applications: ahash::AHashMap<(ExpressionId, ExpressionId), ExpressionId>,
    depths: ahash::AHashMap<lexing::Intern, usize>,
    token_stream: lexing::TokenStream<'s>,
    expressions: Vec<Expression>,
    variables: Vec<Option<ExpressionId>>,
    current_depth: usize,
    unmatched_tokens: usize,
}

impl From<Vec<Expression>> for ExpressionGraph {
    fn from(value: Vec<Expression>) -> Self {
        let cap = value.capacity();
        let len = value.len();
        let ptr = value.as_ptr();
        mem::forget(value);

        ExpressionGraph {
            ptr,
            len,
            cap,
        }
    }
}

impl Drop for ExpressionGraph {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        unsafe { alloc::dealloc(self.ptr as *mut u8, alloc::Layout::array::<Expression>(self.cap).unwrap_unchecked()); }
    }
}

impl ExpressionGraph {
    pub fn get<'g>(&self, expression: ExpressionId) -> Option<&'g Expression> {
        if expression.0 >= self.len {
            return None;
        }

        Some(unsafe { &*self.ptr.add(expression.0) })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'s> ParsingContext<'s> {
    fn add_expression(&mut self, expression: Expression) -> ExpressionId {
        let index = self.expressions.len();
        self.expressions.push(expression);

        ExpressionId(index)
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
            abstractions: ahash::AHashMap::with_capacity(16),
            applications: ahash::AHashMap::with_capacity(16),
            current_depth: 0,
            depths: ahash::AHashMap::with_capacity(16),
            expressions: Vec::with_capacity(16),
            token_stream: lexing::TokenStream::new(source),
            unmatched_tokens: 0,
            variables: Vec::with_capacity(16),
        }
    }

    pub fn parse(mut self) -> Result<ExpressionGraph, Error> {
        self.parse_application()?;

        Ok(self.expressions.into())
    }

    fn parse_application(&mut self) -> Result<ExpressionId, Error> {
        let mut function = self.parse_other()?;

        while let Some(token) = self.token_stream.peek()
            && (self.unmatched_tokens == 0 || token.kind != lexing::TokenKind::RightParenthesis)
        {
            let argument = self.parse_other()?;

            if let Some(expression) = self.applications.get(&(function, argument)).copied() {
                function = expression;
                continue;
            }

            let expression = self.add_expression(Expression::Application { function, argument });
            self.applications.insert((function, argument), expression);

            function = expression;
        }

        Ok(function)
    }

    fn parse_other(&mut self) -> Result<ExpressionId, Error> {
        match self.token_stream.next() {
            Some(lexing::Token {
                kind: lexing::TokenKind::Backslash,
                ..
            }) => {
                let intern = self.expect(lexing::TokenKind::Label)?.intern.unwrap();

                self.expect(lexing::TokenKind::FullStop)?;

                let index = self.depths.insert(intern, self.current_depth);
                self.current_depth += 1;

                let body = self.parse_application()?;

                self.current_depth -= 1;
                if let Some(index) = index {
                    self.depths.insert(intern, index);
                } else {
                    self.depths.remove(&intern);
                }

                if let Some(expression) = self.abstractions.get(&body).copied() {
                    return Ok(expression);
                }

                let expression = self.add_expression(Expression::Abstraction { body });
                self.abstractions.insert(body, expression);

                Ok(expression)
            }

            Some(
                token @ lexing::Token {
                    kind: lexing::TokenKind::Label,
                    intern: Some(intern),
                    ..
                },
            ) => {
                let Some(depth) = self.depths.get(&intern).copied() else {
                    return Err(Error::UndefinedLabel {
                        position: token.position,
                    });
                };

                let index = self.current_depth - depth;

                if let Some(expression) = self.variables.get(index-1).copied().flatten() {
                    return Ok(expression);
                }

                let expression = self.add_expression(Expression::Variable { index });

                if index >= self.variables.len() {
                    self.variables.resize(index, None);
                }
                self.variables[index-1] = Some(expression);

                Ok(expression)
            }

            Some(lexing::Token {
                kind: lexing::TokenKind::LeftParenthesis,
                ..
            }) => {
                self.unmatched_tokens += 1;
                let expression = self.parse_application()?;
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
