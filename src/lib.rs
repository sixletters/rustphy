// Core modules
pub mod ast;
pub mod bytecode_compiler;
pub mod environment;
pub mod escape_analysis;
pub mod instruction;
pub mod lexer;
pub mod machine;
pub mod parser;
pub mod repl;
pub mod symbol_table;
pub mod token;
pub mod wasm_compiler;
pub mod wasm_environment;

// Re-export commonly used types for easier API access
pub use ast::{ExpressionNode, Node, StatementNode};
pub use bytecode_compiler::Compiler as BytecodeCompiler;
pub use lexer::Lexer;
pub use machine::Machine;
pub use parser::Parser;
pub use wasm_compiler::Compiler as WasmCompiler;

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
        vm.run(&instructions)
            .map_err(|e| JsValue::from_str(&format!("Runtime error: {}", e)))?;

        // Return the captured output from print statements
        Ok(vm.get_output())
    }

    /// Compile Rustphy code to WebAssembly Text (WAT) format
    #[wasm_bindgen]
    pub fn compile_to_wasm(source_code: String) -> Result<String, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let lexer = Lexer::new(source_code);
        let mut parser = Parser::new(lexer);

        let program = parser
            .parse_program()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        let mut wasm_compiler = WasmCompiler::new(&program);
        let wat_output = wasm_compiler
            .compile()
            .map_err(|e| JsValue::from_str(&format!("Compile error: {}", e)))?;

        Ok(wat_output)
    }

    /// Parse Rustphy code and return the AST as JSON
    #[wasm_bindgen]
    pub fn parse_to_json(source_code: String) -> Result<String, JsValue> {
        let lexer = Lexer::new(source_code);
        let mut parser = Parser::new(lexer);

        let program = parser
            .parse_program()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        serde_json::to_string_pretty(&program)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Compile to bytecode and return as formatted string
    #[wasm_bindgen]
    pub fn compile_to_bytecode(source_code: String) -> Result<String, JsValue> {
        let lexer = Lexer::new(source_code);
        let mut parser = Parser::new(lexer);

        let program = parser
            .parse_program()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        let mut compiler = BytecodeCompiler::new();
        let instructions = compiler
            .compile(&program)
            .map_err(|e| JsValue::from_str(&format!("Compile error: {}", e)))?;

        let output = instructions
            .iter()
            .enumerate()
            .map(|(i, instr)| format!("{:04} {:?}", i, instr))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output)
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
