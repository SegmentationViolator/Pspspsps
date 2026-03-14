use super::parsing;

pub struct EvaluationContext<'g> {
    expression_graph: &'g mut parsing::ExpressionGraph,
    stack: Vec<parsing::ExpressionId>,
}

impl<'g> EvaluationContext<'g> {
    pub fn new(expression_graph: &'g mut parsing::ExpressionGraph) -> Self {
        Self {
            expression_graph,
            stack: Vec::with_capacity(16),
        }
    }

    pub fn evaluate(mut self) -> parsing::ExpressionId {
        let root = self.expression_graph.root();

        if root == parsing::ExpressionId::NULL {
            return root;
        }

        while self.reduce_expression(root, root) {}

        root
    }

    pub fn reduce_expression(&mut self, expression: parsing::ExpressionId, root: parsing::ExpressionId) -> bool {
        let expression = self.resolve_indirection(expression);

        let parsing::Expression::Application { function, argument } =
            self.expression_graph[expression]
        else {
            return false;
        };

        if self.reduce_expression(function, root) {
            return true;
        }

        let function = self.resolve_indirection(function);

        let parsing::Expression::Abstraction { body } = self.expression_graph[function] else {
            return false;
        };

        self.stack.push(argument);

        let target = if body > root {
            body
        } else {
            self.expression_graph.clone_expression(body)
        };

        self.expression_graph[expression] = parsing::Expression::Indirection { target };
        let reduced = self.beta_reduce(target);
        self.stack.pop();

        reduced
    }

    pub fn beta_reduce(&mut self, expression: parsing::ExpressionId) -> bool {
        match self.expression_graph[expression] {
            parsing::Expression::Abstraction { body } => {
                self.stack.push(parsing::ExpressionId::NULL);
                let was_reduced = self.beta_reduce(body);
                self.stack.pop();

                was_reduced
            }

            parsing::Expression::Application { function, argument } => {
                let reduced_function = self.beta_reduce(function);
                let reduced_argument = self.beta_reduce(argument);

                reduced_function || reduced_argument
            }

            parsing::Expression::Indirection { .. } => {
                self.resolve_indirection(expression);

                false
            }

            parsing::Expression::Symbol { .. } => false,

            parsing::Expression::Variable { index } => {
                let argument = self.stack[self.stack.len() - index];

                if argument == parsing::ExpressionId::NULL {
                    return false;
                }

                self.expression_graph[expression] = parsing::Expression::Indirection {
                    target: self.resolve_indirection(argument),
                };

                true
            }
        }
    }

    pub fn resolve_indirection(
        &mut self,
        expression: parsing::ExpressionId,
    ) -> parsing::ExpressionId {
        let parsing::Expression::Indirection { mut target } = self.expression_graph[expression]
        else {
            return expression;
        };

        let old_target = target;

        while let parsing::Expression::Indirection { target: new_target } =
            self.expression_graph[target]
        {
            target = new_target;
        }

        if old_target != target {
            self.expression_graph[expression] = parsing::Expression::Indirection { target };
        }

        target
    }
}
