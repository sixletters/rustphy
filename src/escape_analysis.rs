use crate::ast::{ExpressionNode, Node, StatementNode};
use crate::symbol_table::SymbolTable;
use std::collections::HashMap;

pub struct EscapeAnalysis {
    pub escaped_variables: HashMap<usize, bool>, // binding_id → escapes
}

impl EscapeAnalysis {
    pub fn analyze(ast: &Node, symbol_table: &SymbolTable) -> Self {
        let mut analyzer = EscapeAnalyzer {
            symbol_table,
            escapes: HashMap::new(),
            function_scope_stack: vec![],
        };
        analyzer.walk_node(ast);
        EscapeAnalysis {
            escaped_variables: analyzer.escapes,
        }
    }

    pub fn does_escape(&self, binding_id: usize) -> bool {
        self.escaped_variables
            .get(&binding_id)
            .copied()
            .unwrap_or(false)
    }
}

pub struct EscapeAnalyzer<'a> {
    symbol_table: &'a SymbolTable<'a>,
    escapes: HashMap<usize, bool>,
    function_scope_stack: Vec<usize>, // Track nested function scopes
}

impl<'a> EscapeAnalyzer<'a> {
    fn walk_node(&mut self, node: &Node) {
        match node {
            Node::ExpressionNode(expr) => self.walk_expression(expr),
            Node::StatementNode(stmt) => self.walk_statement(stmt),
        }
    }

    fn walk_statement(&mut self, stmt: &StatementNode) {
        match stmt {
            StatementNode::Let { value, .. } => {
                self.walk_expression(value);
            }
            StatementNode::Expression { expression, .. } => {
                self.walk_expression(expression);
            }
            StatementNode::Block { statements, .. } => {
                for stmt in statements {
                    self.walk_statement(stmt);
                }
            }
            StatementNode::Program { statements, .. } => {
                for node in statements {
                    self.walk_node(node);
                }
            }
            StatementNode::FuncDeclr { func, .. } => {
                self.walk_expression(func);
            }
            StatementNode::Return { return_value, .. } => {
                self.walk_expression(return_value);
            }
            StatementNode::For { for_block, .. } => {
                self.walk_statement(for_block);
            }
            _ => {}
        }
    }

    fn walk_expression(&mut self, expr: &ExpressionNode) {
        match expr {
            ExpressionNode::Function { body, id, .. } => {
                // Get scope_id of this function from the symbol table
                if let Some(func_scope_id) = self.symbol_table.get_scope_for_node(*id) {
                    // Push this function scope onto the stack
                    self.function_scope_stack.push(func_scope_id);

                    // Walk the function body
                    self.walk_statement(body);

                    // Pop when done
                    self.function_scope_stack.pop();
                }
            }
            ExpressionNode::Identifier { id, .. } => {
                // Check if binding_id of this identifier is within our outside this scope
                // if it is out of this scope, then it has escaped, the signficance also is that once
                // a binding_id escapes for one identifier, it would have escaped for all
                // as it now lives on the heap.
                if let Some(binding_id) = self.symbol_table.resolve(*id) {
                    if let Some(symbol) = self.symbol_table.get_symbol(binding_id) {
                        // Check if we're inside a function
                        if let Some(&current_func_scope) = self.function_scope_stack.last() {
                            // If the variable was declared in an outer scope, it escapes!
                            // note: even thought there is 2 enter escope
                            // calls for a function, once when params are identified
                            // and one for the block body
                            // this logic still works as current_func_scope is only
                            // set when it etners a func and not in blocks
                            if symbol.scope_id < current_func_scope {
                                self.escapes.insert(binding_id, true);
                            }
                        }
                    }
                }
            }
            ExpressionNode::Infix { left, right, .. } => {
                self.walk_expression(left);
                self.walk_expression(right);
            }

            ExpressionNode::Prefix { right, .. } => {
                self.walk_expression(right);
            }

            ExpressionNode::Call {
                function,
                arguments,
                ..
            } => {
                self.walk_expression(function);
                for arg in arguments {
                    self.walk_expression(arg);
                }
            }

            ExpressionNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                self.walk_expression(condition);
                self.walk_statement(if_block);
                if let Some(block) = else_block {
                    self.walk_statement(block);
                }
            }

            ExpressionNode::Index { object, index, .. } => {
                self.walk_expression(object);
                self.walk_expression(index);
            }

            ExpressionNode::Array { elements, .. } => {
                for el in elements {
                    self.walk_expression(el);
                }
            }
            ExpressionNode::HashMap { pairs, .. } => {
                for (k, v) in pairs {
                    self.walk_expression(k);
                    self.walk_expression(v);
                }
            }
            ExpressionNode::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.walk_expression(condition);
                self.walk_expression(then_expr);
                self.walk_expression(else_expr);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use wasmprinter::print_bytes;

    use super::*;
    use crate::ast::AstNode;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::symbol_table::Symbol;

    fn get_ast(input: &str) -> Result<Node, String> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    #[test]
    fn test_no_escape_simple_locals() {
        let input = "
        let x = 1;
        let y = 2;
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x (binding 0) and y (binding 1) don't escape - no nested functions
        assert_eq!(false, ea.does_escape(0), "x should not escape");
        assert_eq!(false, ea.does_escape(1), "y should not escape");
    }

    #[test]
    fn test_simple_escape_captured_by_nested_function() {
        let input = "
        let x = 1;
        let y = 2;
        func test() {
            let z = x;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x (binding 0) escapes - captured by test()
        // y (binding 1) does NOT escape - not used in test()
        assert_eq!(
            true,
            ea.does_escape(0),
            "x should escape (captured by test)"
        );
        assert_eq!(
            false,
            ea.does_escape(1),
            "y should not escape (not captured)"
        );
    }

    #[test]
    fn test_parameter_escape() {
        let input = "
        func outer(x) {
            func inner() {
                return x + 1;
            };
            return inner;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x is a parameter (binding 1) and escapes - captured by inner()
        assert_eq!(
            true,
            ea.does_escape(1),
            "parameter x should escape (captured by inner)"
        );
    }

    #[test]
    fn test_nested_functions_multiple_levels() {
        let input = "
        let a = 1;
        func outer() {
            let b = 2;
            func middle() {
                let c = 3;
                func inner() {
                    return a + b + c;
                };
                return inner;
            };
            return middle;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // a (binding 0) escapes - captured by inner()
        // b (binding 2) escapes - captured by inner()
        // c (binding 4) escapes - captured by inner()
        assert_eq!(true, ea.does_escape(0), "a should escape");
        assert_eq!(true, ea.does_escape(2), "b should escape");
        assert_eq!(true, ea.does_escape(4), "c should escape");
    }

    #[test]
    fn test_mixed_escape_and_local() {
        let input = "
        func outer() {
            let x = 1;
            let y = 2;
            let z = 3;

            func inner() {
                return x + y;
            };

            return z;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x (binding 1) escapes - used in inner()
        // y (binding 2) escapes - used in inner()
        // z (binding 3) does NOT escape - only used in outer(), not in inner()
        assert_eq!(true, ea.does_escape(1), "x should escape");
        assert_eq!(true, ea.does_escape(2), "y should escape");
        assert_eq!(false, ea.does_escape(3), "z should not escape");
    }

    #[test]
    fn test_closure_counter_pattern() {
        let input = "
        func makeCounter() {
            let count = 0;
            func increment() {
                count = count + 1;
                return count;
            };
            return increment;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // count (binding 1) escapes - captured by increment()
        assert_eq!(
            true,
            ea.does_escape(1),
            "count should escape (classic closure pattern)"
        );
    }

    #[test]
    fn test_no_escape_same_scope_usage() {
        let input = "
        func test() {
            let x = 1;
            let y = 2;
            return x + y;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x (binding 1) and y (binding 2) do NOT escape - only used within test(), not captured
        assert_eq!(false, ea.does_escape(1), "x should not escape");
        assert_eq!(false, ea.does_escape(2), "y should not escape");
    }

    #[test]
    fn test_multiple_uses_same_escape_status() {
        let input = "
        let x = 10;
        func foo() {
            let a = x;
            let b = x + 1;
            let c = x + 2;
            return a + b + c;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x (binding 0) escapes - multiple uses in foo() all mark it as escaped
        assert_eq!(
            true,
            ea.does_escape(0),
            "x should escape (used multiple times in foo)"
        );
    }

    #[test]
    fn test_function_in_if_block() {
        let input = "
        let x = 1;
        if (true) {
            func inner() {
                return x;
            };
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let ea = EscapeAnalysis::analyze(&root, &symbol_table);

        // x (binding 0) escapes - captured by inner() even though inner is in if block
        assert_eq!(
            true,
            ea.does_escape(0),
            "x should escape (captured by inner in if block)"
        );
    }
}
