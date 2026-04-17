use rustphy::repl::Repl;
use rustphy::{VERSION, compile_bytecode, compile_wasm, parse, run};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    command: Command,
}

#[derive(Debug)]
enum Command {
    Repl,
    Run {
        file: Option<PathBuf>,
    },
    Parse {
        file: Option<PathBuf>,
    },
    Compile {
        file: Option<PathBuf>,
        target: CompileTarget,
    },
    Version,
    Help,
}

#[derive(Debug, Clone, Copy)]
enum CompileTarget {
    Bytecode,
    Wasm,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);

    let command = match args.next() {
        Some(cmd) => match cmd.as_str() {
            "repl" => Command::Repl,
            "run" => {
                let file = args.next().map(PathBuf::from);
                Command::Run { file }
            }
            "parse" => {
                let file = args.next().map(PathBuf::from);
                Command::Parse { file }
            }
            "compile" => {
                let target_str = args
                    .next()
                    .ok_or("Missing compile target (bytecode/wasm)")?;
                let target = match target_str.as_str() {
                    "bytecode" | "bc" => CompileTarget::Bytecode,
                    "wasm" | "wat" => CompileTarget::Wasm,
                    _ => return Err(format!("Unknown compile target: {}", target_str)),
                };
                let file = args.next().map(PathBuf::from);
                Command::Compile { file, target }
            }
            "version" | "-v" | "--version" => Command::Version,
            "help" | "-h" | "--help" => Command::Help,
            _ => return Err(format!("Unknown command: {}", cmd)),
        },
        None => Command::Repl,
    };

    Ok(Args { command })
}

fn read_source(file: Option<PathBuf>) -> Result<String, String> {
    match file {
        Some(path) => {
            fs::read_to_string(&path).map_err(|e| format!("Failed to read file {:?}: {}", path, e))
        }
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| format!("Failed to read from stdin: {}", e))?;
            Ok(buffer)
        }
    }
}

fn run_repl() {
    let mut repl = Repl::new();

    println!("Rustphy {} REPL", VERSION);
    println!("Type 'exit' or press Ctrl+D to quit\n");

    loop {
        // Print prompt
        print!("> ");
        io::stdout().flush().unwrap();

        // Read line from stdin
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\nBye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();

                // Check for exit command
                if input == "exit" || input == "quit" {
                    println!("Bye!");
                    break;
                }

                // Skip empty lines
                if input.is_empty() {
                    continue;
                }

                // Evaluate the input
                match repl.eval_line(input) {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn print_help() {
    println!("Rustphy {}", VERSION);
    println!();
    println!("USAGE:");
    println!("    rustphy <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    repl                    Start interactive REPL (default if no command given)");
    println!();
    println!("    run [FILE]              Run a Rustphy program");
    println!("                            If no file is specified, reads from stdin");
    println!();
    println!("    parse [FILE]            Parse source code and print the AST as JSON");
    println!("                            If no file is specified, reads from stdin");
    println!();
    println!("    compile <TARGET> [FILE] Compile to the specified target");
    println!("                            TARGET: bytecode|bc|wasm|wat");
    println!("                            If no file is specified, reads from stdin");
    println!();
    println!("    version                 Print version information");
    println!("    help                    Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    rustphy");
    println!("    rustphy repl");
    println!("    rustphy run script.gph");
    println!("    echo 'let x = 5; print(x);' | rustphy run");
    println!("    rustphy parse script.gph");
    println!("    rustphy compile wasm script.gph");
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_help();
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Command::Repl => {
            run_repl();
            return;
        }
        Command::Help => {
            print_help();
            return;
        }
        Command::Version => {
            println!("Rustphy {}", VERSION);
            return;
        }
        Command::Run { file } => {
            let source = match read_source(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            match run(&source) {
                Ok(result) => {
                    println!("{}", result);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Command::Parse { file } => {
            let source = match read_source(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            match parse(&source) {
                Ok(ast) => match serde_json::to_string_pretty(&ast) {
                    Ok(json) => {
                        println!("{}", json);
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to serialize AST to JSON: {}", e)),
                },
                Err(e) => Err(e),
            }
        }
        Command::Compile { file, target } => {
            let source = match read_source(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            let ast = match parse(&source) {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            };

            match target {
                CompileTarget::Bytecode => match compile_bytecode(&ast) {
                    Ok(instructions) => {
                        for (i, instr) in instructions.iter().enumerate() {
                            println!("{:04} {:?}", i, instr);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                CompileTarget::Wasm => match compile_wasm(&ast) {
                    Ok(wat) => {
                        println!("{}", wat);
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
