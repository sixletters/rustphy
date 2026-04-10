use serde::{Deserialize, Serialize};

/// Bytecode instructions for the Goophy virtual machine.
///
/// Each instruction represents a single operation that the VM can execute.
/// Instructions manipulate the operand stack (OS) and runtime stack (RTS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    /// Load boolean constant onto the operand stack.
    ///
    /// Stack effect: [] → [bool]
    LDCB { val: bool },

    /// Load integer constant onto the operand stack.
    ///
    /// Stack effect: [] → [number]
    LDCN { val: i128 },

    /// Load identifier value onto the operand stack.
    ///
    /// Used for reading variable values in expressions.
    /// Stack effect: [] → [value]
    LDI { val: String },

    /// Load string literal onto the operand stack.
    ///
    /// Stack effect: [] → [string]
    LDSL { val: String },

    /// Load symbolic value from environment onto the operand stack.
    ///
    /// Used for accessing variables by their symbol name.
    /// Stack effect: [] → [value]
    LDS { sym: String },

    /// Load function closure onto the operand stack.
    ///
    /// Creates a closure capturing the current environment.
    /// Stack effect: [] → [closure]
    LDF { addr: usize, params: Vec<String> },

    /// Apply unary operator to the top stack value.
    ///
    /// Stack effect: [value] → [result]
    UNOP { ops: UNOPS },

    /// Apply binary operator to the top two stack values.
    ///
    /// Stack effect: [left, right] → [result]
    BINOP { ops: BINOPS },

    /// Make array from top N stack values.
    ///
    /// Pops `size` values from the operand stack, creates an array
    /// containing those values in order, and pushes the array back.
    ///
    /// # Example
    /// ```ignore
    /// // For: [1, 2, 3]
    /// LDCN 1      // Stack: [1]
    /// LDCN 2      // Stack: [1, 2]
    /// LDCN 3      // Stack: [1, 2, 3]
    /// MKARR 3     // Stack: [[1, 2, 3]]
    /// ```
    ///
    /// Stack effect: [v1, v2, ..., vN] → [array]
    MKARR { size: usize },

    /// Make hash map from top 2*size stack values.
    ///
    /// Pops `size` key-value pairs from the operand stack, creates a hash map,
    /// and pushes it back. Keys must be strings or numbers (numbers are converted to strings).
    ///
    /// # Example
    /// ```ignore
    /// // For: {"name": "Alice", "age": 30}
    /// LDSL "name"  // Stack: ["name"]
    /// LDSL "Alice" // Stack: ["name", "Alice"]
    /// LDSL "age"   // Stack: ["name", "Alice", "age"]
    /// LDCN 30      // Stack: ["name", "Alice", "age", 30]
    /// MKHASH 2     // Stack: [{"name": "Alice", "age": 30}]
    /// ```
    ///
    /// Stack effect: [k1, v1, k2, v2, ..., kN, vN] → [hashmap]
    MKHASH { size: usize },

    /// Jump on false - conditional branch.
    ///
    /// If top of stack is falsy, jump to address; otherwise continue.
    /// Stack effect: [condition] → []
    JOF { addr: usize },

    /// Unconditional jump to address.
    ///
    /// Stack effect: [] → []
    GOTO { addr: usize },

    /// Halt execution and return top of stack.
    ///
    /// Stack effect: [result] → [result]
    DONE,

    /// Enter a new scope, declaring symbols.
    ///
    /// Pushes a new stack frame onto the RTS.
    /// Stack effect: [] → []
    ENTERSCOPE { syms: Vec<String> },

    /// Exit current scope, restoring previous environment.
    ///
    /// Pops the current stack frame from the RTS.
    /// Stack effect: [] → []
    EXITSCOPE,

    /// Pop and discard the top value from the operand stack.
    ///
    /// Stack effect: [value] → []
    POP,

    /// Assign top of stack to a symbol in the current environment.
    ///
    /// Stack effect: [value] → [value] (leaves value on stack)
    ASSIGN { sym: String },

    /// Call function with arity arguments.
    ///
    /// Stack effect: [function, arg1, ..., argN] → [result]
    CALL { arity: usize },

    /// Tail call function with arity arguments.
    ///
    /// Optimized call that reuses the current stack frame.
    /// Stack effect: [function, arg1, ..., argN] → [result]
    TAILCALL { arity: usize },

    /// Reset the VM state.
    ///
    /// Stack effect: [] → []
    RESET,

    /// Load element from array or hash map at index/key (read operation)
    ///
    /// Pops an index/key and a container (array or hash map) from the stack,
    /// retrieves the element at that index/key, and pushes it back.
    ///
    /// For arrays: index must be a number
    /// For hash maps: key must be a string or number (numbers are converted to strings)
    ///
    /// # Examples
    /// ```ignore
    /// // Array: arr[2]
    /// LDS "arr"    // Stack: [array]
    /// LDCN 2       // Stack: [array, 2]
    /// LDAI         // Stack: [array[2]]
    ///
    /// // Hash map: dict["key"]
    /// LDS "dict"     // Stack: [hashmap]
    /// LDSL "key"     // Stack: [hashmap, "key"]
    /// LDAI           // Stack: [hashmap["key"]]
    /// ```
    ///
    /// Stack effect: [container, index/key] → [value]
    LDAI,

    /// Store value into array or hash map at index/key (write operation)
    ///
    /// Pops a value, an index/key, and a container (array or hash map) from the stack,
    /// stores the value at that index/key, and pushes the value back onto the stack.
    ///
    /// For arrays: index must be a number, index must be within bounds
    /// For hash maps: key must be a string or number (numbers are converted to strings),
    ///                creates new entry if key doesn't exist
    ///
    /// # Examples
    /// ```ignore
    /// // Array: arr[2] = 42
    /// LDS "arr"    // Stack: [array]
    /// LDCN 2       // Stack: [array, 2]
    /// LDCN 42      // Stack: [array, 2, 42]
    /// STAI         // Stack: [42] (also modifies array)
    ///
    /// // Hash map: dict["key"] = "value"
    /// LDS "dict"     // Stack: [hashmap]
    /// LDSL "key"     // Stack: [hashmap, "key"]
    /// LDSL "value"   // Stack: [hashmap, "key", "value"]
    /// STAI           // Stack: ["value"] (also modifies hashmap)
    /// ```
    ///
    /// Stack effect: [container, index/key, value] → [value]
    STAI,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UNOPS {
    Negative, // "-unary"
    Not,      // "!"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BINOPS {
    Add,      // "+"
    Multiply, // "*"
    Minus,    // "-"
    Divide,   // "/"
    Modulo,   // "%"
    Lt,       // "<"
    Le,       // "<="
    Ge,       // ">="
    Gt,       // ">"
    Eq,       // "==="
    Neq,      // "!=="
    And,      // "&&"
    Or,       // "||"
    Assign,
}
