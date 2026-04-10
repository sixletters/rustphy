//! WebAssembly compiler for the Goophy language.
//!
//! Compiles AST → WAT (WebAssembly Text format) → compile with wat2wasm → WASM binary
//!

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

pub struct Compiler<'a> {
    root_node: &'a Node,
    compiled_function_outputs: Vec<String>,
    compilation_context: CompilationContext,
    string_compilation_context: StringCompilationContext,
}

struct CompilationContext {
    locals: HashMap<String, u32>,
    next_local: u32,
    parent: Option<Box<CompilationContext>>,
}

struct StringCompilationContext {
    collected_string_literals: HashMap<String, usize>,
    next_data_offset: usize,
    string_data: Vec<String>,
}

impl CompilationContext {
    fn new() -> Self {
        CompilationContext {
            locals: HashMap::new(),
            next_local: 0,
            parent: None,
        }
    }

    fn push_scope(&mut self) -> CompilationContext {
        CompilationContext {
            locals: HashMap::new(),
            next_local: 0,
            parent: Some(Box::new(std::mem::replace(self, CompilationContext::new()))),
        }
    }

    fn pop_scope(&mut self) {
        if let Some(parent) = self.parent.take() {
            *self = *parent;
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn new(root_node: &'a Node) -> Self {
        Compiler {
            root_node,
            compiled_function_outputs: vec![],
            compilation_context: CompilationContext::new(),
            string_compilation_context: StringCompilationContext {
                collected_string_literals: HashMap::new(),
                next_data_offset: 0,
                string_data: vec![],
            },
        }
    }

    // Call this compile first
    pub fn compile(&mut self) -> Result<String> {
        // Create a new runtime, just a helper to build wasm code
        let mut runtime = WasmRuntime::new();
        // module header
        runtime.emit_line("(module");

        runtime.emit_line("(memory $heap (export \"memory\") 1)");
        runtime.emit_line("(global $heap_ptr (mut i32) (i32.const 1024))");
        runtime.emit_line("(global $global_env (mut i32) (i32.const 0))");
        runtime.emit_line("(type $function_type (func (param i32 i32) (result i32)))");
        runtime.emit_newline();
        runtime.emit_line("(table $closures 5 funcref)");
        runtime.emit_line("(global $TYPE_CLOSURE (mut i32) (i32.const 2))");
        runtime.emit_line("(global $TYPE_ARRAY (mut i32) (i32.const 1))");
        runtime.emit_line("(global $TYPE_STRING (mut i32) (i32.const 0))");

        runtime.generate_heap_alloc();
        runtime.generate_tag_helpers();
        runtime.generate_arg_helpers();
        runtime.generate_closure_helpers();
        runtime.generate_comparison_helpers();
        runtime.generate_env_helpers();
        runtime.generate_arithmetic_helpers();
        runtime.generate_string_helpers();
        runtime.generate_array_helpers();

        runtime.emit_line(&self.compile_node(self.root_node)?);
        for func_def in self.compiled_function_outputs.iter() {
            runtime.emit_line(func_def);
        }
        for string_data in self.string_compilation_context.string_data.iter() {
            runtime.emit(string_data);
        }
        runtime.emit_line(")");

        Ok(runtime.get_output().to_string())
    }

    fn collect_locals_stmt(&mut self, stmt: &StatementNode) {
        match stmt {
            StatementNode::Block { statements, .. } => {
                let names = Self::scan(statements);
                for name in names {
                    if !self.compilation_context.locals.contains_key(&name) {
                        self.compilation_context
                            .locals
                            .insert(name, self.compilation_context.next_local);
                        self.compilation_context.next_local += 1;
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
                let names = Self::scan(&stmts);
                for name in names {
                    if !self.compilation_context.locals.contains_key(&name) {
                        self.compilation_context
                            .locals
                            .insert(name, self.compilation_context.next_local);
                        self.compilation_context.next_local += 1;
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

    pub fn compile_node(&mut self, node: &Node) -> Result<String> {
        match node {
            Node::ExpressionNode(val) => self.compile_expression(val),
            Node::StatementNode(val) => self.compile_statement(val),
        }
    }

    pub fn compile_statement(&mut self, node: &StatementNode) -> Result<String> {
        let mut runtime = WasmRuntime::new();
        match node {
            StatementNode::Program {
                statements,
                implicit_return,
                ..
            } => {
                self.collect_locals_stmt(node);
                // temporaily export main to test in browser
                // todo: this can actually be made to look prettier in the future
                runtime
                    .function("$main (export \"main\")")
                    .result("i32")
                    .body(|f| {
                        for (name, _) in self.compilation_context.locals.iter() {
                            // fix this
                            f.push_inst(&format!("(local ${} i32)", name));
                        }
                        for node in statements {
                            // todo this might be an error
                            let compiled_statement = match &**node {
                                Node::ExpressionNode(val) => self.compile_expression(val),
                                Node::StatementNode(val) => self.compile_statement(val),
                            }
                            .unwrap();
                            // Split by newlines and push each instruction separately
                            for line in compiled_statement.lines() {
                                if !line.trim().is_empty() {
                                    f.push_inst(line);
                                }
                            }
                        }
                        match implicit_return {
                            Some(val) => {
                                // could be an error here
                                let expr = self.compile_expression(val).unwrap();
                                for line in expr.lines() {
                                    if !line.trim().is_empty() {
                                        f.push_inst(line);
                                    }
                                }
                            }
                            None => {}
                        }
                    })
                    .build();
                Ok(runtime.get_output().to_string())
            }
            StatementNode::Return { return_value, .. } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit_line(&self.compile_expression(return_value)?);
                runtime.emit_line("return");
                Ok(runtime.get_output().to_string())
            }
            StatementNode::Let { value, name, .. } => {
                let identifier_name = match name {
                    ExpressionNode::Identifier { value, .. } => value,
                    _ => panic!("Name must be identifier"),
                };

                let expr = self.compile_expression(value)?;
                Ok(format!("{}\nlocal.set ${}", expr, identifier_name))
            }
            StatementNode::Expression { expression, .. } => {
                Ok(self.compile_expression(expression)?)
            }
            StatementNode::Block { statements, .. } => {
                // for now we do not consider a block as a scope by itself
                let mut runtime = WasmRuntime::new();
                for stmt in statements.iter() {
                    runtime.emit_line(&self.compile_statement(stmt)?);
                }
                Ok(runtime.get_output().to_string())
            }
            StatementNode::FuncDeclr {
                identifier, func, ..
            } => match (identifier, func) {
                (ExpressionNode::Identifier { value, .. }, ExpressionNode::Function { .. }) => {
                    self.compile_function_expression(value.to_string(), func)
                }
                _ => Ok(String::new()),
            },
            _ => Ok(String::new()),
        }
    }
    pub fn compile_expression(&mut self, node: &ExpressionNode) -> Result<String> {
        match node {
            ExpressionNode::Integer { value, .. } => {
                Ok(format!("i32.const {}\ncall $tag_immediate", value))
            }
            ExpressionNode::Boolean { value, .. } => {
                // todo: not sure if I have to actually call tag_immediate here
                // doesnt make sense for there to be a tag immediate here
                if *value {
                    return Ok("i32.const 1".to_string());
                }
                Ok("i32.const 0".to_string())
            }
            ExpressionNode::Identifier { value, .. } => {
                // todo: In the future we have to determine if this identifier is an escape variable etc
                // depending on its type, we have to compile acordingly
                // for now keep it as all local
                Ok(format!("local.get ${}", value))
            }
            ExpressionNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit_line(&self.compile_expression(condition)?);
                runtime.emit_line("if\n");
                runtime.emit_line(&self.compile_statement(if_block)?);
                if let Some(else_block) = else_block {
                    runtime.emit_line("else\n");
                    runtime.emit_line(&self.compile_statement(else_block)?);
                }
                runtime.emit_line("end");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Call {
                function,
                arguments,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                for arg in arguments.iter() {
                    let expr = self.compile_expression(arg)?;
                    runtime.emit_line(&expr);
                }
                // have to decide in the future if it is closure being called
                // or if it is direct function
                match function.as_ref() {
                    ExpressionNode::Identifier { value, .. } => {
                        runtime.emit_line(&format!("call ${}_direct", value));
                    }
                    _ => {}
                }
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Infix {
                operator,
                right,
                left,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                let left_expr = self.compile_expression(left)?;
                let right_expr = self.compile_expression(right)?;
                let op_inst = match operator {
                    // Finish implementing other kinds of infix operators
                    InfixOp::Add => "call $add_values",
                    _ => "",
                };
                runtime.emit_line(&left_expr);
                runtime.emit_line(&right_expr);
                runtime.emit_line(op_inst);
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::String { value, .. } => {
                // Proper escaping in the future
                // not right now though
                let mut runtime = WasmRuntime::new();
                let mut offset = self.string_compilation_context.next_data_offset;
                if let Some(index) = self
                    .string_compilation_context
                    .collected_string_literals
                    .get(value)
                {
                    offset = index.clone();
                } else {
                    self.string_compilation_context
                        .collected_string_literals
                        .insert(
                            value.clone(),
                            self.string_compilation_context.next_data_offset,
                        );
                    self.string_compilation_context.next_data_offset += value.len();
                    self.string_compilation_context
                        .string_data
                        .push(format!("(data (i32.const {}) \"{}\")", offset, value));
                }
                runtime.emit(&format!("i32.const {}", offset));
                runtime.emit(&format!("i32.const {}", value.len()));
                runtime.emit("call $create_string");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Array { elements, .. } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit(&format!("i32.const {}", elements.len()));
                runtime.emit(&format!("call $create_array_empty"));

                for (i, ele) in elements.iter().enumerate() {
                    runtime.emit(&format!("i32.const {}", i));
                    runtime.emit("call $tag_immediate");
                    let compiled_exp = self.compile_expression(ele)?;
                    runtime.emit(&compiled_exp);
                    runtime.emit("call $array_set");
                }
                Ok(runtime.get_output().to_string())
            }
            _ => Ok(String::new()),
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
    fn scan(statements: &Vec<Box<StatementNode>>) -> Vec<String> {
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

    pub fn compile_function_expression(
        &mut self,
        function_identifier: String,
        function_expression: &ExpressionNode,
    ) -> Result<String> {
        match function_expression {
            ExpressionNode::Function {
                parameters, body, ..
            } => {
                // Create a new runtime, just a helper to build wasm code
                let mut runtime = WasmRuntime::new();

                // Push scope on the compilation_context
                self.compilation_context.push_scope();

                // For functions we have to generate two different functions
                // mainly identifier_direct and identifier_closure
                // Generate direct
                let params: Vec<(String, String)> = parameters
                    .iter()
                    .filter_map(|f| match f.as_ref() {
                        ExpressionNode::Identifier { value, .. } => {
                            Some((format!("${}", value), "i32".to_string()))
                        }
                        _ => None,
                    })
                    .collect();

                // Generation of direct functions
                runtime
                    .function(&format!("${}_direct", function_identifier))
                    .params(&params)
                    .body(|f| {
                        let _ = self.compile_statement(&body).map(|u| {
                            f.push_inst(&u);
                        });
                    })
                    .result("i32")
                    .build();
                self.compiled_function_outputs
                    .push(runtime.get_output().to_string());
                self.compilation_context.pop_scope();

                let mut runtime = WasmRuntime::new();
                // Generation of closure
                self.compilation_context.push_scope();
                runtime
                    .func(&format!("${}_closure", function_identifier))
                    .param("$env_ptr", "i32")
                    .param("$arg_struct_ptr", "i32")
                    .result("i32")
                    .body(|f| {
                        f.push_inst("local.get $env_ptr");
                        params.iter().enumerate().for_each(|(i, _)| {
                            f.push_inst("local.get $arg_struct_ptr");
                            f.push_inst(&format!("i32.const {}", i));
                            f.push_inst("call $arg_get");
                        });
                        f.push_inst(&format!("call ${}_direct", function_identifier));
                    })
                    .result("i32")
                    .build();
                self.compiled_function_outputs
                    .push(runtime.get_output().to_string());
                self.compilation_context.pop_scope();
                Ok(String::new())
            }
            _ => Err(WasmCompileError::Other(String::from(
                "function definition not expression",
            ))),
        }
    }
}
