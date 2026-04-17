use crate::bytecode_compiler::Compiler as BytecodeCompiler;
use crate::lexer::Lexer;
use crate::machine::Machine;
use crate::parser::Parser;

pub struct Repl {
    machine: Machine,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            machine: Machine::new(),
        }
    }

    pub fn eval_line(&mut self, input: &str) -> Result<String, String> {
        // Reset execution state but keep environment (variables)
        self.machine.reset_state();

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program_expression().map_err(|e| e.to_string())?;

        let mut bytecode_compiler = BytecodeCompiler::new();
        let instructions = bytecode_compiler
            .compile(&program)
            .map_err(|e| e.to_string())?;

        self.machine
            .eval(&instructions)
            .map_err(|e| e.to_string())
            .map(|result| format!("{:?}", result))
    }
}
