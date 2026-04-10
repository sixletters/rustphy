//! Bytecode compiler for the Goophy language.
//!
//! This module compiles an Abstract Syntax Tree (AST) into bytecode instructions
//! that can be executed by the virtual machine.
//!
//! # Architecture
//!
//! The compiler performs a single-pass traversal of the AST, emitting bytecode
//! instructions and tracking:
//! - **Word Counter (wc)**: The current instruction address for jump resolution
//! - **Instructions**: The sequence of bytecode instructions generated
//!
//! # Example
//!
//! ```
//! use rustphy::bytecode_compiler::Compiler;
//! use rustphy::lexer::Lexer;
//! use rustphy::parser::Parser;
//!
//! let code = "let x = 42;";
//! let lexer = Lexer::new(code.to_string());
//! let mut parser = Parser::new(lexer);
//! let ast = parser.parse_program().unwrap();
//!
//! let mut compiler = Compiler::new();
//! let instructions = compiler.compile(&ast).unwrap();
//! ```

use crate::{
    ast::{ExpressionNode, InfixOp, Node, StatementNode},
    instruction::Instruction,
};

#[cfg(test)]
use crate::instruction::BINOPS;

/// The bytecode compiler that transforms AST nodes into executable instructions.
///
/// Maintains state during compilation including the instruction list and
/// the word counter for address resolution.
pub struct Compiler {
    /// Word counter - tracks the current instruction address for jump targets.
    wc: i128,

    /// The generated sequence of bytecode instructions.
    instructions: Vec<Instruction>,
    // Loop context stack
    loop_context_stack: Vec<LoopContext>,
}

struct LoopContext {
    continue_addr: usize,
    break_patchs: Vec<usize>,
}

impl Compiler {
    /// Creates a new compiler with an empty instruction list.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustphy::bytecode_compiler::Compiler;
    /// let compiler = Compiler::new();
    /// ```
    pub fn new() -> Self {
        Compiler {
            wc: 0,
            instructions: vec![],
            loop_context_stack: vec![],
        }
    }
}

/// Scans a list of statements and extracts all `let` binding names.
///
/// Used to determine which variables are local to a scope, allowing the VM
/// to pre-allocate environment slots for them.
///
/// # Arguments
///
/// * `statements` - The statements to scan for variable declarations
///
/// # Returns
///
/// A vector of variable names declared with `let` statements.
///
/// # Examples
///
/// ```
/// // Given statements: let x = 1; let y = 2; return x + y;
/// // Returns: vec!["x", "y"]
/// ```
pub fn scan(statements: &Vec<Box<StatementNode>>) -> Vec<String> {
    statements
        .iter()
        .filter_map(|statement| match &**statement {
            StatementNode::Let { name, .. } => match name {
                ExpressionNode::Identifier { value, .. } => Some(value.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

impl Compiler {
    /// Compiles an AST node into a complete bytecode program.
    ///
    /// This is the main entry point for compilation. It compiles the root node
    /// and appends a DONE instruction to terminate execution.
    ///
    /// # Arguments
    ///
    /// * `root_node` - The root AST node to compile (typically a Program)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Instruction>)` - The compiled bytecode instructions
    /// * `Err(String)` - Compilation errors such as malformed AST or invalid operations
    ///
    /// # Examples
    ///
    /// ```
    /// use rustphy::bytecode_compiler::Compiler;
    /// use rustphy::lexer::Lexer;
    /// use rustphy::parser::Parser;
    ///
    /// let mut compiler = Compiler::new();
    /// let lexer = Lexer::new("let x = 42;".to_string());
    /// let mut parser = Parser::new(lexer);
    /// let ast = parser.parse_program().unwrap();
    /// let instructions = compiler.compile(&ast).unwrap();
    /// ```
    pub fn compile(&mut self, root_node: &Node) -> Result<Vec<Instruction>, String> {
        match root_node {
            Node::ExpressionNode(val) => {
                self.compile_expression(&val)?;
            }
            Node::StatementNode(val) => {
                self.compile_statement(&val)?;
            }
        }
        self.instructions.push(Instruction::DONE);
        self.wc += 1;
        Ok(self.instructions.clone())
    }

    /// Compiles a statement node into bytecode instructions.
    ///
    /// Handles all statement types including programs, blocks, loops, variable
    /// declarations, returns, and function declarations.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement node to compile
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Statement compiled successfully
    /// * `Err(String)` - Compilation errors such as invalid assignment targets
    pub fn compile_statement(&mut self, statement: &StatementNode) -> Result<(), String> {
        match statement {
            StatementNode::Program { statements, .. } => {
                self.instructions
                    .push(Instruction::ENTERSCOPE { syms: vec![] });
                self.wc += 1;
                for node in statements {
                    match &**node {
                        Node::ExpressionNode(val) => self.compile_expression(val)?,
                        Node::StatementNode(val) => self.compile_statement(val)?,
                    }
                }
                self.instructions.push(Instruction::EXITSCOPE);
                self.wc += 1;
                Ok(())
            }
            StatementNode::Break { token, .. } => {
                let top_ctx = match self.loop_context_stack.last_mut() {
                    Some(val) => val,
                    None => Err(format!("error, break invoked in non loop ctx"))?,
                };
                self.instructions.push(Instruction::GOTO {
                    addr: top_ctx.continue_addr,
                });
                top_ctx.break_patchs.push(self.wc as usize);
                self.wc += 1;
                Ok(())
            }
            StatementNode::Continue { token, .. } => {
                let top_ctx = match self.loop_context_stack.last() {
                    Some(val) => val,
                    None => Err(format!("error, continue invoked in non loop ctx"))?,
                };
                self.instructions.push(Instruction::GOTO {
                    addr: top_ctx.continue_addr,
                });
                self.wc += 1;
                Ok(())
            }
            StatementNode::Let { value, name, .. } => {
                self.compile_expression(value)?;
                match name {
                    ExpressionNode::Identifier { value, .. } => {
                        self.instructions
                            .push(Instruction::ASSIGN { sym: value.clone() });
                    }
                    _ => {
                        return Err(format!(
                            "Compilation error: 'let' statement must have an identifier as its name, got {:?}",
                            name
                        ));
                    }
                }
                // Assignment pushes the value onto the stack, so we need to pop it
                self.instructions.push(Instruction::POP);
                self.wc += 2;
                Ok(())
            }
            StatementNode::Return { return_value, .. } => {
                self.compile_expression(return_value)?;
                self.instructions.push(Instruction::RESET);
                self.wc += 1;
                Ok(())
            }
            StatementNode::Expression { expression, .. } => {
                self.compile_expression(expression)?;
                // Just to make sure that expression statements dont get pushed on the stack
                self.instructions.push(Instruction::POP);
                self.wc += 1;
                Ok(())
            }
            StatementNode::Block { statements, .. } => {
                let locals = scan(statements);
                self.instructions
                    .push(Instruction::ENTERSCOPE { syms: locals });
                self.wc += 1;
                for s in statements.iter() {
                    self.compile_statement(s)?;
                }
                self.instructions.push(Instruction::EXITSCOPE);
                self.wc += 1;
                Ok(())
            }
            StatementNode::For {
                condition,
                for_block,
                ..
            } => {
                let start_of_conditional_evaluation = self.wc;

                let loop_ctx = LoopContext {
                    continue_addr: self.wc as usize,
                    break_patchs: vec![],
                };
                self.loop_context_stack.push(loop_ctx);
                // Compile the conditional expression first
                self.compile_expression(condition)?;

                // Emit JOF with placeholder address
                self.instructions.push(Instruction::JOF {
                    addr: self.wc as usize,
                });
                let saved_jof_idx = self.wc;
                self.wc += 1;

                // Compile the loop body
                self.compile_statement(for_block)?;

                // Jump back to condition evaluation
                self.instructions.push(Instruction::GOTO {
                    addr: start_of_conditional_evaluation as usize,
                });
                self.wc += 1;

                // Patch the JOF to jump to the end of the loop
                self.instructions[saved_jof_idx as usize] = Instruction::JOF {
                    addr: self.wc as usize,
                };

                let top_ctx = match self.loop_context_stack.last() {
                    Some(val) => val,
                    None => return Err(format!("ERROR BRO")),
                };

                for break_idx in top_ctx.break_patchs.iter() {
                    self.instructions[*break_idx] = Instruction::GOTO {
                        addr: self.wc as usize,
                    };
                }
                Ok(())
            }
            StatementNode::FuncDeclr {
                identifier, func, ..
            } => {
                self.compile_expression(func)?;
                match identifier {
                    ExpressionNode::Identifier { value, .. } => {
                        self.instructions
                            .push(Instruction::ASSIGN { sym: value.clone() });
                        self.wc += 1;
                    }
                    _ => {
                        return Err(format!(
                            "Compilation error: function declaration must have an identifier, got {:?}",
                            identifier
                        ));
                    }
                }
                Ok(())
            }
        }
    }

    /// Compiles an expression node into bytecode instructions.
    ///
    /// Handles all expression types including literals, identifiers, operators,
    /// conditionals, functions, and function calls.
    ///
    /// # Arguments
    ///
    /// * `expression` - The expression node to compile
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Expression compiled successfully
    /// * `Err(String)` - Compilation errors such as invalid assignment targets
    pub fn compile_expression(&mut self, expression: &ExpressionNode) -> Result<(), String> {
        match expression {
            ExpressionNode::Identifier { value, .. } => {
                self.instructions.push(Instruction::LDS {
                    sym: value.to_string(),
                });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::Integer { value, .. } => {
                self.instructions.push(Instruction::LDCN {
                    val: *value as i128,
                });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::String { value, .. } => {
                self.instructions
                    .push(Instruction::LDSL { val: value.clone() });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::Index { object, index, .. } => {
                self.compile_expression(object)?;
                self.compile_expression(index)?;
                self.instructions.push(Instruction::LDAI);
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::Prefix {
                operator, right, ..
            } => {
                // Compile the right operand first
                self.compile_expression(&right)?;
                // Convert PrefixOp to UNOPS using the From trait (.into())
                self.instructions.push(Instruction::UNOP {
                    ops: operator.clone().into(),
                });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::Array { elements, .. } => {
                // Compile array literal expression.
                //
                // Strategy:
                // 1. Compile each element expression in order (pushes values onto stack)
                // 2. Emit MKARR instruction with the number of elements
                // 3. MKARR will pop N values, create array, and push array onto stack
                //
                // Example: [1, 2 + 3, x]
                //   LDCN 1          // Stack: [1]
                //   LDCN 2          // Stack: [1, 2]
                //   LDCN 3          // Stack: [1, 2, 3]
                //   BINOP Add       // Stack: [1, 5]
                //   LDS "x"         // Stack: [1, 5, x_value]
                //   MKARR 3         // Stack: [[1, 5, x_value]]
                for e in elements.iter() {
                    self.compile_expression(e)?;
                }
                self.instructions.push(Instruction::MKARR {
                    size: elements.len(),
                });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::Infix {
                left,
                operator,
                right,
                ..
            } => {
                // Special handling for assignment: left side should be an identifier
                if matches!(operator, InfixOp::Assign) {
                    match left.as_ref() {
                        ExpressionNode::Identifier { value, .. } => {
                            // For assignment, push the identifier name itself (not its value)
                            self.instructions
                                .push(Instruction::LDI { val: value.clone() });
                            self.wc += 1;
                        }
                        ExpressionNode::Index { object, index, .. } => {
                            self.compile_expression(object)?;
                            self.compile_expression(index)?;
                            self.compile_expression(right)?;
                            self.instructions.push(Instruction::STAI);
                            self.wc += 1;
                            return Ok(());
                        }
                        _ => {
                            return Err(format!(
                                "Compilation error: assignment target must be an identifier, got {:?}",
                                left
                            ));
                        }
                    }
                    // Compile the right side (the value to assign)
                    self.compile_expression(&right)?;
                } else {
                    // For other binary operations, compile both sides normally
                    self.compile_expression(&left)?;
                    self.compile_expression(&right)?;
                }
                // Convert InfixOp to BINOPS using the From trait (.into())
                self.instructions.push(Instruction::BINOP {
                    ops: operator.clone().into(),
                });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::Boolean { value, .. } => {
                self.instructions.push(Instruction::LDCB { val: *value });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                // Compile if-else expression to bytecode.
                //
                // Control Flow Strategy:
                // 1. Evaluate condition (leaves boolean on stack)
                // 2. JOF (Jump On False) - conditional jump to else block or end
                // 3. If block - executed when condition is true
                // 4. GOTO - skip else block after if block executes
                // 5. Else block (optional) - executed when condition is false
                //
                // Bytecode Pattern (with else):
                //   <condition code>
                //   JOF else_addr       ; Jump to else if condition is false
                //   <if_block code>
                //   GOTO end_addr       ; Skip else block
                // else_addr:
                //   <else_block code>
                // end_addr:
                //
                // Bytecode Pattern (without else):
                //   <condition code>
                //   JOF end_addr        ; Jump past if block if condition is false
                //   <if_block code>
                // end_addr:

                // Step 1: Compile the condition expression
                self.compile_expression(condition)?;

                // Step 2: Emit JOF with placeholder address (will be patched later)
                let jof_idx = self.wc as usize;
                self.instructions.push(Instruction::JOF { addr: 0 });
                self.wc += 1;

                // Step 3: Compile the if block (executed when condition is true)
                self.compile_statement(if_block)?;

                match else_block {
                    Some(else_blk) => {
                        // With else block: need GOTO to skip else after if executes

                        // Step 4a: Emit GOTO with placeholder (skip else block)
                        let goto_idx = self.wc as usize;
                        self.instructions.push(Instruction::GOTO { addr: 0 });
                        self.wc += 1;

                        // Step 4b: Patch JOF to jump to start of else block
                        self.instructions[jof_idx] = Instruction::JOF {
                            addr: self.wc as usize,
                        };

                        // Step 5: Compile the else block
                        self.compile_statement(else_blk)?;

                        // Step 6: Patch GOTO to jump past the else block
                        self.instructions[goto_idx] = Instruction::GOTO {
                            addr: self.wc as usize,
                        };
                    }
                    None => {
                        // Without else block: JOF jumps directly past if block

                        // Patch JOF to jump to the instruction after the if block
                        self.instructions[jof_idx] = Instruction::JOF {
                            addr: self.wc as usize,
                        };
                    }
                }

                Ok(())
            }
            ExpressionNode::Function {
                parameters, body, ..
            } => {
                let mut params_to_string = vec![];
                for param in parameters.iter() {
                    match &**param {
                        ExpressionNode::Identifier { value, .. } => {
                            params_to_string.push(value.clone());
                        }
                        _ => {}
                    }
                }
                self.instructions.push(Instruction::LDF {
                    addr: (self.wc + 2) as usize,
                    params: params_to_string,
                });
                self.wc += 1;

                // Emit GOTO with placeholder address to skip function body
                self.instructions.push(Instruction::GOTO {
                    addr: (self.wc + 1) as usize,
                });
                let goto_idx = self.wc;
                self.wc += 1;
                self.compile_statement(body)?;

                // Implicit return at the end of function
                self.instructions.push(Instruction::RESET);
                self.wc += 1;

                // Patch the GOTO to jump past the function body
                self.instructions[goto_idx as usize] = Instruction::GOTO {
                    addr: self.wc as usize,
                };
                Ok(())
            }
            ExpressionNode::Call {
                function,
                arguments,
                ..
            } => {
                self.compile_expression(function)?;
                for arg in arguments.iter() {
                    self.compile_expression(arg)?;
                }
                self.instructions.push(Instruction::CALL {
                    arity: arguments.len(),
                });
                self.wc += 1;
                Ok(())
            }
            ExpressionNode::HashMap { pairs, .. } => {
                for pair in pairs.iter() {
                    self.compile_expression(&pair.0)?;
                    self.compile_expression(&pair.1)?;
                }
                self.instructions
                    .push(Instruction::MKHASH { size: pairs.len() });
                self.wc += 1;
                Ok(())
            }
            _ => Err(String::from("what ru doing bruh")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_compile_assignment_expression() {
        // Test that x = 1 generates LDI (not LDS) for the identifier
        let input = "let x = 0; x = 42;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have: ENTERSCOPE, LDCN(0), ASSIGN(x), POP, LDI(x), LDCN(42), BINOP(Assign), EXITSCOPE, DONE
        let first_ldcn = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDCN { .. }))
            .unwrap();

        assert!(matches!(
            instructions[first_ldcn],
            Instruction::LDCN { val: 0 }
        ));
        assert!(matches!(
            &instructions[first_ldcn + 1],
            Instruction::ASSIGN { sym } if sym == "x"
        ));
        assert!(matches!(instructions[first_ldcn + 2], Instruction::POP));
        // This is the key test: assignment expression should use LDI
        assert!(matches!(
            &instructions[first_ldcn + 3],
            Instruction::LDI { val } if val == "x"
        ));
        assert!(matches!(
            instructions[first_ldcn + 4],
            Instruction::LDCN { val: 42 }
        ));
        assert!(matches!(
            &instructions[first_ldcn + 5],
            Instruction::BINOP {
                ops: BINOPS::Assign
            }
        ));
    }

    #[test]
    fn test_compile_chained_assignment() {
        let input = "let x = 0; let y = 0; y = 5; x = 10;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Verify both assignments use LDI
        let y_assign = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDI { val } if val == "y"))
            .unwrap();

        assert!(matches!(
            &instructions[y_assign],
            Instruction::LDI { val } if val == "y"
        ));
        assert!(matches!(
            &instructions[y_assign + 1],
            Instruction::LDCN { val: 5 }
        ));
        assert!(matches!(
            &instructions[y_assign + 2],
            Instruction::BINOP {
                ops: BINOPS::Assign
            }
        ));

        let x_assign = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDI { val } if val == "x"))
            .unwrap();

        assert!(matches!(
            &instructions[x_assign],
            Instruction::LDI { val } if val == "x"
        ));
        assert!(matches!(
            &instructions[x_assign + 1],
            Instruction::LDCN { val: 10 }
        ));
        assert!(matches!(
            &instructions[x_assign + 2],
            Instruction::BINOP {
                ops: BINOPS::Assign
            }
        ));
    }

    #[test]
    fn test_compile_assignment_in_arithmetic() {
        // Test (x = 10) + 32 generates correct instructions
        let input = "let x = 0; x = 10 + 32;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // After let statement, should have: LDI(x), LDCN(10), LDCN(32), BINOP(Add), BINOP(Assign)
        let assign_start = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDI { val } if val == "x"))
            .unwrap();

        assert!(matches!(
            &instructions[assign_start],
            Instruction::LDI { val } if val == "x"
        ));
        assert!(matches!(
            &instructions[assign_start + 1],
            Instruction::LDCN { val: 10 }
        ));
        assert!(matches!(
            &instructions[assign_start + 2],
            Instruction::LDCN { val: 32 }
        ));
        assert!(matches!(
            &instructions[assign_start + 3],
            Instruction::BINOP { ops: BINOPS::Add }
        ));
        assert!(matches!(
            &instructions[assign_start + 4],
            Instruction::BINOP {
                ops: BINOPS::Assign
            }
        ));
    }

    #[test]
    fn test_compile_integer_literal() {
        let input = "42;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDCN { val: 42 }))
        );
    }

    #[test]
    fn test_compile_boolean_literals() {
        let input = "true; false;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDCB { val: true }))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDCB { val: false }))
        );
    }

    #[test]
    fn test_compile_string_literal() {
        let input = "\"hello world\";";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == "hello world"))
        );
    }

    #[test]
    fn test_compile_string_literal_empty() {
        let input = "\"\";";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == ""))
        );
    }

    #[test]
    fn test_compile_string_literal_with_escapes() {
        let input = "\"line1\\nline2\\ttab\";";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == "line1\nline2\ttab"))
        );
    }

    #[test]
    fn test_compile_string_in_let_statement() {
        let input = "let message = \"Hello, World!\";";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDSL instruction for the string
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == "Hello, World!"))
        );

        // Should have ASSIGN instruction for the variable
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::ASSIGN { sym } if sym == "message"))
        );
    }

    #[test]
    fn test_compile_multiple_strings() {
        let input = "\"first\"; \"second\"; \"third\";";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Count LDSL instructions
        let ldsl_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::LDSL { .. }))
            .count();

        assert_eq!(ldsl_count, 3);

        // Verify each string
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == "first"))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == "second"))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDSL { val } if val == "third"))
        );
    }

    #[test]
    fn test_compile_identifier() {
        let input = "let x = 5; x;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDS { sym } if sym == "x"))
        );
    }

    #[test]
    fn test_compile_arithmetic_operations() {
        let input = "5 + 3; 10 - 2; 4 * 2; 16 / 2;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Add }))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Minus }))
        );
        assert!(instructions.iter().any(|i| matches!(
            i,
            Instruction::BINOP {
                ops: BINOPS::Multiply
            }
        )));
        assert!(instructions.iter().any(|i| matches!(
            i,
            Instruction::BINOP {
                ops: BINOPS::Divide
            }
        )));
    }

    #[test]
    fn test_compile_comparison_operations() {
        let input = "let a = 5 < 3; let c = 5 > 3;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Lt }))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Gt }))
        );
    }

    #[test]
    fn test_compile_unary_negation() {
        let input = "-5;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(instructions.iter().any(|i| matches!(
            i,
            Instruction::UNOP {
                ops: crate::instruction::UNOPS::Negative
            }
        )));
    }

    #[test]
    fn test_compile_let_statement() {
        let input = "let x = 42;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDCN { val: 42 }))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::ASSIGN { sym } if sym == "x"))
        );
        assert!(instructions.iter().any(|i| matches!(i, Instruction::POP)));
    }

    #[test]
    fn test_compile_return_statement() {
        let input = "func f() { return 42; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let reset_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::RESET))
            .count();

        // Should have 2 RESETs: one from explicit return, one implicit at end of function
        assert_eq!(reset_count, 2);
    }

    #[test]
    fn test_compile_block_statement() {
        let input = "func f() { let x = 5; let y = 10; x + y; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let enterscope_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::ENTERSCOPE { .. }))
            .count();
        let exitscope_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::EXITSCOPE))
            .count();

        // Should have at least 1 ENTERSCOPE (program) and matching EXITSCOPE
        assert!(enterscope_count >= 1);
        assert_eq!(enterscope_count, exitscope_count);
    }

    #[test]
    fn test_compile_for_loop() {
        let input = "let x = 0; for (x < 5) { x = x + 1; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have JOF and GOTO for loop control
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::JOF { .. }))
        );
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::GOTO { .. }))
        );

        // Should have comparison operation
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Lt }))
        );
    }

    #[test]
    fn test_compile_function_declaration() {
        let input = "func add(x, y) { return x + y; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDF instruction with parameters
        let ldf_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDF { .. }))
            .unwrap();

        if let Instruction::LDF { params, .. } = &instructions[ldf_pos] {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "x");
            assert_eq!(params[1], "y");
        } else {
            panic!("Expected LDF instruction");
        }

        // Should have GOTO to skip function body
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::GOTO { .. }))
        );

        // Should have ASSIGN for function name
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::ASSIGN { sym } if sym == "add"))
        );
    }

    #[test]
    fn test_compile_function_call() {
        let input = "func f(x) { return x; }; f(42);";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have CALL instruction with arity 1
        let call_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::CALL { .. }))
            .unwrap();

        if let Instruction::CALL { arity } = instructions[call_pos] {
            assert_eq!(arity, 1);
        } else {
            panic!("Expected CALL instruction");
        }

        // Before CALL, should load function and argument
        assert!(matches!(
            &instructions[call_pos - 2],
            Instruction::LDS { sym } if sym == "f"
        ));
        assert!(matches!(
            instructions[call_pos - 1],
            Instruction::LDCN { val: 42 }
        ));
    }

    #[test]
    fn test_compile_closure() {
        let input = "let x = 10; func f(y) { return x + y; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDF instruction that captures environment
        assert!(
            instructions
                .iter()
                .any(|i| matches!(i, Instruction::LDF { .. }))
        );

        // Function body should reference x (captured variable)
        let ldf_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDF { .. }))
            .unwrap();

        // After LDF and GOTO, function body should have LDS for x
        let function_body_start = ldf_pos + 2;
        let has_x_reference = instructions[function_body_start..]
            .iter()
            .take_while(|i| !matches!(i, Instruction::DONE))
            .any(|i| matches!(i, Instruction::LDS { sym } if sym == "x"));

        assert!(has_x_reference);
    }

    #[test]
    fn test_compile_nested_expressions() {
        let input = "(5 + 3) * (10 - 2);";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Verify the operations exist
        let add_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Add }))
            .unwrap();
        let sub_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Minus }))
            .unwrap();
        let mul_pos = instructions
            .iter()
            .position(|i| {
                matches!(
                    i,
                    Instruction::BINOP {
                        ops: BINOPS::Multiply
                    }
                )
            })
            .unwrap();

        // ADD should come before SUB, and MUL should come last
        assert!(add_pos < sub_pos);
        assert!(sub_pos < mul_pos);
    }

    #[test]
    fn test_compile_multiple_statements() {
        let input = "let x = 5; let y = 10; let z = x + y;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have 3 ASSIGN instructions
        let assign_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::ASSIGN { .. }))
            .count();

        assert_eq!(assign_count, 3);

        // Should have POP instructions after assignments
        let pop_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::POP))
            .count();

        assert_eq!(pop_count, 3);
    }

    #[test]
    fn test_compile_higher_order_function() {
        let input = "func makeAdder(x) { func inner(y) { return x + y; }; return inner; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have 2 LDF instructions (outer and inner functions)
        let ldf_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::LDF { .. }))
            .count();

        assert_eq!(ldf_count, 2);

        // Should have multiple RESET instructions
        let reset_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::RESET))
            .count();

        assert!(reset_count >= 2);
    }

    #[test]
    fn test_compile_function_with_multiple_args() {
        let input = "func f(a, b, c) { return a + b + c; }; f(1, 2, 3);";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Find the CALL instruction
        let call_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::CALL { .. }))
            .unwrap();

        if let Instruction::CALL { arity } = instructions[call_pos] {
            assert_eq!(arity, 3);
        }

        // Find the LDF instruction
        let ldf_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDF { .. }))
            .unwrap();

        if let Instruction::LDF { params, .. } = &instructions[ldf_pos] {
            assert_eq!(params.len(), 3);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
            assert_eq!(params[2], "c");
        }
    }

    #[test]
    fn test_compile_nested_blocks() {
        let input = "func f() { let x = 1; for (x < 10) { let y = 2; x + y; }; };";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have at least 2 ENTERSCOPE (program and for loop block)
        let enterscope_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::ENTERSCOPE { .. }))
            .count();

        assert!(enterscope_count >= 2);

        // Should have matching EXITSCOPE count
        let exitscope_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::EXITSCOPE))
            .count();

        assert_eq!(exitscope_count, enterscope_count);
    }

    #[test]
    fn test_program_empty_returns_none() {
        use crate::machine::Machine;

        // Empty program should return None when evaluated
        let input = "";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        // Empty program returns None
        assert!(result.is_none());
    }

    #[test]
    fn test_program_let_then_expression_with_semicolon() {
        use crate::machine::Machine;

        // "let x = 5; x;" should pop the value, returning None
        let input = "let x = 5; x;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        // Expression with semicolon gets POPped, so eval returns None
        assert!(result.is_none());

        // Should have POPs from both the let statement and the expression statement
        let pop_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::POP))
            .count();
        assert!(pop_count >= 1);
    }
    // ===== Array Compilation Tests =====

    #[test]
    fn test_compile_empty_array() {
        let input = "[];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have: MKARR(0), DONE
        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { .. }))
            .expect("Should have MKARR instruction");

        assert!(matches!(
            instructions[mkarr_pos],
            Instruction::MKARR { size: 0 }
        ));
    }

    #[test]
    fn test_compile_single_element_array() {
        let input = "[42];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have: LDCN(42), MKARR(1), DONE
        let ldcn_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDCN { val: 42 }))
            .expect("Should have LDCN instruction");

        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { .. }))
            .expect("Should have MKARR instruction");

        assert!(matches!(
            instructions[ldcn_pos],
            Instruction::LDCN { val: 42 }
        ));
        assert!(matches!(
            instructions[mkarr_pos],
            Instruction::MKARR { size: 1 }
        ));
        assert_eq!(mkarr_pos, ldcn_pos + 1, "MKARR should follow LDCN");
    }

    #[test]
    fn test_compile_multiple_element_array() {
        let input = "[1, 2, 3];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have: LDCN(1), LDCN(2), LDCN(3), MKARR(3), DONE
        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { size: 3 }))
            .expect("Should have MKARR(3) instruction");

        // Check that we have three LDCN instructions before MKARR
        assert!(matches!(
            instructions[mkarr_pos - 3],
            Instruction::LDCN { val: 1 }
        ));
        assert!(matches!(
            instructions[mkarr_pos - 2],
            Instruction::LDCN { val: 2 }
        ));
        assert!(matches!(
            instructions[mkarr_pos - 1],
            Instruction::LDCN { val: 3 }
        ));
        assert!(matches!(
            instructions[mkarr_pos],
            Instruction::MKARR { size: 3 }
        ));
    }

    #[test]
    fn test_compile_mixed_type_array() {
        let input = "[1, true, \"hello\"];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have: LDCN(1), LDCB(true), LDSL("hello"), MKARR(3)
        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { size: 3 }))
            .expect("Should have MKARR(3) instruction");

        assert!(matches!(
            instructions[mkarr_pos - 3],
            Instruction::LDCN { val: 1 }
        ));
        assert!(matches!(
            instructions[mkarr_pos - 2],
            Instruction::LDCB { val: true }
        ));
        assert!(matches!(
            &instructions[mkarr_pos - 1],
            Instruction::LDSL { val } if val == "hello"
        ));
    }

    #[test]
    fn test_compile_array_with_expressions() {
        let input = "[1 + 1, 2 * 3];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should compile expressions first, then create array
        // LDCN(1), LDCN(1), BINOP(Add), LDCN(2), LDCN(3), BINOP(Mul), MKARR(2)
        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { size: 2 }))
            .expect("Should have MKARR(2) instruction");

        // Check for Add operation before MKARR
        let add_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Add }))
            .expect("Should have ADD operation");

        // Check for Multiply operation before MKARR
        let mul_pos = instructions
            .iter()
            .position(|i| {
                matches!(
                    i,
                    Instruction::BINOP {
                        ops: BINOPS::Multiply
                    }
                )
            })
            .expect("Should have MUL operation");

        assert!(add_pos < mkarr_pos, "Add should come before MKARR");
        assert!(mul_pos < mkarr_pos, "Multiply should come before MKARR");
        assert!(add_pos < mul_pos, "Add should come before Multiply");
    }

    #[test]
    fn test_compile_nested_arrays() {
        let input = "[[1, 2], [3, 4]];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have three MKARR instructions:
        // - Two for inner arrays (size 2 each)
        // - One for outer array (size 2)
        let mkarr_positions: Vec<usize> = instructions
            .iter()
            .enumerate()
            .filter_map(|(i, instr)| {
                if matches!(instr, Instruction::MKARR { .. }) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            mkarr_positions.len(),
            3,
            "Should have three MKARR instructions"
        );

        // All three should be MKARR with size 2
        assert!(matches!(
            instructions[mkarr_positions[0]],
            Instruction::MKARR { size: 2 }
        ));
        assert!(matches!(
            instructions[mkarr_positions[1]],
            Instruction::MKARR { size: 2 }
        ));
        assert!(matches!(
            instructions[mkarr_positions[2]],
            Instruction::MKARR { size: 2 }
        ));
    }

    #[test]
    fn test_compile_array_in_let_statement() {
        let input = "let arr = [1, 2, 3];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have: ENTERSCOPE, LDCN(1), LDCN(2), LDCN(3), MKARR(3), ASSIGN(arr), POP, EXITSCOPE
        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { size: 3 }))
            .expect("Should have MKARR instruction");

        let assign_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::ASSIGN { sym } if sym == "arr"))
            .expect("Should have ASSIGN instruction");

        assert!(mkarr_pos < assign_pos, "MKARR should come before ASSIGN");
        assert_eq!(
            mkarr_pos + 1,
            assign_pos,
            "ASSIGN should immediately follow MKARR"
        );
    }

    #[test]
    fn test_compile_array_with_variable_references() {
        let input = "let x = 1; let arr = [x, x + 1];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDS or LDI for variable reference
        let mkarr_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::MKARR { size: 2 }))
            .expect("Should have MKARR instruction");

        // Before MKARR, should have variable loads
        let has_var_load = instructions[..mkarr_pos].iter().any(|i| {
            matches!(i, Instruction::LDS { sym } if sym == "x")
                || matches!(i, Instruction::LDI { val } if val == "x")
        });

        assert!(has_var_load, "Should load variable x before creating array");
    }

    // ===== Array Indexing Compilation Tests =====
    //
    // These tests verify that array indexing expressions compile to correct bytecode.
    //
    // Bytecode Instructions:
    // - LDAI (Load Array Index): Pops index and array, pushes array[index]
    // - STAI (Store Array Index): Pops value, index, and array; stores value at array[index]
    //
    // Test Coverage:
    // - Reading from arrays (LDAI instruction generation)
    // - Writing to arrays (STAI instruction generation)
    // - Indexing with computed expressions
    // - Nested array indexing
    // - Array indexing in various contexts (let statements, expressions, etc.)
    // - Instruction ordering and stack operations

    #[test]
    fn test_compile_array_index_read() {
        let input = "let arr = [10, 20, 30]; arr[1];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDAI instruction for reading from array
        assert!(
            instructions.iter().any(|i| matches!(i, Instruction::LDAI)),
            "Should have LDAI instruction for array read"
        );

        // Find LDAI position and verify it comes after array and index loads
        let ldai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDAI))
            .expect("Should have LDAI instruction");

        // Before LDAI, should have loaded the array (LDS) and index (LDCN)
        let has_lds_before_ldai = instructions[..ldai_pos]
            .iter()
            .any(|i| matches!(i, Instruction::LDS { sym } if sym == "arr"));

        let has_ldcn_before_ldai = instructions[..ldai_pos]
            .iter()
            .any(|i| matches!(i, Instruction::LDCN { val: 1 }));

        assert!(has_lds_before_ldai, "Should load array symbol before LDAI");
        assert!(
            has_ldcn_before_ldai,
            "Should load index constant before LDAI"
        );
    }

    #[test]
    fn test_compile_array_index_write() {
        let input = "let arr = [1, 2, 3]; arr[0] = 99;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have STAI instruction for writing to array
        assert!(
            instructions.iter().any(|i| matches!(i, Instruction::STAI)),
            "Should have STAI instruction for array write"
        );

        // Find STAI position
        let stai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::STAI))
            .expect("Should have STAI instruction");

        // Before STAI, should have loaded array, index, and value
        let has_array_load = instructions[..stai_pos]
            .iter()
            .any(|i| matches!(i, Instruction::LDS { sym } if sym == "arr"));

        let has_index_load = instructions[..stai_pos]
            .iter()
            .any(|i| matches!(i, Instruction::LDCN { val: 0 }));

        let has_value_load = instructions[..stai_pos]
            .iter()
            .any(|i| matches!(i, Instruction::LDCN { val: 99 }));

        assert!(has_array_load, "Should load array before STAI");
        assert!(has_index_load, "Should load index before STAI");
        assert!(has_value_load, "Should load value before STAI");
    }

    #[test]
    fn test_compile_array_index_with_expression() {
        let input = "let arr = [1, 2, 3]; arr[1 + 1];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDAI instruction
        assert!(
            instructions.iter().any(|i| matches!(i, Instruction::LDAI)),
            "Should have LDAI instruction"
        );

        // Should have BINOP for the index expression (1 + 1)
        let ldai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDAI))
            .unwrap();

        let has_add_before_ldai = instructions[..ldai_pos]
            .iter()
            .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Add }));

        assert!(
            has_add_before_ldai,
            "Should have Add operation for index expression before LDAI"
        );
    }

    #[test]
    fn test_compile_nested_array_index() {
        let input = "let arr = [[1, 2], [3, 4]]; arr[0][1];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have two LDAI instructions for nested indexing
        let ldai_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::LDAI))
            .count();

        assert_eq!(
            ldai_count, 2,
            "Should have two LDAI instructions for nested indexing"
        );
    }

    #[test]
    fn test_compile_array_index_in_expression() {
        let input = "let arr = [10, 20, 30]; arr[0] + 5;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDAI followed by Add operation
        let ldai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDAI))
            .expect("Should have LDAI instruction");

        // After LDAI, should have LDCN(5) and then BINOP(Add)
        let has_add_after_ldai = instructions[ldai_pos..]
            .iter()
            .any(|i| matches!(i, Instruction::BINOP { ops: BINOPS::Add }));

        assert!(
            has_add_after_ldai,
            "Should have Add operation after LDAI for expression"
        );
    }

    #[test]
    fn test_compile_array_index_assignment_with_read() {
        let input = "let arr = [1, 2, 3]; arr[0] = arr[1] + 1;";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have one LDAI (for reading arr[1]) and one STAI (for writing arr[0])
        let ldai_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::LDAI))
            .count();
        let stai_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::STAI))
            .count();

        assert_eq!(ldai_count, 1, "Should have one LDAI for reading arr[1]");
        assert_eq!(stai_count, 1, "Should have one STAI for writing arr[0]");

        // LDAI should come before STAI (read happens before write)
        let ldai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDAI))
            .unwrap();
        let stai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::STAI))
            .unwrap();

        assert!(ldai_pos < stai_pos, "LDAI should come before STAI");
    }

    #[test]
    fn test_compile_array_index_in_let() {
        let input = "let arr = [10, 20, 30]; let x = arr[1];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDAI instruction
        assert!(
            instructions.iter().any(|i| matches!(i, Instruction::LDAI)),
            "Should have LDAI instruction"
        );

        // After LDAI, should have ASSIGN for 'x'
        let ldai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDAI))
            .unwrap();

        let has_assign_after = instructions[ldai_pos..]
            .iter()
            .any(|i| matches!(i, Instruction::ASSIGN { sym } if sym == "x"));

        assert!(has_assign_after, "Should have ASSIGN for 'x' after LDAI");
    }

    #[test]
    fn test_compile_multiple_array_accesses() {
        let input = "let arr = [1, 2, 3]; arr[0]; arr[1]; arr[2];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have three LDAI instructions
        let ldai_count = instructions
            .iter()
            .filter(|i| matches!(i, Instruction::LDAI))
            .count();

        assert_eq!(ldai_count, 3, "Should have three LDAI instructions");
    }

    #[test]
    fn test_compile_array_index_returns_value() {
        let input = "let arr = [42]; arr[0];";
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        // Should have LDAI instruction as part of implicit return
        let exitscope_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::EXITSCOPE))
            .unwrap();

        // LDAI should come before EXITSCOPE (part of implicit return)
        let ldai_pos = instructions
            .iter()
            .position(|i| matches!(i, Instruction::LDAI))
            .unwrap();

        assert!(
            ldai_pos < exitscope_pos,
            "LDAI should come before EXITSCOPE for implicit return"
        );
    }

    // ===== Integration Tests (End-to-End) =====
    //
    // These tests verify the complete pipeline: Parse → Compile → Execute
    // They test real Goophy code and verify the final execution results.
    //
    // Test Coverage:
    // - Basic array indexing operations
    // - Array element reads and writes
    // - Nested array access
    // - Array indexing in expressions and assignments
    // - Runtime error handling (out of bounds, type errors)
    // - Complex scenarios (swapping elements, mixed types, etc.)

    #[test]
    fn test_integration_array_index_read() {
        use crate::machine::Machine;

        // Test end-to-end: parse, compile, and execute array index read
        let code = "(func() {
            let arr = [10, 20, 30];
            return arr[1];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        // Should return 20 (element at index 1)
        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 20),
            _ => panic!("Expected Number(20), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_array_index_write() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [1, 2, 3];
            arr[0] = 99;
            return arr[0];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 99),
            _ => panic!("Expected Number(99), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_nested_array_index() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [[1, 2], [3, 4]];
            return arr[0][1];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 2),
            _ => panic!("Expected Number(2), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_array_index_with_expression() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [10, 20, 30];
            let i = 1;
            return arr[i + 1];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 30), // arr[1+1] = arr[2] = 30
            _ => panic!("Expected Number(30), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_array_assignment_chain() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [1, 2, 3];
            arr[0] = arr[1] + arr[2];
            return arr[0];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 5), // 2 + 3 = 5
            _ => panic!("Expected Number(5), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_array_index_in_expression() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [5, 10, 15];
            return arr[0] + arr[1] + arr[2];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 30), // 5 + 10 + 15 = 30
            _ => panic!("Expected Number(30), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_array_index_runtime_error_out_of_bounds() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [1, 2, 3];
            return arr[10];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.run(&instructions);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_integration_array_modify_multiple_times() {
        use crate::machine::Machine;

        // Simplified test without for loop to avoid loop variable complexity
        let code = "(func() {
            let arr = [0, 0, 0];
            arr[0] = 10;
            arr[1] = 20;
            arr[2] = 30;
            return arr[1];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 20),
            _ => panic!("Expected Number(20), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_array_swap_elements() {
        use crate::machine::Machine;

        let code = "(func() {
            let arr = [1, 2];
            let temp = arr[0];
            arr[0] = arr[1];
            arr[1] = temp;
            return arr[0];
        })()";

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::Number(n)) => assert_eq!(n, 2), // Swapped, arr[0] should be 2
            _ => panic!("Expected Number(2), got {:?}", result),
        }
    }

    #[test]
    fn test_integration_mixed_types_in_array() {
        use crate::machine::Machine;

        let code = r#"(func() {
            let arr = [42, true, "hello"];
            return arr[2];
        })()"#;

        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program_expression().unwrap();

        let mut compiler = Compiler::new();
        let instructions = compiler.compile(&program).unwrap();

        let mut vm = Machine::new();
        let result = vm.eval(&instructions).unwrap();

        match result {
            Some(crate::environment::Value::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected String(\"hello\"), got {:?}", result),
        }
    }
}
