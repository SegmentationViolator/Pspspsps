use std::marker;
use std::mem;
use std::ops;

pub mod graph_state {
    pub struct Complete;
    pub struct Incomplete;
}

pub type CompleteExpressionGraph = ExpressionGraph<graph_state::Complete>;

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

#[derive(Debug)]
pub struct ExpressionGraph<S> {
    expressions: Vec<Expression>,
    root: ExpressionId,
    _state: marker::PhantomData<S>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Eq)]
pub struct ExpressionId(usize);

impl<S> ops::Index<ExpressionId> for ExpressionGraph<S> {
    type Output = Expression;

    fn index(&self, index: ExpressionId) -> &Self::Output {
        self.expressions.index(index.0)
    }
}

impl<S> ops::IndexMut<ExpressionId> for ExpressionGraph<S> {
    fn index_mut(&mut self, index: ExpressionId) -> &mut Self::Output {
        self.expressions.index_mut(index.0)
    }
}

impl<S> ExpressionGraph<S> {
    pub fn add(&mut self, expression: Expression) -> ExpressionId {
        let index = self.expressions.len();
        self.expressions.push(expression);

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

    pub fn len(&self) -> usize {
        self.expressions.len()
    }
}

impl ExpressionGraph<graph_state::Complete> {
    pub fn root(&self) -> ExpressionId {
        self.root
    }
}

impl ExpressionGraph<graph_state::Incomplete> {
    pub fn new(expressions: Vec<Expression>) -> Self {
        Self { expressions, root: ExpressionId::NULL, _state: marker::PhantomData }
    }

    pub fn mark_as_complete(mut self) -> ExpressionGraph<graph_state::Complete> {
        self.root = ExpressionId(self.expressions.len() - 1);
        unsafe { mem::transmute(self) }
    }
}

impl ExpressionId {
    pub const NULL: Self = Self(usize::MAX);
}
