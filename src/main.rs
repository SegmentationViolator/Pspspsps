use std::io::Write;
use std::{collections, io};

mod evaluation;
mod lexing;
mod parsing;

const PROMPT: &[u8] = "\u{03c8}\u{03c8}\u{03c8}\u{03c8}> ".as_bytes();

fn display_expression(
    source: &str,
    expression_graph: &parsing::ExpressionGraph,
    expression: parsing::ExpressionId,
) -> String {
    fn inner_display_expression(
        source: &str,
        expression_graph: &parsing::ExpressionGraph,
        expression: parsing::ExpressionId,
        visiting: &mut collections::HashSet<parsing::ExpressionId>,
    ) -> String {
        if !visiting.insert(expression) {
            return match expression_graph[expression] {
                parsing::Expression::Abstraction { .. } => "\\ ...".to_string(),
                parsing::Expression::Application { .. } => "... ...".to_string(),
                parsing::Expression::Indirection { .. } => "...".to_string(),
                _ => unreachable!(),
            };
        }

        let result = match expression_graph[expression] {
            parsing::Expression::Abstraction { body, .. } => format!(
                "(\\ {})",
                inner_display_expression(source, expression_graph, body, visiting)
            ),

            parsing::Expression::Application {
                function, argument, ..
            } => format!(
                "{} {}",
                inner_display_expression(source, expression_graph, function, visiting),
                inner_display_expression(source, expression_graph, argument, visiting)
            ),

            parsing::Expression::Indirection { target, .. } => {
                inner_display_expression(source, expression_graph, target, visiting)
            }

            parsing::Expression::Symbol { ref span } => source[span.clone()].to_string(),
            parsing::Expression::Variable { index } => format!("{}", index),
        };

        visiting.remove(&expression);
        result
    }

    let mut visited = collections::HashSet::with_capacity(expression_graph.len());
    inner_display_expression(source, expression_graph, expression, &mut visited)
}

fn main() {
    let mut buffer = String::with_capacity(512);
    let mut stderr = io::stderr();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        buffer.clear();
        if let Err(error) = stdout.write_all(PROMPT).and_then(|()| stdout.flush()) {
            return eprintln!("\nError while writing to stdout: {}", error);
        }

        if let Err(error) = stdin.read_line(&mut buffer) {
            return eprintln!("\nError while reading from stdin: {}", error);
        }

        if buffer.is_empty() {
            return;
        }

        let mut expression_graph = match parsing::ParsingContext::new(&buffer).parse() {
            Err(error) => {
                let _ = stderr.write_all(error.display(&buffer).as_bytes());
                return;
            }
            Ok(graph) => graph,
        };

        let root = evaluation::EvaluationContext::new(&mut expression_graph).evaluate();

        println!(
            "{}",
            display_expression(&buffer, &expression_graph, root)
        );
    }
}
