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
        input: Option<String>,
        output: Option<PathBuf>,
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

                // Parse remaining arguments
                let mut file: Option<PathBuf> = None;
                let mut input: Option<String> = None;
                let mut output: Option<PathBuf> = None;

                while let Some(arg) = args.next() {
                    match arg.as_str() {
                        "--file" | "-f" => {
                            file = Some(PathBuf::from(args.next().ok_or("Missing file path after --file")?));
                        }
                        "--input" | "-i" => {
                            input = Some(args.next().ok_or("Missing input code after --input")?.to_string());
                        }
                        "--output" | "-o" => {
                            output = Some(PathBuf::from(args.next().ok_or("Missing output path after --output")?));
                        }
                        _ => {
                            // If no flags, assume it's a file path
                            if file.is_none() && input.is_none() {
                                file = Some(PathBuf::from(arg));
                            } else {
                                return Err(format!("Unexpected argument: {}", arg));
                            }
                        }
                    }
                }

                Command::Compile { file, target, input, output }
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
    println!("    compile <TARGET> [OPTIONS]");
    println!("                            Compile to the specified target");
    println!("                            TARGET: bytecode|bc|wasm|wat");
    println!();
    println!("        OPTIONS:");
    println!("            --file, -f <FILE>      Input file");
    println!("            --input, -i <CODE>     Input code as string");
    println!("            --output, -o <FILE>    Output file (default: output.wat for WASM)");
    println!("            [FILE]                 Input file (positional, same as --file)");
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
    println!();
    println!("    # Compile to WASM (outputs to output.wat by default)");
    println!("    rustphy compile wasm --file script.gph");
    println!("    rustphy compile wasm script.gph");
    println!();
    println!("    # Compile with custom output file");
    println!("    rustphy compile wasm --file script.gph --output my_program.wat");
    println!("    rustphy compile wasm -f script.gph -o my_program.wat");
    println!();
    println!("    # Compile from inline code");
    println!("    rustphy compile wasm --input 'let x = 10; log(x);' -o test.wat");
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
        Command::Compile { file, target, input, output } => {
            // Get source code: either from --input, --file, or stdin
            let source = if let Some(code) = input {
                code
            } else {
                match read_source(file) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
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
                CompileTarget::Bytecode => {
                    match compile_bytecode(&ast) {
                        Ok(instructions) => {
                            let output_str = instructions
                                .iter()
                                .enumerate()
                                .map(|(i, instr)| format!("{:04} {:?}", i, instr))
                                .collect::<Vec<_>>()
                                .join("\n");

                            // Write to output file or stdout
                            if let Some(out_path) = output {
                                match fs::write(&out_path, &output_str) {
                                    Ok(_) => {
                                        println!("Bytecode written to {:?}", out_path);
                                        Ok(())
                                    }
                                    Err(e) => Err(format!("Failed to write output: {}", e)),
                                }
                            } else {
                                println!("{}", output_str);
                                Ok(())
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                CompileTarget::Wasm => match compile_wasm(&ast) {
                    Ok(wat) => {
                        // Default output file is output.wat for WASM
                        let default_output = PathBuf::from("output.wat");
                        let out_path = output.as_ref().unwrap_or(&default_output);

                        match fs::write(&out_path, &wat) {
                            Ok(_) => {
                                println!("WAT output written to {:?}", out_path);
                                Ok(())
                            }
                            Err(e) => Err(format!("Failed to write output: {}", e)),
                        }
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
