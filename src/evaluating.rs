use crate::parsing;

pub fn evaluate(ast: parsing::AST) -> String {
    evaluate_expression(&ast, ast.expressions.len() - 1)
}

pub fn evaluate_expression(ast: &parsing::AST, expression: usize) -> String {
    match ast.expressions[expression] {
        parsing::Expression::Abstraction {
            expression,
        } => {
            format!("(\\{})", evaluate_expression(ast, expression))
        }

        parsing::Expression::Application { function, argument } => {
            format!(
                "({} {})",
                evaluate_expression(ast, function),
                evaluate_expression(ast, argument)
            )
        }

        parsing::Expression::Variable { index } => {
            format!("{index}")
        }
    }
}
