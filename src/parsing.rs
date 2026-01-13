use super::lexing;

pub enum Error {
    IncorrectToken {
        actual: lexing::Token,
        expected: lexing::TokenKind,
    },

    TokenStreamExhausted,

    UnexpectedToken {
        token: lexing::Token,
    },
}

pub enum Expression {
    Call {
        function: Box<Expression>,
        argument: Box<Expression>,
    },

    Definition {
        body: Box<Expression>,
        parameter: usize,
    },

    Variable {
        reference: usize,
    },
}
