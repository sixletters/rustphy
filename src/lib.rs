// Core modules
pub mod ast;
pub mod bytecode_compiler;
pub mod compiler;
pub mod environment;
pub mod instruction;
pub mod lexer;
pub mod machine;
pub mod parser;
pub mod parser_v2;
pub mod symbol_table;
pub mod token;
pub mod wasm_environment;
pub mod wasm_simple_compiler;

// Re-export commonly used types for easier API access
pub use ast::{ExpressionNode, Node, StatementNode};
pub use bytecode_compiler::Compiler as BytecodeCompiler;
pub use compiler::Compiler as WasmCompiler;
pub use lexer::Lexer;
pub use machine::Machine;
pub use parser::Parser;
pub use wasm_simple_compiler::WasmSimpleCompiler;

// WASM bindings (only compiled when targeting wasm32)
#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    #[wasm_bindgen]
    pub fn run_rustphy_code(source_code: String) -> Result<String, JsValue> {
        // Set panic hook for better error messages in browser
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let lexer = Lexer::new(source_code);
        let mut parser = Parser::new(lexer);

        let program = parser
            .parse_program()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        let mut bytecode_compiler = BytecodeCompiler::new();
        let instructions = bytecode_compiler
            .compile(&program)
            .map_err(|e| JsValue::from_str(&format!("Compile error: {}", e)))?;

        let mut vm = Machine::new();
        let result = vm
            .run(&instructions)
            .map_err(|e| JsValue::from_str(&format!("Runtime error: {}", e)))?;

        // Convert result to JSON value, then to string to return to JavaScript
        let json_value = result.to_json_value();
        Ok(serde_json::to_string(&json_value).unwrap_or_else(|_| "null".to_string()))
    }

    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

// High-level API for running Rustphy code
pub fn run(source_code: &str) -> Result<String, String> {
    let lexer = Lexer::new(source_code.to_string());
    let mut parser = Parser::new(lexer);

    let program = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut bytecode_compiler = BytecodeCompiler::new();
    let instructions = bytecode_compiler
        .compile(&program)
        .map_err(|e| format!("Compile error: {}", e))?;

    let mut vm = Machine::new();
    let result = vm
        .run(&instructions)
        .map_err(|e| format!("Runtime error: {}", e))?;

    Ok(format!("{:?}", result))
}

// Parse source code into an AST
pub fn parse(source_code: &str) -> Result<Node, String> {
    let lexer = Lexer::new(source_code.to_string());
    let mut parser = Parser::new(lexer);
    parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))
}

// Compile to bytecode instructions
pub fn compile_bytecode(program: &Node) -> Result<Vec<instruction::Instruction>, String> {
    let mut compiler = BytecodeCompiler::new();
    compiler
        .compile(program)
        .map_err(|e| format!("Compile error: {}", e))
}

// Compile to WASM (WAT format)
pub fn compile_wasm(program: &Node) -> Result<String, String> {
    let mut compiler = WasmCompiler::new(program);
    compiler
        .compile()
        .map_err(|e| format!("Compile error: {}", e))
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
