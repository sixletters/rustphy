//! WebAssembly compiler for the Goophy language.
//!
//! Compiles AST → WAT (WebAssembly Text format) → compile with wat2wasm → WASM binary
//!

use crate::ast::{ExpressionNode, InfixOp, Node, PrefixOp, StatementNode};
use crate::escape_analysis::EscapeAnalysis;
use crate::symbol_table::{BindType, SymbolTable};
use crate::wasm_environment::WasmRuntime;
use std::str::FromStr;

use std::collections::HashMap;

// Define the enum for all built-in functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFunction {
    Log,
    Print,
    Len,
    Push,
    Pop,
}

impl FromStr for BuiltinFunction {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "log" => Ok(BuiltinFunction::Log),
            "print" => Ok(BuiltinFunction::Print),
            "len" => Ok(BuiltinFunction::Len),
            "push" => Ok(BuiltinFunction::Push),
            "pop" => Ok(BuiltinFunction::Pop),
            _ => Err(()),
        }
    }
}

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

/// WebAssembly compiler that transforms Rustphy AST into WAT (WebAssembly Text format).
///
/// The compiler handles:
/// - Variable scoping and closure capture
/// - Function compilation (both declarations and anonymous functions)
/// - String literal deduplication
/// - Escape analysis for determining heap vs stack allocation
pub struct Compiler<'a> {
    root_node: &'a Node,
    compiled_function_outputs: Vec<String>,
    compilation_context: CompilationContext,
    string_compilation_context: StringCompilationContext,
    // todo: make this cleaner and more optimized
    closures_to_register: Vec<String>,
    function_lookup_map: HashMap<String, usize>,
    closure_idx: usize,
    symbol_table: SymbolTable<'a>,
    escape_analysis: EscapeAnalysis,
    loop_counter: usize,
    // todo: currently compilation context is cloned, in the future might
    // be better to use rc and refcell?
    compilation_context_lookup: HashMap<usize, CompilationContext>, // binding_id -> compilation context
}

#[derive(Clone, Debug)]
struct CompilationContext {
    locals: Vec<usize>, // binding_id → local_index, we technically dont even need this honestly
    escaped_locals: HashMap<usize, u32>,
    next_escaped_variable: u32,
    parent: Option<Box<CompilationContext>>,
    depth: usize,
}

struct StringCompilationContext {
    collected_string_literals: HashMap<String, usize>,
    next_data_offset: usize,
    string_data: Vec<String>,
}

impl CompilationContext {
    fn new() -> Self {
        CompilationContext {
            locals: vec![],
            escaped_locals: HashMap::new(),
            next_escaped_variable: 0,
            parent: None,
            depth: 0,
        }
    }

    fn push_scope(&mut self) -> CompilationContext {
        let old_depth = self.depth;
        CompilationContext {
            locals: vec![],
            escaped_locals: HashMap::new(),
            next_escaped_variable: 0,
            parent: Some(Box::new(std::mem::replace(self, CompilationContext::new()))),
            depth: old_depth + 1,
        }
    }

    fn pop_scope(&mut self) {
        if let Some(parent) = self.parent.take() {
            *self = *parent;
        }
    }

    fn is_global(&self) -> bool {
        self.depth == 0
    }
}

impl<'a> Compiler<'a> {
    /// Creates a new WASM compiler for the given AST.
    ///
    /// Initializes symbol table and escape analysis for the program.
    pub fn new(root_node: &'a Node) -> Self {
        let symbol_table = SymbolTable::new(root_node);
        let mut escape_symbol_table = SymbolTable::new(root_node);
        escape_symbol_table.build();
        Compiler {
            root_node,
            compiled_function_outputs: vec![],
            closures_to_register: vec![],
            compilation_context: CompilationContext::new(),
            closure_idx: 0,
            function_lookup_map: HashMap::new(),
            string_compilation_context: StringCompilationContext {
                collected_string_literals: HashMap::new(),
                next_data_offset: 0,
                string_data: vec![],
            },
            loop_counter: 0,
            compilation_context_lookup: HashMap::new(),
            symbol_table: symbol_table,
            escape_analysis: EscapeAnalysis::analyze(root_node, &escape_symbol_table),
        }
    }

    fn compile_builtin_call(
        &mut self,
        builtin: BuiltinFunction,
        arguments: &[Box<ExpressionNode>],
    ) -> Result<String> {
        let mut runtime = WasmRuntime::new();
        match builtin {
            BuiltinFunction::Print => {
                for arg in arguments {
                    runtime.emit_line(&self.compile_expression(arg)?);
                }
                runtime.emit_line("call $print");
                runtime.emit_line("i32.const 0");
            }
            _ => {}
        }
        Ok(runtime.get_output().to_string())
    }

    /// Compiles the AST to WebAssembly Text (WAT) format.
    ///
    /// Returns a complete WASM module including:
    /// - Memory and table declarations
    /// - Runtime functions (environment, closures, type operations)
    /// - Compiled user functions
    /// - String data section
    pub fn compile(&mut self) -> Result<String> {
        // build the symbol table
        self.symbol_table.build();

        // Create a new runtime, just a helper to build wasm code
        let mut runtime = WasmRuntime::new();
        // module header
        runtime.emit_line("(module");

        // Import print function from JavaScript
        runtime.emit_line("(import \"env\" \"print\" (func $print (param i32)))");
        runtime.emit_newline();

        runtime.emit_line("(memory $heap (export \"memory\") 1)");
        runtime.emit_line("(global $heap_ptr (mut i32) (i32.const 1024))");
        runtime.emit_line("(global $global_env_ptr (mut i32) (i32.const 0))");
        runtime.emit_line("(type $function_type (func (param i32 i32) (result i32)))");
        runtime.emit_newline();
        runtime.emit_line("(table $closures 5 funcref)");
        runtime.emit_line("(global $TYPE_OBJECT (mut i32) (i32.const 3))");
        runtime.emit_line("(global $TYPE_CLOSURE (mut i32) (i32.const 2))");
        runtime.emit_line("(global $TYPE_ARRAY (mut i32) (i32.const 1))");
        runtime.emit_line("(global $TYPE_STRING (mut i32) (i32.const 0))");

        runtime.generate_heap_alloc();
        runtime.generate_tag_helpers();
        runtime.generate_object_helpers();
        runtime.generate_arg_helpers();
        runtime.generate_closure_helpers();
        runtime.generate_comparison_helpers();
        runtime.generate_env_helpers();
        runtime.generate_arithmetic_helpers();
        runtime.generate_string_helpers();
        runtime.generate_array_helpers();

        runtime.emit_line(&self.compile_node(self.root_node)?);

        if !self.closures_to_register.is_empty() {
            runtime.emit_line(&format!(
                "(elem (table $closures) (i32.const 0) func {} )",
                self.closures_to_register.join(" ")
            ));
        }
        // add the func_def and ouputs
        for func_def in self.compiled_function_outputs.iter() {
            runtime.emit_line(func_def);
        }
        for string_data in self.string_compilation_context.string_data.iter() {
            runtime.emit(string_data);
        }
        runtime.emit_line(")");

        Ok(runtime.get_output().to_string())
    }

    fn collect_variables_for_scope(&mut self, scope_id: usize) {
        // Get all bindings in this scope (including nested blocks)
        let all_bindings = self
            .symbol_table
            .get_all_bindings_in_function_scope(scope_id);

        // Filter out escaped bindings (they go in environment, not locals)
        // Filter out function params and also function declarations
        for binding_id in all_bindings {
            let current_symbol = match self.symbol_table.get_symbol(binding_id) {
                Some(val) => val,
                None => continue,
            };
            // If its not variable declaraion, then skip
            if !matches!(current_symbol.bind_type, BindType::VariableDeclaraion) {
                continue;
            }
            if self.escape_analysis.does_escape(binding_id) {
                self.compilation_context
                    .escaped_locals
                    .insert(binding_id, self.compilation_context.next_escaped_variable);
                self.compilation_context.next_escaped_variable += 1;
            } else {
                // This binding should be a WASM local
                self.compilation_context.locals.push(binding_id);
            }
            self.compilation_context_lookup
                .insert(binding_id, self.compilation_context.clone());
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
            StatementNode::Program { statements, id, .. } => {
                // Get the program's scope_id
                let scope_id = self
                    .symbol_table
                    .get_scope_for_node(*id)
                    .ok_or(WasmCompileError::Other("No scope for program".to_string()))?;

                // Collect locals for this scope
                self.collect_variables_for_scope(scope_id);

                // temporaily export main to test in browser
                runtime
                    .function("$main (export \"main\")")
                    .body(|f| {
                        // Declare locals using binding_ids
                        for binding_id in self.compilation_context.locals.iter() {
                            let symbol = self.symbol_table.get_symbol(*binding_id).unwrap();
                            f.push_inst(&format!("(local ${} i32)", symbol.name));
                        }
                        f.push_inst("global.get $global_env_ptr");
                        f.push_inst(&format!(
                            "i32.const {}",
                            self.compilation_context.escaped_locals.len()
                        ));
                        f.push_inst("call $create_env");
                        f.push_inst("global.set $global_env_ptr");

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
                let expr = self.compile_expression(value)?;
                if let ExpressionNode::Identifier { id, .. } = name {
                    let binding_id =
                        self.symbol_table
                            .resolve(*id)
                            .ok_or(WasmCompileError::Other(
                                "Unresolved let binding".to_string(),
                            ))?;
                    if self.escape_analysis.does_escape(binding_id) {
                        let env_var_idx =
                            match self.compilation_context.escaped_locals.get(&binding_id) {
                                Some(val) => val,
                                None => {
                                    return Err(WasmCompileError::Other(
                                        "Unable to find binding id in compilation context"
                                            .to_string(),
                                    ));
                                }
                            };
                        if self.compilation_context.is_global() {
                            runtime.emit_line("global.get $global_env_ptr");
                        } else {
                            runtime.emit_line("local.get $env_ptr");
                        }
                        runtime.emit_line(&format!("i32.const {}", env_var_idx));
                        runtime.emit_line(&expr);
                        // drop as env_set returns the value
                        runtime.emit_line("call $env_set");
                        runtime.emit_line("drop");
                        return Ok(runtime.get_output().to_string());
                    }
                    let symbol = self.symbol_table.get_symbol(binding_id).unwrap();
                    return Ok(format!("{}\nlocal.set ${}", expr, symbol.name));
                }
                Err(WasmCompileError::Other(
                    "Let name must be identifier".to_string(),
                ))
            }
            StatementNode::Expression { expression, .. } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit_line(&self.compile_expression(expression)?);
                // For statement expressions, as they have no effects elsewhere, it makes sense to drop
                runtime.emit_line("drop");
                Ok(runtime.get_output().to_string())
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
            } => {
                if let (ExpressionNode::Identifier { value, .. }, ExpressionNode::Function { .. }) = (identifier, func) {
                    self.compile_function_implementations(Some(value.to_string()), func)?;
                    Ok(String::new())
                } else {
                    Err(WasmCompileError::Other("FuncDeclr must have identifier and function".to_string()))
                }
            }
            StatementNode::For {
                condition,
                for_block,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                // use the wasm block/loop syntax to construct for loops
                // block -> structured forward jump
                // loop -> structured backward jump
                // it will screw up nested loops
                self.loop_counter += 1;
                runtime.emit_line(&format!("(block $break_{}", self.loop_counter));
                runtime.emit_line(&format!("(loop $continue_{}", self.loop_counter));
                // compile condition expression and leave on stack
                runtime.emit_line(&self.compile_expression(condition)?);
                // if condition is false then break
                runtime.emit_line("call $untag_immediate");
                runtime.emit_line("i32.eqz");
                runtime.emit_line(&format!("br_if $break_{}", self.loop_counter));
                runtime.emit_line(&self.compile_statement(for_block)?);
                runtime.emit_line(&format!("br $continue_{}", self.loop_counter));
                runtime.emit_line(")");
                runtime.emit_line(")");
                self.loop_counter -= 1;
                Ok(runtime.get_output().to_string())
            }
            StatementNode::Break { .. } => Ok(format!("br $break_{}", self.loop_counter)),
            StatementNode::Continue { .. } => Ok(format!("br $continue_{}", self.loop_counter)),
        }
    }
    pub fn compile_expression(&mut self, node: &ExpressionNode) -> Result<String> {
        match node {
            ExpressionNode::Integer { value, .. } => {
                Ok(format!("i32.const {}\ncall $tag_immediate", value))
            }
            ExpressionNode::Boolean { value, .. } => {
                let mut runtime = WasmRuntime::new();
                if *value {
                    runtime.emit_line("i32.const 1");
                } else {
                    runtime.emit_line("i32.const 0");
                }
                runtime.emit_line("call $tag_immediate");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Identifier { value, id, .. } => {
                let binding_id = self
                    .symbol_table
                    .resolve(*id)
                    .ok_or(WasmCompileError::Other(format!("Unresolved: {}", value)))?;

                match self.symbol_table.get_symbol(binding_id) {
                    Some(val) => {
                        if matches!(val.bind_type, BindType::FunctionDeclaration) {
                            // handle assignment of function declaration
                            let mut runtime = WasmRuntime::new();
                            let func_declr_idx = match self.function_lookup_map.get(value) {
                                Some(val) => val,
                                None => return Err(WasmCompileError::Other(format!("Function '{}' not found in lookup map", value))),
                            };
                            runtime.emit_line(&format!("i32.const {}", func_declr_idx));
                            if self.compilation_context.is_global() {
                                runtime.emit_line("global.get $global_env_ptr");
                            } else {
                                runtime.emit_line("local.get $env_ptr");
                            }
                            runtime.emit_line("call $create_closure");
                            return Ok(runtime.get_output().to_string());
                        }
                    }
                    None => {}
                }

                if self.escape_analysis.does_escape(binding_id) {
                    return self.generate_smart_pointer_loading(&binding_id);
                }

                // Load from WASM local
                let symbol =
                    self.symbol_table
                        .get_symbol(binding_id)
                        .ok_or(WasmCompileError::Other(format!(
                            "No symbol for binding: {}",
                            binding_id
                        )))?;
                Ok(format!("local.get ${}", symbol.name))
            }
            ExpressionNode::Prefix {
                operator, right, ..
            } => {
                let mut runtime = WasmRuntime::new();
                match operator {
                    PrefixOp::Negative => {
                        runtime.emit_line(&self.compile_expression(right)?);
                        // only makes sense to negate if it is a number anyways
                        // so call eimmediate is sage
                        runtime.emit_line("call $untag_immediate");
                        runtime.emit_line("i32.const 0");
                        runtime.emit_line("i32.sub");
                        runtime.emit_line("call $tag_immediate");
                    }
                    PrefixOp::Not => {
                        runtime.emit_line(&self.compile_expression(right)?);
                        runtime.emit_line("call $untag_immediate");
                        runtime.emit_line("i32.eqz");
                        runtime.emit_line("call $tag_immediate");
                    }
                }
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit_line(&self.compile_expression(condition)?);
                runtime.emit_line("call $untag_immediate");
                runtime.emit_line("if\n");
                runtime.emit_line(&self.compile_statement(if_block)?);
                if let Some(else_block) = else_block {
                    runtime.emit_line("else\n");
                    runtime.emit_line(&self.compile_statement(else_block)?);
                }
                runtime.emit_line("end");
                // this is added as all expression statements are dropped
                // so that it wouldnt cause an error;
                runtime.emit_line("i32.const 0");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Call {
                function,
                arguments,
                ..
            } => {
                let mut runtime = WasmRuntime::new();

                // Check if we can use direct call optimization
                if let ExpressionNode::Identifier { value, id, .. } = function.as_ref() {
                    // todo: Handle builtin functions. for now hardcode log to test
                    // has to be string btw
                    if let Ok(builtin) = BuiltinFunction::from_str(value) {
                        return self.compile_builtin_call(builtin, arguments);
                    }

                    let binding_id = match self.symbol_table.resolve(*id) {
                        Some(val) => val,
                        None => return Err(WasmCompileError::Other(format!("Unresolved function identifier: {}", value))),
                    };
                    let symbol = match self.symbol_table.get_symbol(binding_id) {
                        Some(val) => val,
                        None => return Err(WasmCompileError::Other(format!("Symbol not found for function: {}", value))),
                    };

                    if matches!(symbol.bind_type, BindType::FunctionDeclaration) {
                        // Direct call optimization
                        if self.compilation_context.is_global() {
                            runtime.emit_line("global.get $global_env_ptr");
                        } else {
                            runtime.emit_line("local.get $env_ptr");
                        }
                        for arg in arguments.iter() {
                            let expr = self.compile_expression(arg)?;
                            runtime.emit_line(&expr);
                        }
                        runtime.emit_line(&format!("call ${}_direct", value));
                        return Ok(runtime.get_output().to_string());
                    }
                }

                // Default closure call
                runtime.emit_line(&self.compile_expression(function)?);
                runtime.emit_line(&format!("i32.const {}", arguments.len()));
                runtime.emit_line("call $create_arg");
                for (i, arg) in arguments.iter().enumerate() {
                    runtime.emit(&format!("i32.const {}", i));
                    let expr = self.compile_expression(arg)?;
                    runtime.emit_line(&expr);
                    runtime.emit_line("call $arg_set");
                }
                runtime.emit_line("call $call_closure");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Infix {
                operator,
                right,
                left,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                if matches!(operator, InfixOp::Assign) {
                    // Compile the right side (the value to assign)
                    match left.as_ref() {
                        ExpressionNode::Identifier { value, id, .. } => {
                            let right_expr = self.compile_expression(right)?;
                            let binding_id =
                                self.symbol_table
                                    .resolve(*id)
                                    .ok_or(WasmCompileError::Other(
                                        "Unresolved let binding".to_string(),
                                    ))?;

                            if self.escape_analysis.does_escape(binding_id) {
                                return self
                                    .generate_smart_pointer_setting(&binding_id, right_expr);
                            }
                            runtime.emit_line(&right_expr);
                            runtime.emit_line(&format!("local.set ${}", value));
                            runtime.emit_line(&format!("local.get ${}", value));
                        }
                        ExpressionNode::Index { object, index, .. } => {
                            runtime.emit_line(&self.compile_expression(object)?);
                            runtime.emit_line(&self.compile_expression(index)?);
                            runtime.emit_line(&self.compile_expression(right)?);
                            runtime.emit_line("call $subscript_set");
                        }
                        _ => {}
                    }
                    return Ok(runtime.get_output().to_string());
                }

                // Handle short-cirtcuit operators BEFORE evaluating both sides
                if matches!(operator, InfixOp::And) {
                    runtime.emit_line(&self.compile_expression(left)?);
                    runtime.emit_line("call $untag_immediate");
                    runtime.emit_line("if (result i32)");
                    runtime.emit_line(&self.compile_expression(right)?);
                    runtime.emit_line("call $untag_immediate");
                    runtime.emit_line("else");
                    runtime.emit_line("i32.const 0");
                    runtime.emit_line("end");
                    runtime.emit_line("call $tag_immediate");
                    return Ok(runtime.get_output().to_string());
                }

                if matches!(operator, InfixOp::Or) {
                    runtime.emit_line(&self.compile_expression(left)?);
                    runtime.emit_line("call $untag_immediate");
                    runtime.emit_line("if (result i32)");
                    runtime.emit_line("i32.const 1");
                    runtime.emit_line("else");
                    runtime.emit_line(&self.compile_expression(right)?);
                    runtime.emit_line("call $untag_immediate");
                    runtime.emit_line("end");
                    runtime.emit_line("call $tag_immediate");
                    return Ok(runtime.get_output().to_string());
                }

                // handle not assign case
                let left_expr = self.compile_expression(left)?;
                let right_expr = self.compile_expression(right)?;
                let op_inst = match operator {
                    InfixOp::Add => "call $add_values",
                    InfixOp::Subtract => "call $sub_values",
                    InfixOp::Divide => "call $div_values",
                    InfixOp::Multiply => "call $mul_values",
                    InfixOp::Lt => "call $lt_values",
                    InfixOp::Gt => "call $gt_values",
                    InfixOp::Eq => "call $eq_values",
                    InfixOp::NotEq => "call $ne_values",
                    // todo: add And and Or Operators
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
            ExpressionNode::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                let mut runtime = WasmRuntime::new();
                // for tenary, use if (result i32) to leave value on the stack
                runtime.emit_line(&self.compile_expression(condition)?);
                runtime.emit_line("if (result i32)\n");
                runtime.emit_line(&self.compile_expression(then_expr)?);
                runtime.emit_line("else\n");
                runtime.emit_line(&self.compile_expression(else_expr)?);
                runtime.emit_line("end");
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
            ExpressionNode::Function { .. } => {
                let assigned_idx = self.compile_function_implementations(None, node)?;
                let mut runtime = WasmRuntime::new();
                runtime.emit_line(&format!("i32.const {}", assigned_idx));
                if self.compilation_context.is_global() {
                    runtime.emit_line("global.get $global_env_ptr");
                } else {
                    runtime.emit_line("local.get $env_ptr");
                }
                runtime.emit_line("call $create_closure");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::Index { object, index, .. } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit_line(&self.compile_expression(object)?);
                runtime.emit_line(&self.compile_expression(index)?);
                runtime.emit_line("call $subscript_get");
                Ok(runtime.get_output().to_string())
            }
            ExpressionNode::HashMap { pairs, id: _, .. } => {
                let mut runtime = WasmRuntime::new();
                runtime.emit_line("call $create_object_empty");
                for (key, value) in pairs.iter() {
                    runtime.emit_line(&self.compile_expression(key)?);
                    runtime.emit_line(&self.compile_expression(value)?);
                    runtime.emit_line("call $object_set");
                }
                Ok(runtime.get_output().to_string())
            }
        }
    }

    fn get_identifier_compilation_context(&self, binding_id: &usize) -> Result<CompilationContext> {
        match self.compilation_context_lookup.get(binding_id) {
            Some(val) => Ok(val.clone()),
            None => Err(WasmCompileError::Other(String::from(
                "binding id not found",
            ))),
        }
    }

    fn generate_environment_loading(&self, binding_depth: usize) -> Result<String> {
        let mut runtime = WasmRuntime::new();

        let to_walk = self.compilation_context.depth - binding_depth;
        if self.compilation_context.is_global() {
            runtime.emit_line("global.get $global_env_ptr");
        } else {
            runtime.emit_line("local.get $env_ptr");
        }

        for _ in 0..to_walk {
            runtime.emit_line("i32.load");
        }
        return Ok(runtime.get_output().to_string());
    }

    fn generate_smart_pointer_loading(&self, binding_id: &usize) -> Result<String> {
        let mut runtime = WasmRuntime::new();

        let scope_that_identifier_was_binded_in =
            self.get_identifier_compilation_context(&binding_id)?;

        runtime.emit_line(
            &self.generate_environment_loading(scope_that_identifier_was_binded_in.depth)?,
        );

        match scope_that_identifier_was_binded_in
            .escaped_locals
            .get(&binding_id)
        {
            Some(offset) => {
                runtime.emit_line(&format!("i32.const {}\n", offset));
                runtime.emit_line("call $env_get");
                return Ok(runtime.get_output().to_string());
            }
            None => {
                return Err(WasmCompileError::Other(String::from(
                    "binding id unfound error",
                )));
            }
        }
    }

    fn generate_smart_pointer_setting(&self, binding_id: &usize, value: String) -> Result<String> {
        let mut runtime = WasmRuntime::new();

        let scope_that_identifier_was_binded_in =
            self.get_identifier_compilation_context(&binding_id)?;

        runtime.emit_line(
            &self.generate_environment_loading(scope_that_identifier_was_binded_in.depth)?,
        );

        match scope_that_identifier_was_binded_in
            .escaped_locals
            .get(&binding_id)
        {
            Some(offset) => {
                runtime.emit_line(&format!("i32.const {}\n", offset));
                runtime.emit_line(&value);
                runtime.emit_line("call $env_set");
                return Ok(runtime.get_output().to_string());
            }
            None => {
                return Err(WasmCompileError::Other(String::from(
                    "binding id unfound error",
                )));
            }
        }
    }

    pub fn compile_direct_function(
        &mut self,
        function_identifier: String,
        function_expression: &ExpressionNode,
    ) -> Result<String> {
        if let ExpressionNode::Function {
            parameters, body, ..
        } = function_expression
        {
            // Create a new runtime, just a helper to build wasm code
            let mut runtime = WasmRuntime::new();

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
                .param("$env_ptr", "i32")
                .params(&params)
                .body(|f| {
                    // Declare the local variables
                    for binding_id in self.compilation_context.locals.iter() {
                        let symbol = self.symbol_table.get_symbol(*binding_id).unwrap();
                        f.push_inst(&format!("(local ${} i32)", symbol.name));
                    }

                    // to make it consistent throughout, all functions will create their own env and
                    // extend it, and then set the env_ptr inherited as the parent
                    f.push_inst("local.get $env_ptr");
                    f.push_inst(&format!(
                        "i32.const {}",
                        self.compilation_context.escaped_locals.len()
                    ));
                    f.push_inst("call $create_env");

                    // todo: this may thrown an error sometimes
                    let _ = self.compile_statement(&body).map(|u| {
                        f.push_inst(&u);
                    });
                })
                .result("i32")
                .build();

            return Ok(runtime.get_output().to_string());
        }
        Err(WasmCompileError::Other(String::from(
            "Non function node given",
        )))
    }

    fn compile_closure_function(
        &mut self,
        function_identifier: String,
        function_expression: &ExpressionNode,
    ) -> Result<String> {
        if let ExpressionNode::Function { parameters, .. } = function_expression {
            let mut runtime = WasmRuntime::new();
            runtime
                .func(&format!("${}_closure", function_identifier))
                .param("$env_ptr", "i32")
                .param("$arg_struct_ptr", "i32")
                .result("i32")
                .body(|f| {
                    f.push_inst("local.get $env_ptr");
                    for i in 0..parameters.len() {
                        f.push_inst("local.get $arg_struct_ptr");
                        f.push_inst(&format!("i32.const {}", i));
                        f.push_inst("call $arg_get");
                    }
                    f.push_inst(&format!("call ${}_direct", function_identifier));
                })
                .build();
            self.closures_to_register
                .push(format!("${}_closure", function_identifier));
            return Ok(runtime.get_output().to_string());
        }
        Err(WasmCompileError::Other(String::from(
            "Non function node given",
        )))
    }

    pub fn compile_function_implementations(
        &mut self,
        // If function identifier is none, then it is an anonymous function
        function_identifier: Option<String>,
        function_expression: &ExpressionNode,
    ) -> Result<usize> {
        match function_expression {
            ExpressionNode::Function { id, .. } => {
                let assigned_idx = self.closure_idx;
                // Push scope on the compilation_context
                self.compilation_context = self.compilation_context.push_scope();
                // Get the function's id
                let scope_id = self
                    .symbol_table
                    .get_scope_for_node(*id)
                    .ok_or(WasmCompileError::Other("No scope for function".to_string()))?;
                self.collect_variables_for_scope(scope_id);

                let function_name = match function_identifier {
                    Some(val) => val,
                    None => format!("lambda_{}", assigned_idx),
                };

                let direct_function =
                    self.compile_direct_function(function_name.clone(), function_expression)?;
                let closure_function =
                    self.compile_closure_function(function_name.clone(), function_expression)?;
                self.compiled_function_outputs.push(direct_function);
                self.compiled_function_outputs.push(closure_function);
                self.compilation_context.pop_scope();
                self.function_lookup_map
                    .insert(function_name.clone(), self.closure_idx);
                self.closure_idx += 1;
                Ok(assigned_idx)
            }
            _ => Err(WasmCompileError::Other(String::from(
                "function definition not expression",
            ))),
        }
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
        let ast = parser.parse_program().unwrap();
        let mut compiler = Compiler::new(&ast);
        compiler.compile()
    }

    #[test]
    fn test_compile_integer() {
        let wat = compile_code("let x = 42;").unwrap();
        assert!(wat.contains("i32.const 42"));
        assert!(wat.contains("call $tag_immediate"));
    }

    #[test]
    fn test_compile_arithmetic() {
        let wat = compile_code("let x = 10 + 5;").unwrap();
        assert!(wat.contains("i32.const 10"));
        assert!(wat.contains("i32.const 5"));
        assert!(wat.contains("call $add_values"));
    }

    #[test]
    fn test_compile_print() {
        let wat = compile_code("print(42);").unwrap();
        assert!(wat.contains("call $print"));
        assert!(wat.contains("i32.const 42"));
    }

    #[test]
    fn test_compile_function_declaration() {
        let wat = compile_code("func add(a, b) { return a + b; };").unwrap();
        assert!(wat.contains("add_direct"));
        assert!(wat.contains("call $add_values"));
    }

    #[test]
    fn test_compile_string_literal() {
        let wat = compile_code("let s = \"hello\";").unwrap();
        assert!(wat.contains("call $create_string"));
        assert!(wat.contains("\"hello\""));
    }

    #[test]
    fn test_compile_boolean() {
        let wat = compile_code("let t = true;").unwrap();
        assert!(wat.contains("i32.const 1"));
        assert!(wat.contains("call $tag_immediate"));

        let wat_false = compile_code("let f = false;").unwrap();
        assert!(wat_false.contains("i32.const 0"));
    }

    #[test]
    fn test_compile_if_statement() {
        let wat = compile_code("if (true) { let x = 1; };").unwrap();
        assert!(wat.contains("if"));
        assert!(wat.contains("end"));
    }

    #[test]
    fn test_compile_for_loop() {
        let wat = compile_code("let i = 0; for (i < 10) { print(i); i += 1; };").unwrap();
        assert!(wat.contains("block $break_"));
        assert!(wat.contains("loop $continue_"));
        assert!(wat.contains("call $print"));
    }

    #[test]
    fn test_compile_array() {
        let wat = compile_code("let arr = [1, 2, 3];").unwrap();
        assert!(wat.contains("call $create_array_empty"));
        assert!(wat.contains("call $array_set"));
    }

    #[test]
    fn test_compile_index_access() {
        let wat = compile_code("let arr = [1, 2]; let x = arr[0];").unwrap();
        assert!(wat.contains("call $subscript_get"));
    }

}
