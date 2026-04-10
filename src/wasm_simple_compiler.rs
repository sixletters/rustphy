//! WebAssembly compiler for the Goophy language.
//!
//! Compiles AST → WAT (WebAssembly Text format) → compile with wat2wasm → WASM binary
//!
//! # Learning Resources
//!
//! - MDN WAT Guide: https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format
//! - WAT Playground: https://webassembly.github.io/wabt/demo/wat2wasm/
//! - WASM Spec: https://webassembly.github.io/spec/core/text/index.html
//!
//! # Example WAT Output
//!
//! Input Goophy: `42`
//! Output WAT:
//! ```wat
//! (module
//!   (func $main (export "main") (result i32)
//!     i32.const 42
//!   )
//! )
//! ```
//!
//! Input Goophy: `2 + 3`
//! Output WAT:
//! ```wat
//! (module
//!   (func $main (export "main") (result i32)
//!     i32.const 2
//!     i32.const 3
//!     i32.add
//!   )
//! )
//! ```

use crate::ast::{ExpressionNode, InfixOp, Node, StatementNode};
use crate::wasm_environment::WasmRuntime;

use std::collections::HashMap;

/// Compiler errors
#[derive(Debug)]
pub enum WasmCompileError {
    Unsupported(String),
    Other(String),
}

impl std::fmt::Display for WasmCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmCompileError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
            WasmCompileError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for WasmCompileError {}

type Result<T> = std::result::Result<T, WasmCompileError>;

/// WebAssembly compiler
#[allow(dead_code)]
pub struct WasmSimpleCompiler {
    /// Map variable names to WASM local indices
    /// Example: {"x": 0, "y": 1}
    locals: HashMap<String, u32>,
    // Stack to keep track of locals
    locals_stack: Vec<HashMap<String, u32>>,

    /// Next available local index
    next_local: u32,

    /// Runtime for generating WAT code
    runtime: WasmRuntime,
    // function collection
    collected_function_runtimes: Vec<WasmRuntime>,
    // Compilation context
    context: CompilationContext,
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

struct CompilationContext {
    runtime: WasmRuntime,
    locals: HashMap<String, u32>,
    next_local: u32,
    parent: Option<Box<CompilationContext>>,
}

impl CompilationContext {
    fn new() -> Self {
        CompilationContext {
            runtime: WasmRuntime::new(),
            locals: HashMap::new(),
            next_local: 0,
            parent: None,
        }
    }

    fn push_scope(&mut self) -> CompilationContext {
        CompilationContext {
            runtime: WasmRuntime::new(),
            locals: HashMap::new(),
            next_local: 0,
            parent: Some(Box::new(std::mem::replace(self, CompilationContext::new()))),
        }
    }

    fn pop_scope(&mut self) -> WasmRuntime {
        let old_runtime = std::mem::replace(&mut self.runtime, WasmRuntime::new());
        if let Some(parent) = self.parent.take() {
            *self = *parent;
        }
        old_runtime
    }
}

impl WasmSimpleCompiler {
    pub fn new() -> Self {
        WasmSimpleCompiler {
            locals: HashMap::new(),
            next_local: 0,
            runtime: WasmRuntime::new(),
            collected_function_runtimes: vec![],
            locals_stack: vec![],
            context: CompilationContext::new(),
        }
    }

    /// Compile the AST program to WAT
    pub fn compile(&mut self, root_node: &Node) -> Result<String> {
        // module header
        self.runtime.emit_line("(module");
        self.runtime.increment_indent();

        // WASI imports and memory
        // self.runtime.emit_line("(import \"wasi_snapshot_preview1\" \"fd_write\" (func $fd_write (param i32 i32 i32 i32) (result i32)))");

        self.runtime.generate_tag_helpers();
        self.runtime.generate_arithmetic_helpers();

        // Start building the main function
        self.runtime.emit_line("(func $main (result i32)");
        self.runtime.increment_indent();

        // Compile the root node
        match root_node {
            Node::ExpressionNode(val) => {
                self.compile_expression(&val)?;
            }
            Node::StatementNode(val) => {
                self.compile_statement(&val)?;
            }
        }

        self.runtime.decrement_indent();
        self.runtime.emit_line(")");

        for func in self.collected_function_runtimes.iter() {
            self.runtime.emit(func.get_output());
        }

        self.runtime.emit_line("(func (export \"_start\")");
        self.runtime.increment_indent();
        self.runtime.emit_line("call $main");
        self.runtime.emit_line("drop");
        self.runtime.decrement_indent();
        self.runtime.emit_line(")");

        self.runtime.decrement_indent();
        self.runtime.emit_line(")");

        Ok(self.runtime.get_output().to_string())
    }

    /// Compile a statement
    #[allow(dead_code)]
    fn compile_statement(&mut self, stmt: &StatementNode) -> Result<()> {
        // TODO: Handle different statement types
        // - StatementNode::Let
        // - StatementNode::Return
        // - StatementNode::Expression
        // - StatementNode::Block
        // - etc.
        match stmt {
            StatementNode::Program {
                statements,
                implicit_return,
                ..
            } => {
                self.collect_locals_stmt(stmt);
                for (name, _) in self.locals.iter() {
                    self.runtime.emit_line(&format!("(local ${} i32)", name));
                }
                self.runtime.emit_newline();
                for node in statements {
                    match &**node {
                        Node::ExpressionNode(val) => self.compile_expression(val)?,
                        Node::StatementNode(val) => self.compile_statement(val)?,
                    }
                }
                match implicit_return {
                    Some(val) => self.compile_expression(val)?,
                    None => {}
                }
            }
            StatementNode::Block {
                statements,
                implicit_return,
                ..
            } => {
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
                match implicit_return {
                    Some(val) => self.compile_expression(val)?,
                    None => {}
                }
            }

            StatementNode::Let { value, name, .. } => {
                let identifier_name = match name {
                    ExpressionNode::Identifier { value, .. } => value,
                    _ => panic!("Name must be identifier"),
                };
                self.compile_expression(value)?;
                self.add_instruction(&format!("local.set ${}\n", identifier_name));
            }
            StatementNode::Expression { expression, .. } => {
                self.compile_expression(expression)?;
                // self.add_instruction("drop");
            }
            StatementNode::FuncDeclr {
                identifier, func, ..
            } => match (identifier, func) {
                (
                    ExpressionNode::Identifier { value, .. },
                    ExpressionNode::Function {
                        parameters, body, ..
                    },
                ) => {
                    let params: Vec<String> = parameters
                        .iter()
                        .filter_map(|f| match f.as_ref() {
                            ExpressionNode::Identifier { value, .. } => {
                                Some(format!("(param ${} i32)", value))
                            }
                            _ => None,
                        })
                        .collect();
                    let saved_runtime = self.runtime.clone();
                    let new_runtime = WasmRuntime::new();
                    self.runtime = new_runtime;

                    // For function declarations manually get the locals
                    self.locals_stack.push(self.locals.clone());
                    self.locals = HashMap::new();

                    self.runtime.set_indent_level(1);
                    self.collect_locals_stmt(body);
                    self.add_instruction(&format!(
                        "(func ${} {} (result i32)",
                        value,
                        params.join(" ")
                    ));
                    self.runtime.increment_indent();
                    for (name, _) in self.locals.iter() {
                        self.runtime.emit_line(&format!("(local ${} i32)", name));
                    }
                    // Compile function body
                    self.compile_statement(body)?;

                    self.runtime.decrement_indent();
                    self.add_instruction(")");
                    self.collected_function_runtimes.push(self.runtime.clone());
                    self.runtime = saved_runtime;
                    match self.locals_stack.pop() {
                        Some(val) => self.locals = val,
                        None => {
                            panic!("OH no i fcked up so")
                        }
                    };
                }
                _ => {}
            },
            StatementNode::Return { return_value, .. } => {
                self.compile_expression(return_value)?;
                self.add_instruction("return");
            }
            _ => {
                return Err(WasmCompileError::Unsupported(
                    "Statements not implemented".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Compile an expression (leaves result on WASM stack)
    #[allow(dead_code)]
    fn compile_expression(&mut self, expr: &ExpressionNode) -> Result<()> {
        // TODO: Handle different expression types
        // - ExpressionNode::Integer → i32.const
        // - ExpressionNode::Boolean → i32.const 0/1
        // - ExpressionNode::Identifier → local.get
        // - ExpressionNode::Infix → compile left, compile right, operator
        // - ExpressionNode::Prefix → compile operand, apply operator
        // - ExpressionNode::If → if/else/end blocks
        // - etc.
        match expr {
            ExpressionNode::Integer { value, .. } => {
                self.add_instruction(&format!("i32.const {}", value.to_string()));
                self.add_instruction("call $tag_int");
            }
            ExpressionNode::Boolean { value, .. } => {
                if *value {
                    self.add_instruction(&format!("i32.const 1"));
                } else {
                    self.add_instruction(&format!("i32.const 0"));
                }
                self.add_instruction("call $tag_bool");
            }
            ExpressionNode::Identifier { value, .. } => {
                self.add_instruction(&format!("local.get ${}", value));
            }
            ExpressionNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                self.compile_expression(condition)?;
                self.add_instruction("if");
                self.runtime.increment_indent();
                self.compile_statement(if_block)?;

                match else_block {
                    Some(val) => {
                        self.runtime.decrement_indent();
                        self.add_instruction("else");
                        self.runtime.increment_indent();
                        self.compile_statement(val)?;
                    }
                    _ => {}
                }
                self.runtime.decrement_indent();
                self.add_instruction("end");
            }
            ExpressionNode::Call {
                function,
                arguments,
                ..
            } => {
                // Right now the language does not support closures, but once it does we should
                // put the closure on the stack, right now lets just get the function as an identifier
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                match function.as_ref() {
                    ExpressionNode::Identifier { value, .. } => {
                        self.add_instruction(&format!("call ${}", value));
                    }
                    _ => {}
                }
            }
            ExpressionNode::Infix {
                left,
                operator,
                right,
                ..
            } => {
                if matches!(operator, InfixOp::Assign) {
                    match left.as_ref() {
                        ExpressionNode::Identifier { value, .. } => {
                            self.compile_expression(right)?;
                            // Use local.tee to set AND keep value on stack (for implicit returns)
                            self.add_instruction(&format!("local.set ${}", value));
                        }
                        _ => {}
                    }
                } else {
                    self.compile_expression(left)?;
                    self.compile_expression(right)?;

                    // Call the appropriate helper
                    match operator {
                        InfixOp::Add => self.add_instruction("call $add_values"),
                        InfixOp::Subtract => self.add_instruction("call $sub_values"),
                        InfixOp::Lt => self.add_instruction("call $lt_values"),
                        InfixOp::Gt => self.add_instruction("call $gt_values"),
                        InfixOp::Eq => self.add_instruction("call $eq_values"),
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn collect_locals_stmt(&mut self, stmt: &StatementNode) {
        match stmt {
            StatementNode::Block { statements, .. } => {
                let names = scan(statements);
                for name in names {
                    if !self.locals.contains_key(&name) {
                        self.locals.insert(name, self.next_local);
                        self.next_local += 1;
                    }
                }
            }
            StatementNode::Program { statements, .. } => {
                let stmts: Vec<Box<StatementNode>> = statements
                    .iter()
                    .filter_map(|node| match &**node {
                        Node::StatementNode(val) => Some(Box::new((*val).clone())),
                        _ => None,
                    })
                    .collect(); // ← Collect the iterator into a Vec
                let names = scan(&stmts);
                for name in names {
                    if !self.locals.contains_key(&name) {
                        self.locals.insert(name, self.next_local);
                        self.next_local += 1;
                    }
                }
            }
            StatementNode::Expression { expression, .. } => match expression {
                ExpressionNode::If {
                    if_block,
                    else_block,
                    ..
                } => {
                    self.collect_locals_stmt(if_block);
                    match else_block {
                        Some(val) => self.collect_locals_stmt(val),
                        None => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn add_instruction(&mut self, instr: &str) {
        self.runtime.emit_line(instr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile_code(code: &str) -> Result<String> {
        let lexer = Lexer::new(code.to_string());
        let mut parser = Parser::new(lexer);
        let ast = parser.parse_program().expect("Parse failed");

        let mut compiler = WasmSimpleCompiler::new();
        compiler.compile(&ast)
    }

    #[test]
    #[ignore] // Remove #[ignore] when you implement
    fn test_integer_literal() {
        let wat = compile_code("42").expect("Compile failed");
        assert!(wat.contains("i32.const 42"));
    }

    #[test]
    #[ignore]
    fn test_addition() {
        let wat = compile_code("1 + 2").expect("Compile failed");
        assert!(wat.contains("i32.const 1"));
        assert!(wat.contains("i32.const 2"));
        assert!(wat.contains("i32.add"));
    }

    #[test]
    #[ignore]
    fn test_variable() {
        let wat = compile_code("var x = 10; x").expect("Compile failed");
        assert!(wat.contains("local"));
    }
}
