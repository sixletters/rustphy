//! Stack-based virtual machine for executing bytecode instructions.
//!
//! This module implements a simple VM that executes a sequence of bytecode instructions.
//! The VM uses two stacks:
//! - **Operand Stack (OS)**: Holds intermediate values during computation
//! - **Runtime Stack (RTS)**: Manages stack frames for function calls and scope management
//!
//! # Architecture
//!
//! The VM follows a stack-based execution model where:
//! - Instructions manipulate values on the operand stack
//! - Function calls create new stack frames with saved environments
//! - Scoping is handled through environment chains (lexical scoping)
//! - Closures capture their defining environment
//!
//! # Example
//!
//! ```
//! use rust_impl::machine::Machine;
//! use rust_impl::instruction::Instruction;
//!
//! let mut vm = Machine::new();
//! let instructions = vec![
//!     Instruction::LDCN { val: 42 },
//!     Instruction::DONE,
//! ];
//! let result = vm.run(&instructions).unwrap();
//! ```

use serde_json::to_string;

use crate::{
    environment::{BuiltinFn, Environment, Value},
    instruction::{BINOPS, Instruction, UNOPS},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Represents a stack frame in the runtime stack.
///
/// Stack frames are created for:
/// - Function calls (with `is_call_frame = true`)
/// - Block scopes (with `is_call_frame = false`)
#[derive(Debug)]
pub struct StackFrame {
    /// Program counter to return to when this frame is popped.
    /// Only meaningful for call frames.
    pc: usize,

    /// Saved environment to restore when this frame is popped.
    env: Rc<RefCell<Environment>>,

    /// True if this frame represents a function call, false for block scopes.
    is_call_frame: bool,
}

/// The virtual machine that executes bytecode instructions.
///
/// The machine maintains execution state including:
/// - An operand stack for intermediate values
/// - A runtime stack for managing function calls and scopes
/// - A program counter tracking the current instruction
/// - The current environment for variable lookups
pub struct Machine {
    /// Operand stack - holds intermediate computation values.
    os: Vec<Value>,

    /// Runtime stack - holds stack frames for function calls and scopes.
    rts: Vec<StackFrame>,

    /// Program counter - index of the current instruction being executed.
    pc: usize,

    /// Current execution environment for variable bindings.
    env: Rc<RefCell<Environment>>,

    /// Execution completion flag.
    is_done: bool,
}

impl Machine {
    /// Creates a new virtual machine with an empty global environment.
    ///
    /// Initializes:
    /// - Empty operand stack
    /// - Empty runtime stack
    /// - Program counter at 0
    /// - Fresh global environment
    /// - Execution flag set to not done
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_impl::machine::Machine;
    /// let vm = Machine::new();
    /// ```
    pub fn new() -> Self {
        let global_env = Environment::new();

        Machine {
            os: Vec::new(),
            rts: Vec::new(),
            pc: 0,
            env: global_env,
            is_done: false,
        }
    }

    /// Determines if a value is truthy in a boolean context.
    ///
    /// Truthiness rules:
    /// - `Bool(true)` is truthy, `Bool(false)` is falsy
    /// - `Number(0)` is falsy, all other numbers are truthy
    /// - `Float(0.0)` is falsy, all other floats are truthy
    /// - Empty strings are falsy, non-empty strings are truthy
    /// - Other types (Closure, Builtin, etc.) cannot be used in boolean contexts
    ///
    /// # Arguments
    ///
    /// * `value` - The value to check for truthiness
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - The truthiness of the value
    /// * `Err(String)` - If the value type cannot be used in boolean context
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_impl::machine::Machine;
    /// use rust_impl::environment::Value;
    ///
    /// assert_eq!(Machine::is_truthy(&Value::Bool(true)), Ok(true));
    /// assert_eq!(Machine::is_truthy(&Value::Number(0)), Ok(false));
    /// assert_eq!(Machine::is_truthy(&Value::Number(42)), Ok(true));
    /// ```
    pub fn is_truthy(value: &Value) -> Result<bool, String> {
        match value {
            Value::Bool(val) => Ok(*val),
            Value::Number(val) => Ok(*val != 0),
            Value::Float(val) => Ok(*val != 0.0),
            Value::String(val) => Ok(!val.is_empty()),
            _ => Err(format!(
                "Type error: value {:?} cannot be used in boolean context",
                value
            )),
        }
    }

    /// Executes a sequence of instructions until completion.
    ///
    /// Runs the instruction at the current program counter, updating the PC
    /// after each instruction, until a `DONE` instruction is encountered.
    ///
    /// # Arguments
    ///
    /// * `instructions` - The bytecode program to execute
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The value on top of the operand stack when execution completes
    /// * `Err(String)` - Runtime errors including:
    ///   - Type mismatches in operations
    ///   - Undefined variables
    ///   - Stack underflow
    ///   - Division by zero
    ///   - Empty operand stack at completion
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_impl::machine::Machine;
    /// use rust_impl::instruction::Instruction;
    /// use rust_impl::environment::Value;
    ///
    /// let mut vm = Machine::new();
    /// let instructions = vec![
    ///     Instruction::LDCN { val: 10 },
    ///     Instruction::LDCN { val: 32 },
    ///     Instruction::BINOP { ops: rust_impl::instruction::BINOPS::Add },
    ///     Instruction::DONE,
    /// ];
    /// let result = vm.run(&instructions).unwrap();
    /// assert!(matches!(result, Value::Number(42)));
    /// ```
    pub fn run(&mut self, instructions: &Vec<Instruction>) -> Result<Value, String> {
        while !self.is_done {
            self.execute(&instructions[self.pc])?;
        }

        match self.os.last() {
            Some(val) => Ok(val.clone()),
            None => Err("Runtime error: operand stack is empty, no value to return".to_string()),
        }
    }

    /// Executes a binary operation by popping two operands from the stack.
    ///
    /// Pops the right operand, then the left operand, applies the operation,
    /// and pushes the result back onto the operand stack.
    ///
    /// # Arguments
    ///
    /// * `op` - The binary operation to perform
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Operation completed successfully
    /// * `Err(String)` - Runtime errors including:
    ///   - Stack underflow (fewer than 2 operands)
    ///   - Type mismatch for the operation
    ///   - Division or modulo by zero
    pub fn execute_binop(&mut self, op: &BINOPS) -> Result<(), String> {
        let right = self.os.pop().ok_or_else(|| {
            format!(
                "Runtime error at PC {}: stack underflow, expected right operand for binary operation {:?}",
                self.pc, op
            )
        })?;
        let left = self.os.pop().ok_or_else(|| {
            format!(
                "Runtime error at PC {}: stack underflow, expected left operand for binary operation {:?}",
                self.pc, op
            )
        })?;

        let result = match op {
            BINOPS::Add => add_values(left, right)?,
            BINOPS::Minus => sub_values(left, right)?,
            BINOPS::Multiply => mul_values(left, right)?,
            BINOPS::Divide => div_values(left, right)?,
            BINOPS::Modulo => mod_values(left, right)?,
            BINOPS::Lt => compare_values(left, right, |a, b| a < b)?,
            BINOPS::Le => compare_values(left, right, |a, b| a <= b)?,
            BINOPS::Gt => compare_values(left, right, |a, b| a > b)?,
            BINOPS::Ge => compare_values(left, right, |a, b| a >= b)?,
            BINOPS::Eq => eq_values(left, right),
            BINOPS::Neq => match eq_values(left, right) {
                Value::Bool(b) => Value::Bool(!b),
                _ => {
                    return Err(format!(
                        "Runtime error at PC {}: equality comparison did not return boolean",
                        self.pc
                    ));
                }
            },
            BINOPS::And => {
                let left_bool = Machine::is_truthy(&left)
                    .map_err(|e| format!("Runtime error at PC {}: {}", self.pc, e))?;
                let right_bool = Machine::is_truthy(&right)
                    .map_err(|e| format!("Runtime error at PC {}: {}", self.pc, e))?;
                Value::Bool(left_bool && right_bool)
            }
            BINOPS::Or => {
                let left_bool = Machine::is_truthy(&left)
                    .map_err(|e| format!("Runtime error at PC {}: {}", self.pc, e))?;
                let right_bool = Machine::is_truthy(&right)
                    .map_err(|e| format!("Runtime error at PC {}: {}", self.pc, e))?;
                Value::Bool(left_bool || right_bool)
            }
            BINOPS::Assign => {
                match left {
                    Value::Identifier(val) => {
                        self.env
                            .borrow_mut()
                            .set_assign(val.clone().as_str(), right.clone())
                            .map_err(|e| format!("Runtime error at PC {}: {}", self.pc, e))?;
                    }
                    _ => {
                        return Err(format!(
                            "Runtime error at PC {}: assignment target must be an identifier",
                            self.pc
                        ));
                    }
                }
                right
            }
        };

        self.os.push(result);
        Ok(())
    }

    /// Executes a unary operation by popping one operand from the stack.
    ///
    /// Pops the operand, applies the operation, and pushes the result back
    /// onto the operand stack.
    ///
    /// # Arguments
    ///
    /// * `op` - The unary operation to perform
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Operation completed successfully
    /// * `Err(String)` - Runtime errors including:
    ///   - Stack underflow (empty operand stack)
    ///   - Type mismatch (e.g., negating a non-numeric value)
    pub fn execute_unop(&mut self, op: &UNOPS) -> Result<(), String> {
        let operand = self.os.pop().ok_or_else(|| {
            format!(
                "Runtime error at PC {}: stack underflow, expected operand for unary operation {:?}",
                self.pc, op
            )
        })?;

        let result = match op {
            UNOPS::Negative => match operand {
                Value::Number(n) => Value::Number(-n),
                Value::Float(f) => Value::Float(-f),
                _ => {
                    return Err(format!(
                        "Runtime error at PC {}: type error, cannot negate value of type {:?}",
                        self.pc, operand
                    ));
                }
            },
            UNOPS::Not => match Machine::is_truthy(&operand) {
                Ok(val) => Value::Bool(!val),
                Err(err) => return Err(format!("Runtime error at PC {}: {}", self.pc, err)),
            },
        };
        self.os.push(result);
        Ok(())
    }

    /// Executes a built-in function with the given arguments.
    ///
    /// Built-in functions are native Rust functions that are pre-populated
    /// in the global environment and can be called from the VM.
    ///
    /// # Arguments
    ///
    /// * `builtin_fn` - The built-in function to execute
    /// * `args` - The arguments passed to the function
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The result of the built-in function
    /// * `Err(String)` - If the function fails (e.g., wrong number of arguments)
    pub fn execute_builtin(
        &mut self,
        builtin_fn: BuiltinFn,
        args: &Vec<Value>,
    ) -> Result<Value, String> {
        match builtin_fn {
            BuiltinFn::Print => {
                if args.len() != 1 {
                    return Err(format!(
                        "Runtime error at PC {}: print expects 1 argument, got {}",
                        self.pc,
                        args.len()
                    ));
                }
                println!("{:?}", args[0]);
                // print returns the value that was printed
                Ok(args[0].clone())
            }
            BuiltinFn::PushArr => {
                if args.len() != 2 {
                    return Err(format!(
                        "Runtime error at PC {}: pushArr expects 2 arguments, got {}",
                        self.pc,
                        args.len()
                    ));
                }
                match &args[0] {
                    Value::Array(arr) => {
                        arr.borrow_mut().push(args[1].clone());
                        Ok(Value::Array(arr.clone()))
                    }
                    _ => Err(format!(
                        "Runtime error at PC {}: pushArr expects an array as first argument",
                        self.pc
                    )),
                }
            }
            BuiltinFn::Len => {
                if args.len() != 1 {
                    return Err(format!(
                        "Runtime error at PC {}: len expects 1 argument, got {}",
                        self.pc,
                        args.len()
                    ));
                }
                match &args[0] {
                    Value::Array(arr) => Ok(Value::Number(arr.borrow().len() as i128)),
                    Value::HashMap(map) => Ok(Value::Number(map.borrow().len() as i128)),
                    Value::String(s) => Ok(Value::Number(s.len() as i128)),
                    _ => Err(format!(
                        "Runtime error at PC {}: len expects an array, hashmap, or string, got {:?}",
                        self.pc, args[0]
                    )),
                }
            }
        }
    }

    /// Executes a single bytecode instruction.
    ///
    /// This is the core instruction dispatch method that implements the VM's
    /// instruction set. Most instructions update the program counter (PC) by 1,
    /// except for control flow instructions (JOF, GOTO, CALL) which may jump
    /// to different locations.
    ///
    /// # Arguments
    ///
    /// * `instr` - The instruction to execute
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Instruction executed successfully
    /// * `Err(String)` - Runtime errors including:
    ///   - Stack underflow
    ///   - Type mismatches
    ///   - Undefined variables
    ///   - Invalid operations
    pub fn execute(&mut self, instr: &Instruction) -> Result<(), String> {
        match instr {
            Instruction::LDCB { val } => {
                self.os.push(Value::Bool(*val));
                self.pc += 1;
                Ok(())
            }
            Instruction::LDCN { val } => {
                self.os.push(Value::Number(*val));
                self.pc += 1;
                Ok(())
            }
            Instruction::LDI { val } => {
                self.os.push(Value::Identifier(val.clone()));
                self.pc += 1;
                Ok(())
            }
            Instruction::LDSL { val } => {
                self.os.push(Value::String(val.clone()));
                self.pc += 1;
                Ok(())
            }
            Instruction::LDS { sym } => {
                let value = match self.get_builtin(sym) {
                    Some(val) => Value::Builtin { name: val },
                    None => self.env.borrow().get(sym).ok_or_else(|| {
                        format!(
                            "Runtime error at PC {}: undefined variable '{}'",
                            self.pc, sym
                        )
                    })?,
                };
                self.os.push(value);
                self.pc += 1;
                Ok(())
            }
            Instruction::LDAI => {
                // Pop index
                let index = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected index",
                        self.pc
                    )
                })?;

                // Pop container (array or hash map)
                let container = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected array or hash map",
                        self.pc
                    )
                })?;

                // Read element from container
                let element = match container {
                    Value::Array(ref arr) => {
                        // Array requires numeric index
                        let idx = match index {
                            Value::Number(n) => n as usize,
                            _ => {
                                return Err(format!(
                                    "Runtime error at PC {}: array index must be a number, got {:?}",
                                    self.pc, index
                                ));
                            }
                        };

                        if idx >= arr.borrow().len() {
                            return Err(format!(
                                "Runtime error at PC {}: array index out of bounds: index {} >= length {}",
                                self.pc,
                                idx,
                                arr.borrow().len()
                            ));
                        }

                        arr.borrow()[idx].clone()
                    }
                    Value::HashMap(ref map) => {
                        // Hash map accepts string or number keys (numbers are converted to strings)
                        let key = match index {
                            Value::String(s) => s,
                            Value::Number(n) => n.to_string(),
                            _ => {
                                return Err(format!(
                                    "Runtime error at PC {}: hash map key must be a string or number, got {:?}",
                                    self.pc, index
                                ));
                            }
                        };

                        map.borrow().get(&key).cloned().ok_or_else(|| {
                            format!(
                                "Runtime error at PC {}: hash map key '{}' not found",
                                self.pc, key
                            )
                        })?
                    }
                    _ => {
                        return Err(format!(
                            "Runtime error at PC {}: cannot index into {:?}, expected array or hash map",
                            self.pc, container
                        ));
                    }
                };

                self.os.push(element);
                self.pc += 1;
                Ok(())
            }
            Instruction::STAI => {
                // Pop value to store
                let value = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected value to store",
                        self.pc
                    )
                })?;

                // Pop index/key
                let index = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected index",
                        self.pc
                    )
                })?;

                // Pop container (array or hash map)
                let container = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected array or hash map",
                        self.pc
                    )
                })?;

                // Write element to container
                match container {
                    Value::Array(ref arr) => {
                        // Array requires numeric index
                        let idx = match index {
                            Value::Number(n) => n as usize,
                            _ => {
                                return Err(format!(
                                    "Runtime error at PC {}: array index must be a number, got {:?}",
                                    self.pc, index
                                ));
                            }
                        };

                        if idx >= arr.borrow().len() {
                            return Err(format!(
                                "Runtime error at PC {}: array index out of bounds: index {} >= length {}",
                                self.pc,
                                idx,
                                arr.borrow().len()
                            ));
                        }

                        arr.borrow_mut()[idx] = value.clone();
                    }
                    Value::HashMap(ref map) => {
                        // Hash map accepts string or number keys (numbers are converted to strings)
                        let key = match index {
                            Value::String(s) => s,
                            Value::Number(n) => n.to_string(),
                            _ => {
                                return Err(format!(
                                    "Runtime error at PC {}: hash map key must be a string or number, got {:?}",
                                    self.pc, index
                                ));
                            }
                        };

                        map.borrow_mut().insert(key, value.clone());
                    }
                    _ => {
                        return Err(format!(
                            "Runtime error at PC {}: cannot index into {:?}, expected array or hash map",
                            self.pc, container
                        ));
                    }
                }

                // STAI pushes the value back onto the stack (for use in expressions)
                self.os.push(value);
                self.pc += 1;
                Ok(())
            }
            Instruction::LDF { addr, params } => {
                let closure = Value::Closure {
                    params: params.clone(),
                    addr: addr.clone(),
                    env: self.env.clone(),
                };
                self.os.push(closure);
                self.pc += 1;
                Ok(())
            }
            Instruction::CALL { arity } => {
                let mut args = vec![Value::Unassigned; *arity];
                for i in (0..*arity).rev() {
                    args[i] = self.os.pop().ok_or_else(|| {
                        format!(
                            "Runtime error at PC {}: stack underflow, expected {} arguments but found fewer",
                            self.pc, arity
                        )
                    })?;
                }
                let function_obj = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected closure to call",
                        self.pc
                    )
                })?;

                match function_obj {
                    Value::Closure { params, addr, env } => {
                        self.rts.push(StackFrame {
                            pc: self.pc + 1,
                            env: self.env.clone(),
                            is_call_frame: true,
                        });
                        self.env = Environment::extend(env.clone());
                        for (i, val) in params.iter().enumerate() {
                            self.env
                                .borrow_mut()
                                .set_declare(val.clone(), args[i].clone());
                        }
                        self.pc = addr;
                        Ok(())
                    }
                    Value::Builtin { name } => {
                        let result = self.execute_builtin(name, &args)?;
                        self.os.push(result);
                        self.pc += 1;
                        Ok(())
                    }
                    _ => Err(format!(
                        "Runtime error at PC {}: type error, expected closure or builtin but got {:?}",
                        self.pc, function_obj
                    )),
                }
            }
            Instruction::POP => {
                self.os.pop();
                self.pc += 1;
                Ok(())
            }
            Instruction::MKHASH { size } => {
                let map = Rc::new(RefCell::new(HashMap::new()));

                // Pop key-value pairs in reverse order (stack pops from top)
                // Each iteration pops: value then key
                for i in 0..*size {
                    // Pop value first
                    let value = self.os.pop().ok_or_else(|| {
                        format!(
                            "Runtime error at PC {}: stack underflow while creating hash map, expected {} pairs ({} values) but only found {}",
                            self.pc, size, size * 2, i * 2
                        )
                    })?;

                    // Pop key second
                    let key_value = self.os.pop().ok_or_else(|| {
                        format!(
                            "Runtime error at PC {}: stack underflow while creating hash map, expected {} pairs ({} keys) but only found {}",
                            self.pc, size, size * 2, i * 2 + 1
                        )
                    })?;

                    // Convert key to string (supports string and number keys)
                    let key = match key_value {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        _ => {
                            return Err(format!(
                                "Runtime error at PC {}: hash map key must be a string or number, got {:?}",
                                self.pc, key_value
                            ));
                        }
                    };

                    map.borrow_mut().insert(key, value);
                }

                self.os.push(Value::HashMap(map));
                self.pc += 1;
                Ok(())
            }
            Instruction::ASSIGN { sym } => {
                let value = self.os.last().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, no value to assign to '{}'",
                        self.pc, sym
                    )
                })?;
                self.env
                    .borrow_mut()
                    .set_declare(sym.clone(), value.clone());
                self.pc += 1;
                Ok(())
            }
            Instruction::ENTERSCOPE { syms } => {
                self.rts.push(StackFrame {
                    pc: 0,
                    env: self.env.clone(),
                    is_call_frame: false,
                });
                self.env = Environment::extend(self.env.clone());
                for ide in syms.iter() {
                    self.env
                        .borrow_mut()
                        .set_declare(ide.clone(), Value::Unassigned);
                }
                self.pc += 1;
                Ok(())
            }
            Instruction::MKARR { size } => {
                // Make array from top N stack values.
                //
                // Pops `size` values from the operand stack, creates an array
                // containing those values in the correct order, and pushes the
                // array back onto the stack.
                //
                // Note: Values are popped in reverse order (LIFO), so we need to
                // reverse them to maintain the original array order.
                //
                // Example execution:
                //   Stack before: [v1, v2, v3]  (v3 is top)
                //   MKARR 3
                //   Pop: v3, v2, v1 (reverse order)
                //   Reverse: v1, v2, v3
                //   Stack after: [[v1, v2, v3]]
                let arr = Rc::new(RefCell::new(vec![]));
                for i in 0..*size {
                    match self.os.pop() {
                        Some(val) => {
                            arr.borrow_mut().push(val);
                        }
                        None => {
                            return Err(format!(
                                "Runtime error at PC {}: stack underflow while creating array, expected {} elements but only found {}",
                                self.pc, size, i
                            ));
                        }
                    }
                }
                // Reverse to restore original order (compensate for LIFO pop)
                arr.borrow_mut().reverse();
                self.os.push(Value::Array(arr));
                self.pc += 1;
                Ok(())
            }
            Instruction::EXITSCOPE => {
                let sf = self.rts.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: cannot exit scope, runtime stack is empty",
                        self.pc
                    )
                })?;
                self.env = sf.env;
                self.pc += 1;
                Ok(())
            }
            Instruction::JOF { addr } => {
                let cond = self.os.pop().ok_or_else(|| {
                    format!(
                        "Runtime error at PC {}: stack underflow, expected condition for JOF",
                        self.pc
                    )
                })?;
                let truthy = Machine::is_truthy(&cond)
                    .map_err(|err| format!("Runtime error at PC {}: {}", self.pc, err))?;
                if truthy {
                    self.pc += 1
                } else {
                    self.pc = *addr
                }
                Ok(())
            }
            Instruction::GOTO { addr } => {
                self.pc = *addr;
                Ok(())
            }
            Instruction::RESET => {
                loop {
                    let top_frame = self.rts.pop().ok_or_else(|| {
                        format!(
                            "Runtime error at PC {}: cannot reset, runtime stack is empty",
                            self.pc
                        )
                    })?;
                    if top_frame.is_call_frame {
                        self.pc = top_frame.pc;
                        self.env = top_frame.env;
                        break;
                    }
                }
                Ok(())
            }
            Instruction::BINOP { ops } => {
                self.execute_binop(ops)?;
                self.pc += 1;
                Ok(())
            }
            Instruction::UNOP { ops } => {
                self.execute_unop(ops)?;
                self.pc += 1;
                Ok(())
            }
            Instruction::DONE => {
                self.is_done = true;
                Ok(())
            }
            _ => Err(format!(
                "Runtime error at PC {}: unimplemented instruction {:?}",
                self.pc, instr
            )),
        }
    }

    fn get_builtin(&self, name: &str) -> Option<BuiltinFn> {
        match name {
            "print" => return Some(BuiltinFn::Print),
            "push_arr" => return Some(BuiltinFn::PushArr),
            "len" => return Some(BuiltinFn::Len),
            _ => None,
        }
    }
}

// ============================================================================
// Helper functions for binary operations
// ============================================================================

/// Adds two values together.
///
/// Supports:
/// - Number + Number -> Number
/// - Float + Float -> Float
/// - Number + Float -> Float (with promotion)
/// - Float + Number -> Float (with promotion)
/// - String + String -> String (concatenation)
///
/// # Returns
///
/// * `Ok(Value)` - The result of the addition
/// * `Err(String)` - If the types cannot be added
fn add_values(left: Value, right: Value) -> Result<Value, String> {
    match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
        (Value::Number(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
        (Value::Float(l), Value::Number(r)) => Ok(Value::Float(l + *r as f64)),
        (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
        _ => Err(format!("Type error: cannot add {:?} and {:?}", left, right)),
    }
}

/// Subtracts the right value from the left value.
///
/// Supports numeric types with automatic type promotion to Float when needed.
///
/// # Returns
///
/// * `Ok(Value)` - The result of the subtraction
/// * `Err(String)` - If the types cannot be subtracted
fn sub_values(left: Value, right: Value) -> Result<Value, String> {
    match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
        (Value::Number(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
        (Value::Float(l), Value::Number(r)) => Ok(Value::Float(l - *r as f64)),
        _ => Err(format!(
            "Type error: cannot subtract {:?} from {:?}",
            right, left
        )),
    }
}

/// Multiplies two values.
///
/// Supports numeric types with automatic type promotion to Float when needed.
///
/// # Returns
///
/// * `Ok(Value)` - The result of the multiplication
/// * `Err(String)` - If the types cannot be multiplied
fn mul_values(left: Value, right: Value) -> Result<Value, String> {
    match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
        (Value::Number(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
        (Value::Float(l), Value::Number(r)) => Ok(Value::Float(l * *r as f64)),
        _ => Err(format!(
            "Type error: cannot multiply {:?} and {:?}",
            left, right
        )),
    }
}

/// Divides the left value by the right value.
///
/// Supports numeric types with automatic type promotion to Float when needed.
///
/// # Returns
///
/// * `Ok(Value)` - The result of the division
/// * `Err(String)` - If division by zero or types cannot be divided
fn div_values(left: Value, right: Value) -> Result<Value, String> {
    match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => {
            if *r == 0 {
                return Err("Runtime error: division by zero".to_string());
            }
            Ok(Value::Number(l / r))
        }
        (Value::Float(l), Value::Float(r)) => {
            if *r == 0.0 {
                return Err("Runtime error: division by zero".to_string());
            }
            Ok(Value::Float(l / r))
        }
        (Value::Number(l), Value::Float(r)) => {
            if *r == 0.0 {
                return Err("Runtime error: division by zero".to_string());
            }
            Ok(Value::Float(*l as f64 / r))
        }
        (Value::Float(l), Value::Number(r)) => {
            if *r == 0 {
                return Err("Runtime error: division by zero".to_string());
            }
            Ok(Value::Float(l / *r as f64))
        }
        _ => Err(format!(
            "Type error: cannot divide {:?} by {:?}",
            left, right
        )),
    }
}

/// Computes the modulo of left value by right value.
///
/// Supports numeric types with automatic type promotion to Float when needed.
///
/// # Returns
///
/// * `Ok(Value)` - The result of the modulo operation
/// * `Err(String)` - If modulo by zero or types cannot perform modulo
fn mod_values(left: Value, right: Value) -> Result<Value, String> {
    match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => {
            if *r == 0 {
                return Err("Runtime error: modulo by zero".to_string());
            }
            Ok(Value::Number(l % r))
        }
        (Value::Float(l), Value::Float(r)) => {
            if *r == 0.0 {
                return Err("Runtime error: modulo by zero".to_string());
            }
            Ok(Value::Float(l % r))
        }
        (Value::Number(l), Value::Float(r)) => {
            if *r == 0.0 {
                return Err("Runtime error: modulo by zero".to_string());
            }
            Ok(Value::Float((*l as f64) % r))
        }
        (Value::Float(l), Value::Number(r)) => {
            if *r == 0 {
                return Err("Runtime error: modulo by zero".to_string());
            }
            Ok(Value::Float(l % (*r as f64)))
        }
        _ => Err(format!(
            "Type error: cannot perform modulo on {:?} and {:?}",
            left, right
        )),
    }
}

/// Compares two numeric values using the provided comparison function.
///
/// Converts both values to f64 for comparison.
///
/// # Arguments
///
/// * `left` - The left operand
/// * `right` - The right operand
/// * `op` - The comparison function (e.g., <, <=, >, >=)
///
/// # Returns
///
/// * `Ok(Value::Bool)` - The comparison result
/// * `Err(String)` - If the values are not numeric types
fn compare_values<F>(left: Value, right: Value, op: F) -> Result<Value, String>
where
    F: Fn(f64, f64) -> bool,
{
    let result = match (&left, &right) {
        (Value::Number(l), Value::Number(r)) => op(*l as f64, *r as f64),
        (Value::Float(l), Value::Float(r)) => op(*l, *r),
        (Value::Number(l), Value::Float(r)) => op(*l as f64, *r),
        (Value::Float(l), Value::Number(r)) => op(*l, *r as f64),
        _ => {
            return Err(format!(
                "Type error: cannot compare {:?} and {:?}",
                left, right
            ));
        }
    };
    Ok(Value::Bool(result))
}

/// Tests two values for equality.
///
/// Supports:
/// - Number == Number
/// - Float == Float
/// - Number == Float (with promotion)
/// - Float == Number (with promotion)
/// - Bool == Bool
/// - String == String
/// - All other comparisons return false
///
/// # Returnse
///
/// A `Value::Bool` indicating whether the values are equal.
fn eq_values(left: Value, right: Value) -> Value {
    let result = match (left, right) {
        (Value::Number(l), Value::Number(r)) => l == r,
        (Value::Float(l), Value::Float(r)) => l == r,
        (Value::Number(l), Value::Float(r)) => (l as f64) == r,
        (Value::Float(l), Value::Number(r)) => l == (r as f64),
        (Value::Bool(l), Value::Bool(r)) => l == r,
        (Value::String(l), Value::String(r)) => l == r,
        _ => false,
    };
    Value::Bool(result)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_machine() {
        let vm = Machine::new();
        assert_eq!(vm.os.len(), 0);
        assert_eq!(vm.rts.len(), 0);
        assert_eq!(vm.pc, 0);
        assert!(!vm.is_done);
    }

    #[test]
    fn test_is_truthy() {
        assert_eq!(Machine::is_truthy(&Value::Bool(true)), Ok(true));
        assert_eq!(Machine::is_truthy(&Value::Bool(false)), Ok(false));
        assert_eq!(Machine::is_truthy(&Value::Number(0)), Ok(false));
        assert_eq!(Machine::is_truthy(&Value::Number(42)), Ok(true));
        assert_eq!(Machine::is_truthy(&Value::Number(-1)), Ok(true));
        assert_eq!(Machine::is_truthy(&Value::Float(0.0)), Ok(false));
        assert_eq!(Machine::is_truthy(&Value::Float(3.14)), Ok(true));
        assert_eq!(
            Machine::is_truthy(&Value::String("".to_string())),
            Ok(false)
        );
        assert_eq!(
            Machine::is_truthy(&Value::String("hello".to_string())),
            Ok(true)
        );
        assert!(Machine::is_truthy(&Value::Unassigned).is_err());
    }

    #[test]
    fn test_load_constant_bool() {
        let mut vm = Machine::new();
        let instructions = vec![Instruction::LDCB { val: true }, Instruction::DONE];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_load_constant_number() {
        let mut vm = Machine::new();
        let instructions = vec![Instruction::LDCN { val: 42 }, Instruction::DONE];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_load_string_literal() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDSL {
                val: "hello world".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello world"));
    }

    #[test]
    fn test_load_string_literal_empty() {
        let mut vm = Machine::new();
        let instructions = vec![Instruction::LDSL { val: String::new() }, Instruction::DONE];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s.is_empty()));
    }

    #[test]
    fn test_load_string_literal_with_escapes() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDSL {
                val: "line1\nline2\ttab".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "line1\nline2\ttab"));
    }

    #[test]
    fn test_string_assignment() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::ENTERSCOPE { syms: vec![] },
            Instruction::LDSL {
                val: "Hello, World!".to_string(),
            },
            Instruction::ASSIGN {
                sym: "message".to_string(),
            },
            Instruction::POP, // Pop the assignment result
            Instruction::LDS {
                sym: "message".to_string(),
            },
            Instruction::EXITSCOPE,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "Hello, World!"));
    }

    #[test]
    fn test_multiple_strings() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDSL {
                val: "first".to_string(),
            },
            Instruction::POP,
            Instruction::LDSL {
                val: "second".to_string(),
            },
            Instruction::POP,
            Instruction::LDSL {
                val: "third".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "third"));
    }

    #[test]
    fn test_load_identifier() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Identifier(s) if s == "x"));
    }

    #[test]
    fn test_arithmetic_addition() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 32 },
            Instruction::BINOP { ops: BINOPS::Add },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_arithmetic_subtraction() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 50 },
            Instruction::LDCN { val: 8 },
            Instruction::BINOP { ops: BINOPS::Minus },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_arithmetic_multiplication() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 6 },
            Instruction::LDCN { val: 7 },
            Instruction::BINOP {
                ops: BINOPS::Multiply,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_arithmetic_division() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 84 },
            Instruction::LDCN { val: 2 },
            Instruction::BINOP {
                ops: BINOPS::Divide,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_arithmetic_modulo() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 3 },
            Instruction::BINOP {
                ops: BINOPS::Modulo,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(1)));
    }

    #[test]
    fn test_comparison_less_than() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 5 },
            Instruction::LDCN { val: 10 },
            Instruction::BINOP { ops: BINOPS::Lt },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_comparison_greater_than() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 5 },
            Instruction::BINOP { ops: BINOPS::Gt },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_equality() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::LDCN { val: 42 },
            Instruction::BINOP { ops: BINOPS::Eq },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_inequality() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::LDCN { val: 10 },
            Instruction::BINOP { ops: BINOPS::Neq },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_unary_negation() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::UNOP {
                ops: UNOPS::Negative,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(-42)));
    }

    #[test]
    fn test_unary_not() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCB { val: true },
            Instruction::UNOP { ops: UNOPS::Not },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_variable_assignment_and_load() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::LDS {
                sym: "x".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_pop_instruction() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::POP,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(1)));
    }

    #[test]
    fn test_jump_on_false_true_condition() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCB { val: true },
            Instruction::JOF { addr: 4 },
            Instruction::LDCN { val: 42 },
            Instruction::DONE,
            Instruction::LDCN { val: 99 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_jump_on_false_false_condition() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCB { val: false },
            Instruction::JOF { addr: 4 },
            Instruction::LDCN { val: 42 },
            Instruction::DONE,
            Instruction::LDCN { val: 99 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(99)));
    }

    #[test]
    fn test_goto() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::GOTO { addr: 3 },
            Instruction::LDCN { val: 1 },
            Instruction::DONE,
            Instruction::LDCN { val: 42 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_enter_and_exit_scope() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::ENTERSCOPE {
                syms: vec!["y".to_string()],
            },
            Instruction::LDCN { val: 20 },
            Instruction::ASSIGN {
                sym: "y".to_string(),
            },
            Instruction::LDS {
                sym: "y".to_string(),
            },
            Instruction::EXITSCOPE,
            Instruction::LDS {
                sym: "x".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(10)));
    }

    #[test]
    fn test_simple_function_call() {
        let mut vm = Machine::new();
        let instructions = vec![
            // 0: Skip over function definition
            Instruction::GOTO { addr: 5 },
            // 1: Function body - load parameter x and return
            Instruction::LDS {
                sym: "x".to_string(),
            },
            // 2: Return from function
            Instruction::RESET,
            // 3: Should not reach here
            Instruction::LDCN { val: 999 },
            Instruction::DONE,
            // 5: Main program - load function
            Instruction::LDF {
                addr: 1,
                params: vec!["x".to_string()],
            },
            // 6: Load argument
            Instruction::LDCN { val: 42 },
            // 7: Call function with arity 1
            Instruction::CALL { arity: 1 },
            // 8: Done
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_function_with_arithmetic() {
        let mut vm = Machine::new();
        let instructions = vec![
            // 0: Skip over function definition
            Instruction::GOTO { addr: 5 },
            // 1: Function body - x * 2
            Instruction::LDS {
                sym: "x".to_string(),
            },
            Instruction::LDCN { val: 2 },
            Instruction::BINOP {
                ops: BINOPS::Multiply,
            },
            Instruction::RESET,
            // 5: Main program - define function that doubles its argument
            Instruction::LDF {
                addr: 1,
                params: vec!["x".to_string()],
            },
            // 6: Load argument 21
            Instruction::LDCN { val: 21 },
            // 7: Call function
            Instruction::CALL { arity: 1 },
            // 8: Done
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_closure_captures_environment() {
        let mut vm = Machine::new();
        let instructions = vec![
            // 0: Set x = 10 in global scope
            Instruction::LDCN { val: 10 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            // 2: Skip over function definition
            Instruction::GOTO { addr: 7 },
            // 3: Function body: x + y (10 + 32)
            Instruction::LDS {
                sym: "x".to_string(),
            },
            Instruction::LDS {
                sym: "y".to_string(),
            },
            Instruction::BINOP { ops: BINOPS::Add },
            Instruction::RESET,
            // 7: Main program - create closure that uses x
            Instruction::LDF {
                addr: 3,
                params: vec!["y".to_string()],
            },
            // 8: Call with argument 32
            Instruction::LDCN { val: 32 },
            // 9: Call the function
            Instruction::CALL { arity: 1 },
            // 10: Done
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDS {
                sym: "undefined".to_string(),
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("undefined variable"));
    }

    #[test]
    fn test_division_by_zero() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::LDCN { val: 0 },
            Instruction::BINOP {
                ops: BINOPS::Divide,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("division by zero"));
    }

    #[test]
    fn test_stack_underflow() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::BINOP { ops: BINOPS::Add },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("stack underflow"));
    }

    #[test]
    fn test_string_concatenation() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::ASSIGN {
                sym: "hello".to_string(),
            },
            Instruction::LDCN { val: 43 },
            Instruction::ASSIGN {
                sym: "world".to_string(),
            },
            Instruction::DONE,
        ];
        vm.run(&instructions).unwrap();
    }

    #[test]
    fn test_complex_expression() {
        let mut vm = Machine::new();
        // (10 + 5) * 2 - 8 = 30 - 8 = 22
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 5 },
            Instruction::BINOP { ops: BINOPS::Add },
            Instruction::LDCN { val: 2 },
            Instruction::BINOP {
                ops: BINOPS::Multiply,
            },
            Instruction::LDCN { val: 8 },
            Instruction::BINOP { ops: BINOPS::Minus },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(22)));
    }

    #[test]
    fn test_assignment_as_expression() {
        let mut vm = Machine::new();
        // Declare x first, then use assignment expression: x = 42
        let instructions = vec![
            // Declare x
            Instruction::LDCN { val: 0 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::POP,
            // Use assignment as expression: x = 42
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));

        // Verify the variable was actually assigned
        let value = vm.env.borrow().get("x").unwrap();
        assert!(matches!(value, Value::Number(42)));
    }

    #[test]
    fn test_chained_assignment() {
        let mut vm = Machine::new();
        // Declare x and y first, then: y = (x = 42)
        let instructions = vec![
            // Declare x
            Instruction::LDCN { val: 0 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::POP,
            // Declare y
            Instruction::LDCN { val: 0 },
            Instruction::ASSIGN {
                sym: "y".to_string(),
            },
            Instruction::POP,
            // Chained assignment: y = (x = 42)
            Instruction::LDI {
                val: "y".to_string(),
            },
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));

        // Verify both variables were assigned
        let x_value = vm.env.borrow().get("x").unwrap();
        let y_value = vm.env.borrow().get("y").unwrap();
        assert!(matches!(x_value, Value::Number(42)));
        assert!(matches!(y_value, Value::Number(42)));
    }

    #[test]
    fn test_assignment_in_arithmetic_expression() {
        let mut vm = Machine::new();
        // Declare x first, then: (x = 10) + 32 = 42
        let instructions = vec![
            // Declare x
            Instruction::LDCN { val: 0 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::POP,
            // (x = 10) + 32
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 10 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::LDCN { val: 32 },
            Instruction::BINOP { ops: BINOPS::Add },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));

        // Verify x was assigned
        let x_value = vm.env.borrow().get("x").unwrap();
        assert!(matches!(x_value, Value::Number(10)));
    }

    #[test]
    fn test_assignment_reassignment() {
        let mut vm = Machine::new();
        // Declare x, then reassign x = 10, then reassign x = 20
        let instructions = vec![
            // Declare x
            Instruction::LDCN { val: 0 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::POP,
            // First reassignment: x = 10
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 10 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::POP,
            // Second reassignment: x = 20
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 20 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(20)));

        // Verify x was reassigned
        let x_value = vm.env.borrow().get("x").unwrap();
        assert!(matches!(x_value, Value::Number(20)));
    }

    #[test]
    fn test_assignment_with_different_types() {
        let mut vm = Machine::new();
        // Declare variables first, then test assigning different value types
        let instructions = vec![
            // Declare x
            Instruction::LDCB { val: false },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::POP,
            // Declare y
            Instruction::LDCN { val: 0 },
            Instruction::ASSIGN {
                sym: "y".to_string(),
            },
            Instruction::POP,
            // x = true
            Instruction::LDI {
                val: "x".to_string(),
            },
            Instruction::LDCB { val: true },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::POP,
            // y = 42
            Instruction::LDI {
                val: "y".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));

        // Verify both assignments worked
        let x_value = vm.env.borrow().get("x").unwrap();
        let y_value = vm.env.borrow().get("y").unwrap();
        assert!(matches!(x_value, Value::Bool(true)));
        assert!(matches!(y_value, Value::Number(42)));
    }

    #[test]
    fn test_assignment_to_undeclared_variable() {
        let mut vm = Machine::new();
        // Try to assign to undeclared variable - should fail
        let instructions = vec![
            Instruction::LDI {
                val: "undeclared".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::BINOP {
                ops: BINOPS::Assign,
            },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Variable 'undeclared' not found")
        );
    }

    #[test]
    fn test_builtin_print() {
        let mut vm = Machine::new();
        // Test calling print builtin: print(42)
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::CALL { arity: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        // print should return the value it printed
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_builtin_print_wrong_arity() {
        let mut vm = Machine::new();
        // Test calling print with wrong number of arguments
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::LDCN { val: 10 },
            Instruction::CALL { arity: 2 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("print expects 1 argument"));
    }

    #[test]
    fn test_builtin_print_no_args() {
        let mut vm = Machine::new();
        // Test calling print with no arguments
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::CALL { arity: 0 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("print expects 1 argument"));
    }

    #[test]
    fn test_builtin_print_string() {
        let mut vm = Machine::new();
        // Test calling print with a string
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDSL {
                val: "Hello, World!".to_string(),
            },
            Instruction::CALL { arity: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "Hello, World!"));
    }

    #[test]
    fn test_builtin_print_bool() {
        let mut vm = Machine::new();
        // Test calling print with a boolean
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDCB { val: true },
            Instruction::CALL { arity: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_builtin_in_expression() {
        let mut vm = Machine::new();
        // Test using print in an expression: print(10) + 32 = 42
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDCN { val: 10 },
            Instruction::CALL { arity: 1 },
            Instruction::LDCN { val: 32 },
            Instruction::BINOP { ops: BINOPS::Add },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_builtin_with_variable() {
        let mut vm = Machine::new();
        // Test calling print with a variable: x = 42; print(x)
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::ASSIGN {
                sym: "x".to_string(),
            },
            Instruction::POP,
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDS {
                sym: "x".to_string(),
            },
            Instruction::CALL { arity: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_builtin_nested_call() {
        let mut vm = Machine::new();
        // Test nested builtin calls: print(print(42))
        let instructions = vec![
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDS {
                sym: "print".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::CALL { arity: 1 },
            Instruction::CALL { arity: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_get_builtin() {
        let vm = Machine::new();
        assert_eq!(vm.get_builtin("print"), Some(BuiltinFn::Print));
        assert_eq!(vm.get_builtin("nonexistent"), None);
        assert_eq!(vm.get_builtin(""), None);
    }

    // ===== Array (MKARR) Tests =====

    #[test]
    fn test_mkarr_empty_array() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::MKARR { size: 0 }, // Create empty array
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.borrow().len(), 0, "Empty array should have length 0");
            }
            _ => panic!("Expected Array value, got {:?}", result),
        }
    }

    #[test]
    fn test_mkarr_single_element() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::MKARR { size: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                assert_eq!(borrowed.len(), 1);
                match &borrowed[0] {
                    Value::Number(n) => assert_eq!(*n, 42),
                    _ => panic!("Expected Number in array"),
                }
            }
            _ => panic!("Expected Array value"),
        }
    }

    #[test]
    fn test_mkarr_multiple_elements() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::LDCN { val: 3 },
            Instruction::MKARR { size: 3 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                assert_eq!(borrowed.len(), 3);

                // Check elements are in correct order
                match &borrowed[0] {
                    Value::Number(n) => assert_eq!(*n, 1),
                    _ => panic!("Expected Number at index 0"),
                }
                match &borrowed[1] {
                    Value::Number(n) => assert_eq!(*n, 2),
                    _ => panic!("Expected Number at index 1"),
                }
                match &borrowed[2] {
                    Value::Number(n) => assert_eq!(*n, 3),
                    _ => panic!("Expected Number at index 2"),
                }
            }
            _ => panic!("Expected Array value"),
        }
    }

    #[test]
    fn test_mkarr_mixed_types() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::LDCB { val: true },
            Instruction::LDSL {
                val: "hello".to_string(),
            },
            Instruction::MKARR { size: 3 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                assert_eq!(borrowed.len(), 3);

                match &borrowed[0] {
                    Value::Number(n) => assert_eq!(*n, 42),
                    _ => panic!("Expected Number at index 0"),
                }
                match &borrowed[1] {
                    Value::Bool(b) => assert_eq!(*b, true),
                    _ => panic!("Expected Bool at index 1"),
                }
                match &borrowed[2] {
                    Value::String(s) => assert_eq!(s, "hello"),
                    _ => panic!("Expected String at index 2"),
                }
            }
            _ => panic!("Expected Array value"),
        }
    }

    #[test]
    fn test_mkarr_nested_arrays() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create first inner array [1, 2]
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            // Create second inner array [3, 4]
            Instruction::LDCN { val: 3 },
            Instruction::LDCN { val: 4 },
            Instruction::MKARR { size: 2 },
            // Create outer array [[1, 2], [3, 4]]
            Instruction::MKARR { size: 2 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(outer) => {
                let borrowed_outer = outer.borrow();
                assert_eq!(borrowed_outer.len(), 2);

                // Check first inner array
                match &borrowed_outer[0] {
                    Value::Array(inner1) => {
                        let borrowed_inner1 = inner1.borrow();
                        assert_eq!(borrowed_inner1.len(), 2);
                        match &borrowed_inner1[0] {
                            Value::Number(n) => assert_eq!(*n, 1),
                            _ => panic!("Expected Number"),
                        }
                        match &borrowed_inner1[1] {
                            Value::Number(n) => assert_eq!(*n, 2),
                            _ => panic!("Expected Number"),
                        }
                    }
                    _ => panic!("Expected Array at index 0"),
                }

                // Check second inner array
                match &borrowed_outer[1] {
                    Value::Array(inner2) => {
                        let borrowed_inner2 = inner2.borrow();
                        assert_eq!(borrowed_inner2.len(), 2);
                        match &borrowed_inner2[0] {
                            Value::Number(n) => assert_eq!(*n, 3),
                            _ => panic!("Expected Number"),
                        }
                        match &borrowed_inner2[1] {
                            Value::Number(n) => assert_eq!(*n, 4),
                            _ => panic!("Expected Number"),
                        }
                    }
                    _ => panic!("Expected Array at index 1"),
                }
            }
            _ => panic!("Expected Array value"),
        }
    }

    #[test]
    fn test_mkarr_with_expression_results() {
        let mut vm = Machine::new();
        // Test array with computed values: [1 + 2, 3 * 4]
        let instructions = vec![
            // Compute 1 + 2
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::BINOP { ops: BINOPS::Add },
            // Compute 3 * 4
            Instruction::LDCN { val: 3 },
            Instruction::LDCN { val: 4 },
            Instruction::BINOP {
                ops: BINOPS::Multiply,
            },
            // Create array from results
            Instruction::MKARR { size: 2 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                assert_eq!(borrowed.len(), 2);

                match &borrowed[0] {
                    Value::Number(n) => assert_eq!(*n, 3), // 1 + 2
                    _ => panic!("Expected Number at index 0"),
                }
                match &borrowed[1] {
                    Value::Number(n) => assert_eq!(*n, 12), // 3 * 4
                    _ => panic!("Expected Number at index 1"),
                }
            }
            _ => panic!("Expected Array value"),
        }
    }

    #[test]
    fn test_mkarr_preserves_order() {
        let mut vm = Machine::new();
        // Verify that elements maintain their push order
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 20 },
            Instruction::LDCN { val: 30 },
            Instruction::LDCN { val: 40 },
            Instruction::LDCN { val: 50 },
            Instruction::MKARR { size: 5 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        match result {
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                let expected = vec![10, 20, 30, 40, 50];

                for (i, expected_val) in expected.iter().enumerate() {
                    match &borrowed[i] {
                        Value::Number(n) => assert_eq!(*n, *expected_val),
                        _ => panic!("Expected Number at index {}", i),
                    }
                }
            }
            _ => panic!("Expected Array value"),
        }
    }

    // ===== Array Indexing (LDAI/STAI) Tests =====
    //
    // These tests verify the execution of array indexing bytecode instructions.
    //
    // Instructions:
    // - LDAI: Load array element (read operation)
    //   Stack effect: [array, index] → [value]
    //
    // - STAI: Store array element (write operation)
    //   Stack effect: [array, index, value] → []
    //   Note: Modifies the array in place
    //
    // Test Coverage:
    // - LDAI: Reading elements at various indices
    // - STAI: Writing elements and verifying updates
    // - Error cases: out of bounds, wrong types, stack underflow
    // - Edge cases: first/last elements, nested arrays, computed indices
    // - Complex scenarios: read-write combinations, multiple updates

    #[test]
    fn test_ldai_simple() {
        let mut vm = Machine::new();
        // Create array [10, 20, 30] and read index 1
        let instructions = vec![
            // Create array
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 20 },
            Instruction::LDCN { val: 30 },
            Instruction::MKARR { size: 3 },
            // Duplicate array reference for later use
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            // Load array and index
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 1 },
            // Load array element at index 1
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(20)));
    }

    #[test]
    fn test_ldai_first_element() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 },
            Instruction::LDCN { val: 99 },
            Instruction::MKARR { size: 2 },
            Instruction::LDCN { val: 0 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_ldai_last_element() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::LDCN { val: 3 },
            Instruction::MKARR { size: 3 },
            Instruction::LDCN { val: 2 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(3)));
    }

    #[test]
    fn test_ldai_out_of_bounds() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::LDCN { val: 5 }, // Index out of bounds
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("array index out of bounds"));
    }

    #[test]
    fn test_ldai_negative_index_error() {
        let mut vm = Machine::new();
        // Negative indices are not supported (would become large usize)
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::LDCN { val: -1 }, // Negative index
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        // This will fail because -1 as usize is a very large number, causing out of bounds
        assert!(result.is_err());
    }

    #[test]
    fn test_ldai_non_number_index_error() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::LDCB { val: true }, // Boolean index - invalid
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("array index must be a number"));
    }

    #[test]
    fn test_ldai_non_array_error() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 }, // Number, not array
            Instruction::LDCN { val: 0 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot index into"));
    }

    #[test]
    fn test_stai_simple() {
        let mut vm = Machine::new();
        // Create array [1, 2, 3], store 99 at index 1, then read it
        let instructions = vec![
            // Create array
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::LDCN { val: 3 },
            Instruction::MKARR { size: 3 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            Instruction::POP,
            // Store 99 at arr[1]
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 99 },
            Instruction::STAI,
            // Read arr[1] to verify
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(99)));
    }

    #[test]
    fn test_stai_first_element() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            Instruction::POP,
            // Store 42 at index 0
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 42 },
            Instruction::STAI,
            // Read back
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_stai_multiple_updates() {
        let mut vm = Machine::new();
        // Update multiple elements in array
        let instructions = vec![
            // Create array [0, 0, 0]
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 0 },
            Instruction::MKARR { size: 3 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            Instruction::POP,
            // arr[0] = 10
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 10 },
            Instruction::STAI,
            Instruction::POP,
            // arr[1] = 20
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 20 },
            Instruction::STAI,
            Instruction::POP,
            // arr[2] = 30
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 2 },
            Instruction::LDCN { val: 30 },
            Instruction::STAI,
            Instruction::POP,
            // Read arr[1] to verify
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(20)));
    }

    #[test]
    fn test_stai_out_of_bounds() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::LDCN { val: 5 }, // Index out of bounds
            Instruction::LDCN { val: 99 },
            Instruction::STAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("array index out of bounds"));
    }

    #[test]
    fn test_stai_non_array_error() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDCN { val: 42 }, // Number, not array
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 99 },
            Instruction::STAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot index into"));
    }

    #[test]
    fn test_stai_overwrites_value() {
        let mut vm = Machine::new();
        // Create array, write to same index twice
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            Instruction::POP,
            // First write: arr[0] = 100
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 100 },
            Instruction::STAI,
            Instruction::POP,
            // Second write: arr[0] = 200
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDCN { val: 200 },
            Instruction::STAI,
            Instruction::POP,
            // Read arr[0] - should be 200
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(200)));
    }

    #[test]
    fn test_nested_array_indexing() {
        let mut vm = Machine::new();
        // Create [[1, 2], [3, 4]] and access [0][1] = 2
        let instructions = vec![
            // Create inner arrays
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 2 },
            Instruction::MKARR { size: 2 },
            Instruction::LDCN { val: 3 },
            Instruction::LDCN { val: 4 },
            Instruction::MKARR { size: 2 },
            // Create outer array
            Instruction::MKARR { size: 2 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            // Access arr[0]
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDAI,
            // Access [1] on the result
            Instruction::LDCN { val: 1 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(2)));
    }

    #[test]
    fn test_array_index_with_computed_index() {
        let mut vm = Machine::new();
        // arr[1 + 1] should access arr[2]
        let instructions = vec![
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 20 },
            Instruction::LDCN { val: 30 },
            Instruction::MKARR { size: 3 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            // Load array
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            // Compute index: 1 + 1 = 2
            Instruction::LDCN { val: 1 },
            Instruction::LDCN { val: 1 },
            Instruction::BINOP { ops: BINOPS::Add },
            // Access arr[2]
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(30)));
    }

    #[test]
    fn test_array_read_write_combined() {
        let mut vm = Machine::new();
        // arr[0] = arr[1] + 1
        let instructions = vec![
            // Create array [10, 20, 30]
            Instruction::LDCN { val: 10 },
            Instruction::LDCN { val: 20 },
            Instruction::LDCN { val: 30 },
            Instruction::MKARR { size: 3 },
            Instruction::ASSIGN {
                sym: "arr".to_string(),
            },
            Instruction::POP,
            // Read arr[1]
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::LDAI,
            // Add 1
            Instruction::LDCN { val: 1 },
            Instruction::BINOP { ops: BINOPS::Add },
            // Store to arr[0]
            Instruction::ASSIGN {
                sym: "temp".to_string(),
            },
            Instruction::POP,
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDS {
                sym: "temp".to_string(),
            },
            Instruction::STAI,
            Instruction::POP,
            // Read arr[0] to verify
            Instruction::LDS {
                sym: "arr".to_string(),
            },
            Instruction::LDCN { val: 0 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(21))); // 20 + 1
    }

    #[test]
    fn test_array_index_stack_underflow_no_array() {
        let mut vm = Machine::new();
        // LDAI with only index on stack (no array)
        let instructions = vec![
            Instruction::LDCN { val: 0 }, // Just index, no array
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("stack underflow, expected array or hash map")
        );
    }

    #[test]
    fn test_array_index_stack_underflow_no_index() {
        let mut vm = Machine::new();
        // LDAI with empty stack
        let instructions = vec![
            Instruction::LDAI, // Nothing on stack
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("stack underflow, expected index")
        );
    }

    // ===== Hash Map Tests =====
    //
    // These tests verify hash map creation, reading, and writing operations.
    //
    // Test Coverage:
    // - Creating hash maps (MKHASH instruction)
    // - Reading from hash maps (LDAI instruction with hash map)
    // - Writing to hash maps (STAI instruction with hash map)
    // - String and numeric keys
    // - Error handling for invalid keys and missing keys
    // - Complex scenarios (nested hash maps, mixed types, etc.)

    #[test]
    fn test_mkhash_empty() {
        let mut vm = Machine::new();
        let instructions = vec![Instruction::MKHASH { size: 0 }, Instruction::DONE];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::HashMap(_)));
    }

    #[test]
    fn test_mkhash_single_pair() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDSL {
                val: "key".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::MKHASH { size: 1 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::HashMap(_)));
    }

    #[test]
    fn test_mkhash_multiple_pairs() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::LDSL {
                val: "name".to_string(),
            },
            Instruction::LDSL {
                val: "Alice".to_string(),
            },
            Instruction::LDSL {
                val: "age".to_string(),
            },
            Instruction::LDCN { val: 30 },
            Instruction::MKHASH { size: 2 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::HashMap(_)));
    }

    #[test]
    fn test_mkhash_numeric_keys() {
        let mut vm = Machine::new();
        // Numeric keys should be converted to strings
        let instructions = vec![
            Instruction::LDCN { val: 1 },
            Instruction::LDSL {
                val: "first".to_string(),
            },
            Instruction::LDCN { val: 2 },
            Instruction::LDSL {
                val: "second".to_string(),
            },
            Instruction::MKHASH { size: 2 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::HashMap(_)));
    }

    #[test]
    fn test_hash_map_read_string_key() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create hash map {"name": "Alice"}
            Instruction::LDSL {
                val: "name".to_string(),
            },
            Instruction::LDSL {
                val: "Alice".to_string(),
            },
            Instruction::MKHASH { size: 1 },
            Instruction::ASSIGN {
                sym: "dict".to_string(),
            },
            // Read dict["name"]
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "name".to_string(),
            },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "Alice"));
    }

    #[test]
    fn test_hash_map_read_numeric_key() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create hash map {1: "value"}
            Instruction::LDCN { val: 1 },
            Instruction::LDSL {
                val: "value".to_string(),
            },
            Instruction::MKHASH { size: 1 },
            Instruction::ASSIGN {
                sym: "dict".to_string(),
            },
            // Read dict[1] (numeric key converted to string "1")
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "value"));
    }

    #[test]
    fn test_hash_map_write_new_key() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create empty hash map
            Instruction::MKHASH { size: 0 },
            Instruction::ASSIGN {
                sym: "dict".to_string(),
            },
            Instruction::POP,
            // Write dict["key"] = 42
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "key".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::STAI,
            Instruction::POP,
            // Read dict["key"]
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "key".to_string(),
            },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }

    #[test]
    fn test_hash_map_write_existing_key() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create hash map {"x": 10}
            Instruction::LDSL {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 10 },
            Instruction::MKHASH { size: 1 },
            Instruction::ASSIGN {
                sym: "dict".to_string(),
            },
            Instruction::POP,
            // Overwrite dict["x"] = 20
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 20 },
            Instruction::STAI,
            Instruction::POP,
            // Read dict["x"]
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "x".to_string(),
            },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(20)));
    }

    #[test]
    fn test_hash_map_read_missing_key_error() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create hash map {"x": 1}
            Instruction::LDSL {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 1 },
            Instruction::MKHASH { size: 1 },
            // Try to read non-existent key
            Instruction::LDSL {
                val: "y".to_string(),
            },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("key 'y' not found"));
    }

    #[test]
    fn test_hash_map_invalid_key_type_error() {
        let mut vm = Machine::new();
        let instructions = vec![
            Instruction::MKHASH { size: 0 },
            // Try to use boolean as key (invalid)
            Instruction::LDCB { val: true },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("key must be a string or number")
        );
    }

    #[test]
    fn test_hash_map_mixed_value_types() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create {"str": "hello", "num": 42, "bool": true}
            Instruction::LDSL {
                val: "str".to_string(),
            },
            Instruction::LDSL {
                val: "hello".to_string(),
            },
            Instruction::LDSL {
                val: "num".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::LDSL {
                val: "bool".to_string(),
            },
            Instruction::LDCB { val: true },
            Instruction::MKHASH { size: 3 },
            Instruction::ASSIGN {
                sym: "dict".to_string(),
            },
            // Read bool value
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "bool".to_string(),
            },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_hash_map_nested() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Create inner map {"inner": "value"}
            Instruction::LDSL {
                val: "inner".to_string(),
            },
            Instruction::LDSL {
                val: "value".to_string(),
            },
            Instruction::MKHASH { size: 1 },
            Instruction::ASSIGN {
                sym: "inner_map".to_string(),
            },
            Instruction::POP,
            // Create outer map {"outer": inner_map}
            Instruction::LDSL {
                val: "outer".to_string(),
            },
            Instruction::LDS {
                sym: "inner_map".to_string(),
            },
            Instruction::MKHASH { size: 1 },
            Instruction::ASSIGN {
                sym: "outer_map".to_string(),
            },
            // Access outer_map["outer"]["inner"]
            Instruction::LDS {
                sym: "outer_map".to_string(),
            },
            Instruction::LDSL {
                val: "outer".to_string(),
            },
            Instruction::LDAI,
            Instruction::LDSL {
                val: "inner".to_string(),
            },
            Instruction::LDAI,
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::String(s) if s == "value"));
    }

    #[test]
    fn test_mkhash_stack_underflow() {
        let mut vm = Machine::new();
        let instructions = vec![
            // Only push one value, but try to create hash with 2 pairs
            Instruction::LDSL {
                val: "key".to_string(),
            },
            Instruction::MKHASH { size: 2 },
            Instruction::DONE,
        ];
        let result = vm.run(&instructions);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("stack underflow while creating hash map")
        );
    }

    #[test]
    fn test_hash_map_stai_returns_value() {
        let mut vm = Machine::new();
        // Test that STAI pushes the value back (for use in expressions)
        let instructions = vec![
            Instruction::MKHASH { size: 0 },
            Instruction::ASSIGN {
                sym: "dict".to_string(),
            },
            Instruction::POP,
            // dict["x"] = 42, result should be 42
            Instruction::LDS {
                sym: "dict".to_string(),
            },
            Instruction::LDSL {
                val: "x".to_string(),
            },
            Instruction::LDCN { val: 42 },
            Instruction::STAI,
            // The 42 should now be on stack
            Instruction::DONE,
        ];
        let result = vm.run(&instructions).unwrap();
        assert!(matches!(result, Value::Number(42)));
    }
}
