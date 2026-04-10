//! WebAssembly Runtime Environment
//!
//! This module contains the runtime support system for dynamic typing in WASM.
//! It generates WAT (WebAssembly Text) code for:
//! - Value tagging/untagging (converting between tagged and untagged values)
//! - Type checking predicates (is_immediate, is_pointer)
//! - Heap allocation (bump allocator for objects)
//! - Environment management (lexical scope chains)
//! - Argument passing structs for closures
//! - Closure creation and indirect calls
//! - String operations (create, concat, length)
//! - Arithmetic operations on tagged values
//! - Comparison operations on tagged values
//!
//! # Tagging Scheme
//!
//! All values in our system are represented as i32 ints.
//!
//! We use 1-bit pointer tagging (nan-boxing lite):
//! - If `value & 1 == 1`: immediate integer  → actual value = `value >> 1`
//! - If `value & 1 == 0`: heap pointer       → actual address = the value itself
//!
//! Aligned heap allocations always have an even address, so bit 0 is always 0
//! for real pointers, making this safe.
//!
//! ```text
//! ┌──────────────────────────────┬─┐
//! │      Payload (31 bits)       │T│   T = tag bit (1 bit)
//! └──────────────────────────────┴─┘
//!
//! T = 1 → immediate integer  (tag_immediate: (val << 1) | 1)
//! T = 0 → heap pointer       (the raw aligned address)
//! ```
//!
//! # Heap Object Layout
//!
//! All heap objects start with an 8-byte header:
//!
//! ```text
//! Offset 0: type_tag (i32)
//! Offset 4: length / size (i32)
//! Offset 8: data...
//!
//! Type tags:
//!   TYPE_STRING  = 0  →  [0][byte_len][utf8 bytes...]
//!   TYPE_ARRAY   = 1  →  [1][length][tagged_elem0][tagged_elem1]...
//!   TYPE_CLOSURE = 2  →  [2][func_idx][env_ptr]
//! ```
//!
//! # Environment (Lexical Scope) Layout
//!
//! ```text
//! Offset 0: parent_ptr (i32) — pointer to enclosing env, 0 if global
//! Offset 4: var_count  (i32) — number of captured variables
//! Offset 8: var0, var4, var8, ... (tagged i32 values)
//! ```
//!
//! # Argument Struct Layout
//!
//! Used to pass arguments to closures via `$call_closure`:
//!
//! ```text
//! Offset 0:            (i32) — reserved / length slot (set by caller)
//! Offset 4:  arg0      (tagged i32)
//! Offset 8:  arg1      (tagged i32)
//! ...
//! ```
//!
//! # Closure Layout
//!
//! ```text
//! Offset 0: type_tag = 2  (i32)
//! Offset 4: func_idx      (i32) — index into the function table
//! Offset 8: env_ptr       (i32) — pointer to captured environment
//! ```
//!
//! All closure functions share the uniform signature:
//!   `(func (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32))`

use std::fmt::Write as FmtWrite;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Indentation string (3 spaces)
const INDENT: &str = "   ";

// ============================================================================
// WASM RUNTIME ENVIRONMENT
// ============================================================================

/// Generates WAT code for the complete WASM runtime environment
#[derive(Clone)]
pub struct WasmRuntime {
    output: String,
    indent_level: usize,
}

pub struct FunctionBuilder<'a> {
    runtime: &'a mut WasmRuntime,
    name: String,
    params: Vec<String>,  // ["$val i32", "$x i32"]
    results: Vec<String>, // ["i32"]
    locals: Vec<String>,  // ["$temp i32"]
    body: Vec<String>,    // Instructions
    base_indent: usize,   // Starting indentation level
}

// Optional - for complex control flow (future enhancement)
#[allow(dead_code)]
pub struct BlockBuilder<'a> {
    function: &'a mut FunctionBuilder<'a>,
    block_type: BlockType, // If, Loop, Block
}

#[allow(dead_code)]
enum BlockType {
    If,
    IfElse,
    Loop,
    Block,
}

impl<'a> FunctionBuilder<'a> {
    /// Add a parameter to the function
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$add")
    ///     .param("$a", "i32")
    ///     .param("$b", "i32")
    ///     .result("i32");
    /// ```
    pub fn param(mut self, name: &str, type_: &str) -> Self {
        self.params.push(format!("(param {} {})", name, type_));
        self
    }

    /// Add multiple parameters at once
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$add")
    ///     .params(&[("$a", "i32"), ("$b", "i32")])
    ///     .result("i32");
    /// ```
    pub fn params(mut self, params: &[(String, String)]) -> Self {
        for (name, type_) in params {
            self.params.push(format!("(param {} {})", name, type_));
        }
        self
    }

    /// Add a result type to the function
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$get_value")
    ///     .result("i32");
    /// ```
    pub fn result(mut self, type_: &str) -> Self {
        self.results.push(format!("(result {})", type_));
        self
    }

    /// Add a local variable to the function
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$complex")
    ///     .param("$x", "i32")
    ///     .local("$temp", "i32")
    ///     .result("i32");
    /// ```
    pub fn local(mut self, name: &str, type_: &str) -> Self {
        self.locals.push(format!("(local {} {})", name, type_));
        self
    }

    /// Add a single instruction to the function body
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$tag_int")
    ///     .param("$val", "i32")
    ///     .result("i32")
    ///     .inst("local.get $val")
    ///     .inst("i32.const 2")
    ///     .inst("i32.shl")
    ///     .build();
    /// ```
    pub fn inst(mut self, instruction: &str) -> Self {
        self.body.push(instruction.to_string());
        self
    }

    /// Add instruction without consuming self (for use in closures)
    ///
    /// This method is useful inside the `body()` closure where you don't
    /// want to consume the builder.
    pub fn push_inst(&mut self, instruction: &str) {
        self.body.push(instruction.to_string());
    }

    /// Add multiple instructions at once
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$tag_int")
    ///     .param("$val", "i32")
    ///     .result("i32")
    ///     .emit_body(&[
    ///         "local.get $val",
    ///         "i32.const 2",
    ///         "i32.shl"
    ///     ]);
    /// ```
    pub fn emit_body(mut self, instructions: &[&str]) -> Self {
        for inst in instructions {
            self.body.push(inst.to_string());
        }
        self
    }

    /// Build the function body using a closure
    ///
    /// Allows for more complex body construction with conditionals.
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$complex")
    ///     .param("$x", "i32")
    ///     .result("i32")
    ///     .body(|f| {
    ///         f.inst("local.get $x");
    ///         f.inst("i32.const 1");
    ///         f.inst("i32.add");
    ///     });
    /// ```
    pub fn body<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut FunctionBuilder),
    {
        f(&mut self);
        self
    }

    /// Finalize and emit the function to the runtime
    ///
    /// This consumes the builder and writes the complete function to the
    /// WasmRuntime output buffer. The function is emitted at the indentation
    /// level that was active when the builder was created (base_indent).
    pub fn build(self) {
        // Set runtime indent to the level when function() was called
        self.runtime.indent_level = self.base_indent;

        // Build function header
        let mut header = format!("(func {}", self.name);

        // Add params
        if !self.params.is_empty() {
            header.push(' ');
            header.push_str(&self.params.join(" "));
        }

        // Add results
        if !self.results.is_empty() {
            header.push(' ');
            header.push_str(&self.results.join(" "));
        }

        // Emit function header at base_indent level
        self.runtime.emit_line(&header);
        self.runtime.increment_indent();

        // Emit locals at base_indent + 1
        for local in &self.locals {
            self.runtime.emit_line(local);
        }

        // Emit body instructions at base_indent + 1
        for inst in &self.body {
            self.runtime.emit_line(inst);
        }

        // Close function - decrement back to base_indent
        self.runtime.decrement_indent();
        self.runtime.emit_line(")");
    }

    /// Alias for build() - finalizes and emits the function
    pub fn emit(self) {
        self.build()
    }
}

impl WasmRuntime {
    pub fn new() -> Self {
        WasmRuntime {
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Start building a function using the builder pattern
    ///
    /// Returns a `FunctionBuilder` that allows fluent construction of WAT functions.
    ///
    /// # Example
    /// ```ignore
    /// runtime.function("$tag_int")
    ///     .param("$val", "i32")
    ///     .result("i32")
    ///     .inst("local.get $val")
    ///     .inst("i32.const 2")
    ///     .inst("i32.shl")
    ///     .build();
    /// ```
    pub fn function(&mut self, name: &str) -> FunctionBuilder<'_> {
        let current_indent = self.indent_level;
        FunctionBuilder {
            runtime: self,
            name: name.to_string(),
            params: vec![],
            results: vec![],
            locals: vec![],
            body: vec![],
            base_indent: current_indent,
        }
    }

    /// Shorthand for `function()` - start building a function
    ///
    /// # Example
    /// ```ignore
    /// runtime.func("$add")
    ///     .params(&[("$a", "i32"), ("$b", "i32")])
    ///     .result("i32")
    ///     .emit_body(&["local.get $a", "local.get $b", "i32.add"]);
    /// ```
    pub fn func(&mut self, name: &str) -> FunctionBuilder<'_> {
        self.function(name)
    }

    pub fn emit(&mut self, instr: &str) {
        writeln!(self.output, "{}\n", instr).unwrap();
    }

    pub fn increment_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrement_indent(&mut self) {
        self.indent_level -= 1;
    }

    /// Generate all runtime helper functions as WAT
    ///
    /// Emits helpers in dependency order:
    /// 1. Tag helpers (primitives everything else depends on)
    /// 2. Type check helpers
    /// 3. Heap allocator
    /// 4. Environment (lexical scope chains)
    /// 5. Argument structs (closure calling convention)
    /// 6. Closures (create + call_indirect)
    /// 7. String operations (create, concat, length)
    /// 8. Arithmetic on tagged values
    /// 9. Comparisons on tagged values
    pub fn generate_all(&mut self) -> String {
        self.output.clear();

        self.generate_tag_helpers();
        self.generate_type_check_helpers();
        self.generate_heap_alloc();
        self.generate_env_helpers();
        self.generate_arg_helpers();
        self.generate_closure_helpers();
        self.generate_string_helpers();
        self.generate_array_helpers();
        self.generate_arithmetic_helpers();
        self.generate_comparison_helpers();

        self.output.clone()
    }

    /// Get the generated WAT output
    ///
    /// Returns a reference to the output string containing all generated WAT code.
    pub fn get_output(&self) -> &str {
        &self.output
    }

    /// Get the current indentation level
    ///
    /// Useful for testing and debugging.
    pub fn get_indent_level(&self) -> usize {
        self.indent_level
    }

    pub fn set_indent_level(&mut self, new_level: usize) {
        self.indent_level = new_level;
    }

    // ========================================================================
    // TAG/UNTAG HELPERS
    // ========================================================================

    pub fn generate_tag_helpers(&mut self) {
        self.emit_comment("Tag/Untag Helper Functions (1-bit LSB scheme)");

        // tag_immediate: (val << 1) | 1  — sets bit 0 to mark as integer
        self.func("$tag_immediate")
            .param("$value", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $value");
                f.push_inst("i32.const 1");
                f.push_inst("i32.shl");
                f.push_inst("i32.const 1");
                f.push_inst("i32.or");
            })
            .build();

        // untag_immediate: val >> 1 (signed) — recovers the original integer
        self.func("$untag_immediate")
            .param("$value", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $value");
                f.push_inst("i32.const 1");
                f.push_inst("i32.shr_s");
            })
            .build();

        // is_immediate: val & 1 — returns 1 if tagged integer, 0 if heap pointer
        self.func("$is_immediate")
            .param("$value", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $value");
                f.push_inst("i32.const 1");
                f.push_inst("i32.and");
            })
            .build();

        // is_pointer: (val & 1) == 0 — returns 1 if heap pointer, 0 if integer
        self.func("$is_pointer")
            .param("$value", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $value");
                f.push_inst("i32.const 1");
                f.push_inst("i32.and");
                f.push_inst("i32.eqz");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // TYPE CHECKING HELPERS
    // ========================================================================

    fn generate_type_check_helpers(&mut self) {
        self.emit_comment("Heap Object Type Checking");

        // Read the type_tag from offset 0 of a heap object.
        // Caller must ensure the value is actually a pointer (use $is_pointer first).
        self.func("$heap_type_tag")
            .param("$ptr", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $ptr");
                f.push_inst("i32.load");
            })
            .build();

        // is_string: heap[ptr+0] == 0 (TYPE_STRING)
        self.func("$is_string")
            .param("$ptr", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $ptr");
                f.push_inst("call $heap_type_tag");
                f.push_inst("i32.const 0");
                f.push_inst("i32.eq");
            })
            .build();

        // is_array: heap[ptr+0] == 1 (TYPE_ARRAY)
        self.func("$is_array")
            .param("$ptr", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $ptr");
                f.push_inst("call $heap_type_tag");
                f.push_inst("i32.const 1");
                f.push_inst("i32.eq");
            })
            .build();

        // is_closure: heap[ptr+0] == 2 (TYPE_CLOSURE)
        self.func("$is_closure")
            .param("$ptr", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $ptr");
                f.push_inst("call $heap_type_tag");
                f.push_inst("i32.const 2");
                f.push_inst("i32.eq");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // HEAP ALLOCATION
    // ========================================================================

    pub fn generate_heap_alloc(&mut self) {
        self.emit_comment("Heap Allocation (Bump Allocator)");

        // Simple bump allocator - allocate N bytes and return pointer
        self.func("$alloc")
            .param("$size", "i32")
            .result("i32")
            .local("$ptr", "i32")
            .body(|f| {
                f.push_inst(";; Save current heap pointer");
                f.push_inst("global.get $heap_ptr");
                f.push_inst("local.set $ptr");
                f.push_inst("");
                f.push_inst(";; Bump heap pointer forward");
                f.push_inst("global.get $heap_ptr");
                f.push_inst("local.get $size");
                f.push_inst("i32.add");
                f.push_inst("global.set $heap_ptr");
                f.push_inst("");
                f.push_inst(";; Return old pointer");
                f.push_inst("local.get $ptr");
            })
            .build();

        // Allocate a heap object with type tag and size header
        self.func("$heap_alloc")
            .param("$type_tag", "i32")
            .param("$size", "i32")
            .result("i32")
            .local("$ptr", "i32")
            .local("$total_size", "i32")
            .body(|f| {
                f.push_inst(";; Calculate total size: 8 bytes header + size");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $size");
                f.push_inst("i32.add");
                f.push_inst("local.set $total_size");
                f.push_inst("");
                f.push_inst(";; Allocate memory");
                f.push_inst("local.get $total_size");
                f.push_inst("call $alloc");
                f.push_inst("local.set $ptr");
                f.push_inst("");
                f.push_inst(";; Write type tag at offset 0");
                f.push_inst("local.get $ptr");
                f.push_inst("local.get $type_tag");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Write size at offset 4");
                f.push_inst("local.get $ptr");
                f.push_inst("local.get $size");
                f.push_inst("i32.store offset=4");
                f.push_inst("");
                f.push_inst(";; Return pointer to object");
                f.push_inst("local.get $ptr");
            })
            .build();

        // Allocate a string object
        self.func("$alloc_string")
            .param("$length", "i32")
            .result("i32")
            .local("$ptr", "i32")
            .body(|f| {
                f.push_inst(";; Allocate string object (TYPE_STRING, length)");
                f.push_inst("global.get $TYPE_STRING");
                f.push_inst("local.get $length");
                f.push_inst("call $heap_alloc");
                f.push_inst("local.set $ptr");
                f.push_inst("");
                f.push_inst(";; Return pointer");
                f.push_inst("local.get $ptr");
            })
            .build();

        // Write a byte to a string at given index
        self.func("$string_set")
            .param("$ptr", "i32")
            .param("$index", "i32")
            .param("$byte", "i32")
            .body(|f| {
                f.push_inst(";; Calculate address: ptr + 8 (header) + index");
                f.push_inst("local.get $ptr");
                f.push_inst("i32.const 8");
                f.push_inst("i32.add");
                f.push_inst("local.get $index");
                f.push_inst("i32.add");
                f.push_inst(";; Store byte");
                f.push_inst("local.get $byte");
                f.push_inst("i32.store8");
            })
            .build();

        // Read a byte from a string at given index
        self.func("$string_get")
            .param("$ptr", "i32")
            .param("$index", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Calculate address: ptr + 8 (header) + index");
                f.push_inst("local.get $ptr");
                f.push_inst("i32.const 8");
                f.push_inst("i32.add");
                f.push_inst("local.get $index");
                f.push_inst("i32.add");
                f.push_inst(";; Load byte");
                f.push_inst("i32.load8_u");
            })
            .build();

        // Get string length
        self.func("$string_length")
            .param("$ptr", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Load length from offset 4");
                f.push_inst("local.get $ptr");
                f.push_inst("i32.load offset=4");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // ARITHMETIC OPERATIONS
    // ========================================================================

    pub fn generate_arithmetic_helpers(&mut self) {
        self.emit_comment("Arithmetic Operations on Tagged Values");

        // Addition — fast path when both operands are immediates (matching task_2.wat)
        self.func("$add_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("call $is_immediate");
                f.push_inst("local.get $b");
                f.push_inst("call $is_immediate");
                f.push_inst("i32.and");
                f.push_inst("if (result i32)");
                f.push_inst("   local.get $a");
                f.push_inst("   call $untag_immediate");
                f.push_inst("   local.get $b");
                f.push_inst("   call $untag_immediate");
                f.push_inst("   i32.add");
                f.push_inst("   call $tag_immediate");
                f.push_inst("else");
                f.push_inst("   ;; heap object addition not yet supported");
                f.push_inst("   i32.const 0");
                f.push_inst("end");
            })
            .build();

        // Subtraction
        self.func("$sub_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("call $untag_immediate");
                f.push_inst("local.get $b");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.sub");
                f.push_inst("call $tag_immediate");
            })
            .build();

        // Multiplication
        self.func("$mul_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("call $untag_immediate");
                f.push_inst("local.get $b");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.mul");
                f.push_inst("call $tag_immediate");
            })
            .build();

        // Division (signed)
        self.func("$div_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("call $untag_immediate");
                f.push_inst("local.get $b");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.div_s");
                f.push_inst("call $tag_immediate");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // COMPARISON OPERATIONS
    // ========================================================================

    pub fn generate_comparison_helpers(&mut self) {
        self.emit_comment("Comparison Operations on Tagged Values");

        // Less than — result is 0 or 1, returned as tagged immediate
        self.func("$lt_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("call $untag_immediate");
                f.push_inst("local.get $b");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.lt_s");
                f.push_inst("call $tag_immediate");
            })
            .build();

        // Greater than
        self.func("$gt_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("call $untag_immediate");
                f.push_inst("local.get $b");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.gt_s");
                f.push_inst("call $tag_immediate");
            })
            .build();

        // Equals — tagged values for primitives compare equal iff the raw i32 is equal
        self.func("$eq_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("local.get $b");
                f.push_inst("i32.eq");
                f.push_inst("call $tag_immediate");
            })
            .build();

        // Not equals
        self.func("$ne_values")
            .param("$a", "i32")
            .param("$b", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $a");
                f.push_inst("local.get $b");
                f.push_inst("i32.ne");
                f.push_inst("call $tag_immediate");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // ENVIRONMENT HELPERS
    // ========================================================================
    // Environments form a linked list of lexical scopes.
    // Layout: [parent_ptr (i32)][var_count (i32)][var0 (i32)][var1 (i32)]...
    //
    // - parent_ptr = 0 means global/top-level scope
    // - var_count  = number of slots allocated in this frame
    // - variables are accessed by index (0-based) relative to this frame

    pub fn generate_env_helpers(&mut self) {
        self.emit_comment("Environment Helpers (lexical scope chains)");

        // Allocate a new env frame with `var_count` variable slots.
        // Stores parent_ptr and var_count in the header; slots are zero-initialised
        // by the WASM bump allocator (memory starts zeroed).
        self.func("$create_env")
            .param("$parent_ptr", "i32")
            .param("$var_count", "i32")
            .result("i32")
            .local("$env_ptr", "i32")
            .body(|f| {
                f.push_inst(";; Allocate 8 (header) + var_count * 4 bytes");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $var_count");
                f.push_inst("i32.const 4");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("call $alloc");
                f.push_inst("local.set $env_ptr");
                f.push_inst("");
                f.push_inst(";; Store parent_ptr at offset 0");
                f.push_inst("local.get $env_ptr");
                f.push_inst("local.get $parent_ptr");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Store var_count at offset 4");
                f.push_inst("local.get $env_ptr");
                f.push_inst("local.get $var_count");
                f.push_inst("i32.store offset=4");
                f.push_inst("");
                f.push_inst("local.get $env_ptr");
            })
            .build();

        // Read the tagged value stored at slot `$index` in the env frame.
        // Address = env_ptr + 8 + index * 4
        self.func("$env_get")
            .param("$env_ptr", "i32")
            .param("$index", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Address = env_ptr + 8 + index * 4");
                f.push_inst("local.get $env_ptr");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $index");
                f.push_inst("i32.const 4");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("i32.add");
                f.push_inst("i32.load");
            })
            .build();

        // Write a tagged value into slot `$index` of the env frame.
        // Address = env_ptr + 8 + index * 4
        self.func("$env_set")
            .param("$env_ptr", "i32")
            .param("$index", "i32")
            .param("$value", "i32")
            .body(|f| {
                f.push_inst(";; Address = env_ptr + 8 + index * 4");
                f.push_inst("local.get $env_ptr");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $index");
                f.push_inst("i32.const 4");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("i32.add");
                f.push_inst("local.get $value");
                f.push_inst("i32.store");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // ARGUMENT STRUCT HELPERS
    // ========================================================================
    // Argument structs are used to pass arguments to closures uniformly.
    // Layout: [reserved/length (i32)][arg0 (tagged i32)][arg1 (tagged i32)]...
    //
    // - Slot 0 (offset 0) is reserved; callers may write the arg count there.
    // - Arguments start at offset 4 (index 0 → offset 4, index 1 → offset 8, ...).
    // - The caller must call $create_arg with the number of arguments, then
    //   $arg_set for each argument before calling $call_closure.

    pub fn generate_arg_helpers(&mut self) {
        self.emit_comment("Argument Struct Helpers (closure calling convention)");

        // Allocate an arg struct for `$length` arguments.
        // Size = 4 (header slot) + 4 * length bytes.
        self.func("$create_arg")
            .param("$length", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Allocate 4 (header) + length * 4 bytes");
                f.push_inst("i32.const 4");
                f.push_inst("i32.const 4");
                f.push_inst("local.get $length");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("call $alloc");
            })
            .build();

        // Write a tagged value at argument slot `$idx`.
        // Address = arg_struct_ptr + 4 + idx * 4.
        // Returns the arg_struct_ptr so calls can be chained.
        self.func("$arg_set")
            .param("$arg_struct_ptr", "i32")
            .param("$idx", "i32")
            .param("$value", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Address = arg_struct_ptr + 4 + idx * 4");
                f.push_inst("i32.const 4");
                f.push_inst("i32.const 4");
                f.push_inst("local.get $idx");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("local.get $arg_struct_ptr");
                f.push_inst("i32.add");
                f.push_inst("local.get $value");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Return arg_struct_ptr for chaining");
                f.push_inst("local.get $arg_struct_ptr");
            })
            .build();

        // Read the tagged value at argument slot `$idx`.
        // Address = arg_struct_ptr + 4 + idx * 4.
        self.func("$arg_get")
            .param("$arg_struct_ptr", "i32")
            .param("$idx", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Address = arg_struct_ptr + 4 + idx * 4");
                f.push_inst("local.get $arg_struct_ptr");
                f.push_inst("i32.const 4");
                f.push_inst("i32.const 4");
                f.push_inst("local.get $idx");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("i32.add");
                f.push_inst("i32.load");
            })
            .build();

        // Read the length value stored at offset 0 of the arg struct.
        // Callers are responsible for writing this if they need it.
        self.func("$arg_length")
            .param("$arg_struct_ptr", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $arg_struct_ptr");
                f.push_inst("i32.load");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // CLOSURE HELPERS
    // ========================================================================
    // Closures capture a function table index and an environment pointer.
    // Layout: [TYPE_CLOSURE=2 (i32)][func_idx (i32)][env_ptr (i32)]  (12 bytes)
    //
    // All closure functions share the uniform signature:
    //   (func (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32))
    //
    // This means every function in the table can be called uniformly via
    // call_indirect using $function_type, regardless of how many arguments
    // it actually needs — the arg struct carries them.

    pub fn generate_closure_helpers(&mut self) {
        self.emit_comment("Closure Helpers");

        // Allocate a 12-byte closure object on the heap.
        // Stores: [TYPE_CLOSURE=2][func_idx][env_ptr]
        self.func("$create_closure")
            .param("$func_idx", "i32")
            .param("$env_ptr", "i32")
            .result("i32")
            .local("$closure_ptr", "i32")
            .body(|f| {
                f.push_inst(";; Allocate 12 bytes: [type_tag][func_idx][env_ptr]");
                f.push_inst("i32.const 12");
                f.push_inst("call $alloc");
                f.push_inst("local.set $closure_ptr");
                f.push_inst("");
                f.push_inst(";; Store TYPE_CLOSURE at offset 0");
                f.push_inst("local.get $closure_ptr");
                f.push_inst("global.get $TYPE_CLOSURE");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Store func_idx at offset 4");
                f.push_inst("local.get $closure_ptr");
                f.push_inst("local.get $func_idx");
                f.push_inst("i32.store offset=4");
                f.push_inst("");
                f.push_inst(";; Store env_ptr at offset 8");
                f.push_inst("local.get $closure_ptr");
                f.push_inst("local.get $env_ptr");
                f.push_inst("i32.store offset=8");
                f.push_inst("");
                f.push_inst("local.get $closure_ptr");
            })
            .build();

        // Call a closure by extracting its func_idx and env_ptr, then
        // dispatching via call_indirect through the function table.
        // The callee receives (env_ptr, arg_struct_ptr) per the uniform signature.
        self.func("$call_closure")
            .param("$closure_ptr", "i32")
            .param("$arg_struct_ptr", "i32")
            .result("i32")
            .local("$func_idx", "i32")
            .local("$env", "i32")
            .body(|f| {
                f.push_inst(";; Load func_idx from offset 4");
                f.push_inst("local.get $closure_ptr");
                f.push_inst("i32.load offset=4");
                f.push_inst("local.set $func_idx");
                f.push_inst("");
                f.push_inst(";; Load env_ptr from offset 8");
                f.push_inst("local.get $closure_ptr");
                f.push_inst("i32.load offset=8");
                f.push_inst("local.set $env");
                f.push_inst("");
                f.push_inst(
                    ";; Dispatch: call_indirect expects (env_ptr, arg_struct_ptr, func_idx)",
                );
                f.push_inst("local.get $env");
                f.push_inst("local.get $arg_struct_ptr");
                f.push_inst("local.get $func_idx");
                f.push_inst("call_indirect (type $function_type)");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // STRING HELPERS
    // ========================================================================
    // Higher-level string operations that complement the low-level byte
    // accessors ($string_get / $string_set) already emitted by generate_heap_alloc.
    //
    // String layout on the heap:
    //   Offset 0: TYPE_STRING = 0  (i32)
    //   Offset 4: byte_length      (i32)
    //   Offset 8: UTF-8 bytes...
    //
    // $create_string copies raw bytes from a linear-memory data pointer into a
    // freshly allocated heap object.  $string_concat allocates a new object and
    // copies both strings into it byte-by-byte.

    pub fn generate_string_helpers(&mut self) {
        self.emit_comment("String Helpers");

        // Allocate a string object and copy `$length` bytes from `$data_ptr`.
        // Uses a loop to copy one byte at a time.
        self.func("$create_string")
            .param("$data_ptr", "i32")
            .param("$length", "i32")
            .result("i32")
            .local("$str_ptr", "i32")
            .local("$i", "i32")
            .body(|f| {
                f.push_inst(";; Allocate 8-byte header + length bytes");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $length");
                f.push_inst("i32.add");
                f.push_inst("call $alloc");
                f.push_inst("local.set $str_ptr");
                f.push_inst("");
                f.push_inst(";; Store TYPE_STRING at offset 0");
                f.push_inst("local.get $str_ptr");
                f.push_inst("global.get $TYPE_STRING");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Store byte length at offset 4");
                f.push_inst("local.get $str_ptr");
                f.push_inst("local.get $length");
                f.push_inst("i32.store offset=4");
                f.push_inst("");
                f.push_inst(";; Copy bytes from data_ptr into the string body");
                f.push_inst("(block $done");
                f.push_inst("   (loop $copy");
                f.push_inst("      local.get $i");
                f.push_inst("      local.get $length");
                f.push_inst("      i32.ge_u");
                f.push_inst("      br_if $done");
                f.push_inst("");
                f.push_inst("      local.get $str_ptr");
                f.push_inst("      i32.const 8");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.add");
                f.push_inst("      i32.add");
                f.push_inst("      local.get $data_ptr");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.add");
                f.push_inst("      i32.load8_u");
                f.push_inst("      i32.store8");
                f.push_inst("");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.const 1");
                f.push_inst("      i32.add");
                f.push_inst("      local.set $i");
                f.push_inst("      br $copy");
                f.push_inst("   )");
                f.push_inst(")");
                f.push_inst("");
                f.push_inst("local.get $str_ptr");
            })
            .build();

        // Concatenate two heap strings into a new string.
        // Reads lengths from each string header, allocates a combined object,
        // then copies str1 bytes followed by str2 bytes.
        self.func("$string_concat")
            .param("$str1", "i32")
            .param("$str2", "i32")
            .result("i32")
            .local("$len1", "i32")
            .local("$len2", "i32")
            .local("$new_str", "i32")
            .local("$i", "i32")
            .body(|f| {
                f.push_inst(";; Load lengths from each string header (offset 4)");
                f.push_inst("local.get $str1");
                f.push_inst("i32.load offset=4");
                f.push_inst("local.set $len1");
                f.push_inst("local.get $str2");
                f.push_inst("i32.load offset=4");
                f.push_inst("local.set $len2");
                f.push_inst("");
                f.push_inst(";; Allocate 8 (header) + len1 + len2 bytes");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $len1");
                f.push_inst("local.get $len2");
                f.push_inst("i32.add");
                f.push_inst("i32.add");
                f.push_inst("call $alloc");
                f.push_inst("local.set $new_str");
                f.push_inst("");
                f.push_inst(";; Store TYPE_STRING at offset 0");
                f.push_inst("local.get $new_str");
                f.push_inst("global.get $TYPE_STRING");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Store combined length at offset 4");
                f.push_inst("local.get $new_str");
                f.push_inst("local.get $len1");
                f.push_inst("local.get $len2");
                f.push_inst("i32.add");
                f.push_inst("i32.store offset=4");
                f.push_inst("");
                f.push_inst(";; Copy str1 bytes");
                f.push_inst("(block $done1");
                f.push_inst("   (loop $loop1");
                f.push_inst("      local.get $i");
                f.push_inst("      local.get $len1");
                f.push_inst("      i32.ge_u");
                f.push_inst("      br_if $done1");
                f.push_inst("      local.get $new_str");
                f.push_inst("      i32.const 8");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.add");
                f.push_inst("      i32.add");
                f.push_inst("      local.get $str1");
                f.push_inst("      i32.const 8");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.add");
                f.push_inst("      i32.add");
                f.push_inst("      i32.load8_u");
                f.push_inst("      i32.store8");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.const 1");
                f.push_inst("      i32.add");
                f.push_inst("      local.set $i");
                f.push_inst("      br $loop1");
                f.push_inst("   )");
                f.push_inst(")");
                f.push_inst("");
                f.push_inst(";; Reset i and copy str2 bytes");
                f.push_inst("i32.const 0");
                f.push_inst("local.set $i");
                f.push_inst("(block $done2");
                f.push_inst("   (loop $loop2");
                f.push_inst("      local.get $i");
                f.push_inst("      local.get $len2");
                f.push_inst("      i32.ge_u");
                f.push_inst("      br_if $done2");
                f.push_inst("      local.get $new_str");
                f.push_inst("      i32.const 8");
                f.push_inst("      local.get $len1");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.add");
                f.push_inst("      i32.add");
                f.push_inst("      i32.add");
                f.push_inst("      local.get $str2");
                f.push_inst("      i32.const 8");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.add");
                f.push_inst("      i32.add");
                f.push_inst("      i32.load8_u");
                f.push_inst("      i32.store8");
                f.push_inst("      local.get $i");
                f.push_inst("      i32.const 1");
                f.push_inst("      i32.add");
                f.push_inst("      local.set $i");
                f.push_inst("      br $loop2");
                f.push_inst("   )");
                f.push_inst(")");
                f.push_inst("");
                f.push_inst("local.get $new_str");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // ARRAY HELPERS
    // ========================================================================
    // Arrays are heap objects with the same 8-byte header pattern as strings.
    //
    // Array layout on the heap:
    //   Offset 0: TYPE_ARRAY = 1    (i32)
    //   Offset 4: element_count     (i32)
    //   Offset 8: elem[0]           (tagged i32)
    //   Offset 12: elem[1]          (tagged i32)
    //   ...
    //
    // Arrays store tagged values (immediates or pointers) and are fixed-size
    // once allocated.

    pub fn generate_array_helpers(&mut self) {
        self.emit_comment("Array Helpers");

        // Allocate an empty array with capacity for `$count` elements.
        // All element slots are zero-initialized by the allocator.
        self.func("$create_array_empty")
            .param("$count", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Allocate array: TYPE_ARRAY, size=count*4");
                f.push_inst("global.get $TYPE_ARRAY");
                f.push_inst("local.get $count");
                f.push_inst("i32.const 4");
                f.push_inst("i32.mul");
                f.push_inst("call $heap_alloc");
            })
            .build();

        // Get element at index from array.
        // Takes a tagged index, returns the tagged value at that index.
        // TODO: Add bounds checking (trap if index >= array.length)
        self.func("$array_get")
            .param("$arr_ptr", "i32")
            .param("$idx_tagged", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Calculate address: arr_ptr + 8 + (idx * 4)");
                f.push_inst("local.get $arr_ptr");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $idx_tagged");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.const 4");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("i32.add");
                f.push_inst("");
                f.push_inst(";; Load tagged value");
                f.push_inst("i32.load");
            })
            .build();

        // Set element at index in array.
        // Takes a tagged index and a tagged value to store.
        // Returns the array pointer for chaining.
        // TODO: Add bounds checking (trap if index >= array.length)
        self.func("$array_set")
            .param("$arr_ptr", "i32")
            .param("$idx_tagged", "i32")
            .param("$val", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst(";; Calculate address: arr_ptr + 8 + (idx * 4)");
                f.push_inst("local.get $arr_ptr");
                f.push_inst("i32.const 8");
                f.push_inst("local.get $idx_tagged");
                f.push_inst("call $untag_immediate");
                f.push_inst("i32.const 4");
                f.push_inst("i32.mul");
                f.push_inst("i32.add");
                f.push_inst("i32.add");
                f.push_inst("");
                f.push_inst(";; Store tagged value");
                f.push_inst("local.get $val");
                f.push_inst("i32.store");
                f.push_inst("");
                f.push_inst(";; Return arr_ptr for chaining");
                f.push_inst("local.get $arr_ptr");
            })
            .build();

        self.emit_newline();
    }

    // ========================================================================
    // HELPER METHODS FOR WAT EMISSION
    // ========================================================================

    fn indent(&self) -> String {
        INDENT.repeat(self.indent_level)
    }

    /// Emit a line with proper indentation
    ///
    /// Adds the current indentation level and a newline.
    pub fn emit_line(&mut self, line: &str) {
        writeln!(self.output, "{}{}", self.indent(), line).unwrap();
    }

    /// Emit a comment line
    pub fn emit_comment(&mut self, comment: &str) {
        writeln!(self.output, "{};; {}", self.indent(), comment).unwrap();
    }

    /// Emit a blank line
    pub fn emit_newline(&mut self) {
        writeln!(self.output).unwrap();
    }

    // Example helper for emitting a complete function
    #[allow(dead_code)]
    fn emit_function(&mut self, name: &str, params: &str, result: &str, body: &[&str]) {
        self.emit_line(&format!("(func {} {} {}", name, params, result));
        self.indent_level += 1;

        for line in body {
            self.emit_line(line);
        }

        self.indent_level -= 1;
        self.emit_line(")");
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function_no_params() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$get_constant")
            .result("i32")
            .inst("i32.const 42")
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("(func $get_constant (result i32)"));
        assert!(wat.contains("   i32.const 42"));
    }

    #[test]
    fn test_function_with_single_param() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$tag_int")
            .param("$val", "i32")
            .result("i32")
            .inst("local.get $val")
            .inst("i32.const 2")
            .inst("i32.shl")
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("(func $tag_int (param $val i32) (result i32)"));
        assert!(wat.contains("i32.shl"));
    }

    #[test]
    fn test_emit_body_shorthand() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$untag_int")
            .param("$tagged", "i32")
            .result("i32")
            .emit_body(&["local.get $tagged", "i32.const 2", "i32.shr_s"])
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("local.get $tagged"));
        assert!(wat.contains("i32.shr_s"));
    }

    #[test]
    fn test_closure_based_body() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$closure_test")
            .param("$x", "i32")
            .result("i32")
            .body(|f| {
                f.push_inst("local.get $x");
                f.push_inst("i32.const 1");
                f.push_inst("i32.add");
            })
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("i32.add"));
    }

    #[test]
    fn test_multiple_params() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$add")
            .params(&[
                ("$a".to_string(), "i32".to_string()),
                ("$b".to_string(), "i32".to_string()),
            ])
            .result("i32")
            .emit_body(&["local.get $a", "local.get $b", "i32.add"])
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("(param $a i32) (param $b i32)"));
    }

    #[test]
    fn test_with_local_variables() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$with_locals")
            .param("$a", "i32")
            .local("$temp", "i32")
            .result("i32")
            .inst("local.get $a")
            .inst("local.set $temp")
            .inst("local.get $temp")
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("(local $temp i32)"));
        assert!(wat.contains("local.set $temp"));
    }

    #[test]
    fn test_realistic_tag_int() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$tag_int")
            .param("$val", "i32")
            .result("i32")
            .emit_body(&["local.get $val", "i32.const 2", "i32.shl"])
            .build();

        let expected = "(func $tag_int (param $val i32) (result i32)\n   local.get $val\n   i32.const 2\n   i32.shl\n)\n";
        assert_eq!(runtime.get_output(), expected);
    }

    #[test]
    fn test_multiple_functions() {
        let mut runtime = WasmRuntime::new();

        runtime
            .func("$tag_int")
            .param("$val", "i32")
            .result("i32")
            .emit_body(&["local.get $val", "i32.const 2", "i32.shl"])
            .build();

        runtime
            .func("$untag_int")
            .param("$tagged", "i32")
            .result("i32")
            .emit_body(&["local.get $tagged", "i32.const 2", "i32.shr_s"])
            .build();

        let wat = runtime.get_output();
        assert!(wat.contains("func $tag_int"));
        assert!(wat.contains("func $untag_int"));
    }

    #[test]
    fn test_indentation() {
        let mut runtime = WasmRuntime::new();
        runtime
            .func("$test")
            .param("$val", "i32")
            .local("$temp", "i32")
            .result("i32")
            .inst("local.get $val")
            .build();

        let lines: Vec<&str> = runtime.get_output().lines().collect();
        assert!(lines[0].starts_with("(func"));
        assert!(lines[1].starts_with("   ")); // Indented
    }

    #[test]
    fn test_indent_tracking() {
        let mut runtime = WasmRuntime::new();
        assert_eq!(runtime.get_indent_level(), 0);

        runtime.increment_indent();
        assert_eq!(runtime.get_indent_level(), 1);

        runtime.decrement_indent();
        assert_eq!(runtime.get_indent_level(), 0);
    }

    #[test]
    fn test_base_indent_captured() {
        let mut runtime = WasmRuntime::new();

        // Start at indent level 0, create a function builder
        let builder = runtime
            .func("$test")
            .param("$val", "i32")
            .result("i32")
            .inst("local.get $val");

        // Manually change the runtime's indent level after creating builder
        // (This simulates what would happen if we created the builder earlier
        // and the runtime's indent changed in the meantime)
        builder.runtime.increment_indent();
        builder.runtime.increment_indent(); // Now at indent 2

        // Build should still emit at base_indent (0), not current indent (2)
        builder.build();

        let wat = runtime.get_output();
        let lines: Vec<&str> = wat.lines().collect();

        // Function header should have NO indentation (base_indent was 0)
        assert!(lines[0].starts_with("(func"));
        assert!(!lines[0].starts_with(" "));

        // Body should have 1 level of indentation (base_indent + 1 = 0 + 1 = 1)
        assert!(lines[1].starts_with("   "));
        assert!(!lines[1].starts_with("      ")); // Not 2 levels
    }

    #[test]
    fn test_nested_indent_levels() {
        let mut runtime = WasmRuntime::new();

        // Create a function at indent level 1
        runtime.increment_indent();
        runtime
            .func("$inner")
            .param("$x", "i32")
            .result("i32")
            .inst("local.get $x")
            .build();

        let wat = runtime.get_output();
        let lines: Vec<&str> = wat.lines().collect();

        // Function header should be at indent level 1 (3 spaces)
        assert!(lines[0].starts_with("   (func"));

        // Body should be at indent level 2 (6 spaces)
        assert!(lines[1].starts_with("      "));

        // Closing paren should be back at indent level 1
        assert!(lines[2].starts_with("   )"));
        assert!(!lines[2].starts_with("      "));
    }

    // -----------------------------------------------------------------------
    // Environment helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_env_helpers_emits_all_three_functions() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_env_helpers();
        let wat = runtime.get_output();
        assert!(wat.contains("func $create_env"), "missing $create_env");
        assert!(wat.contains("func $env_get"), "missing $env_get");
        assert!(wat.contains("func $env_set"), "missing $env_set");
    }

    #[test]
    fn test_create_env_signature() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_env_helpers();
        let wat = runtime.get_output();
        // Must take parent_ptr and var_count, must return i32
        assert!(wat.contains("(param $parent_ptr i32)"));
        assert!(wat.contains("(param $var_count i32)"));
        // Allocates 8 + var_count * 4
        assert!(wat.contains("i32.const 8"));
        assert!(wat.contains("i32.const 4"));
        assert!(wat.contains("i32.mul"));
        assert!(wat.contains("call $alloc"));
    }

    #[test]
    fn test_env_get_uses_correct_offset() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_env_helpers();
        let wat = runtime.get_output();
        // env_get address calculation: env_ptr + 8 + index * 4
        assert!(wat.contains("func $env_get"));
        assert!(wat.contains("i32.load"));
    }

    #[test]
    fn test_env_set_uses_correct_offset() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_env_helpers();
        let wat = runtime.get_output();
        assert!(wat.contains("func $env_set"));
        assert!(wat.contains("i32.store"));
    }

    // -----------------------------------------------------------------------
    // Argument struct helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_arg_helpers_emits_all_four_functions() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_arg_helpers();
        let wat = runtime.get_output();
        assert!(wat.contains("func $create_arg"), "missing $create_arg");
        assert!(wat.contains("func $arg_set"), "missing $arg_set");
        assert!(wat.contains("func $arg_get"), "missing $arg_get");
        assert!(wat.contains("func $arg_length"), "missing $arg_length");
    }

    #[test]
    fn test_create_arg_allocates_correctly() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_arg_helpers();
        let wat = runtime.get_output();
        // Size = 4 (header) + length * 4
        assert!(wat.contains("i32.const 4"));
        assert!(wat.contains("call $alloc"));
    }

    #[test]
    fn test_arg_set_returns_arg_struct_ptr() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_arg_helpers();
        let wat = runtime.get_output();
        // $arg_set must have result i32 (for chaining) and return arg_struct_ptr
        let pos = wat.find("func $arg_set").unwrap();
        let end = wat.find("func $arg_get").unwrap_or(wat.len());
        let after = &wat[pos..end];
        assert!(after.contains("(result i32)"));
        assert!(after.contains("local.get $arg_struct_ptr"));
    }

    #[test]
    fn test_arg_length_reads_offset_zero() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_arg_helpers();
        let wat = runtime.get_output();
        // $arg_length just does i32.load from the raw pointer (offset 0)
        let pos = wat.find("func $arg_length").unwrap();
        let after = &wat[pos..];
        assert!(after.contains("i32.load"));
    }

    // -----------------------------------------------------------------------
    // Closure helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_closure_helpers_emits_both_functions() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_closure_helpers();
        let wat = runtime.get_output();
        assert!(
            wat.contains("func $create_closure"),
            "missing $create_closure"
        );
        assert!(wat.contains("func $call_closure"), "missing $call_closure");
    }

    #[test]
    fn test_call_closure_reads_func_idx_and_env() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_closure_helpers();
        let wat = runtime.get_output();
        let pos = wat.find("func $call_closure").unwrap();
        let after = &wat[pos..pos + 400];
        assert!(after.contains("i32.load offset=4"), "func_idx at offset 4");
        assert!(after.contains("i32.load offset=8"), "env_ptr at offset 8");
        assert!(
            after.contains("call_indirect"),
            "must dispatch via call_indirect"
        );
    }

    // -----------------------------------------------------------------------
    // String helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_string_helpers_emits_both_functions() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_string_helpers();
        let wat = runtime.get_output();
        assert!(
            wat.contains("func $create_string"),
            "missing $create_string"
        );
        assert!(
            wat.contains("func $string_concat"),
            "missing $string_concat"
        );
    }

    #[test]
    fn test_string_concat_copies_both_strings() {
        let mut runtime = WasmRuntime::new();
        runtime.generate_string_helpers();
        let wat = runtime.get_output();
        let pos = wat.find("func $string_concat").unwrap();
        let after = &wat[pos..];
        // Reads both lengths from offset 4
        assert!(after.contains("i32.load offset=4"));
        // Two copy loops
        assert!(after.contains("$loop1"));
        assert!(after.contains("$loop2"));
        // Resets $i between the two loops
        assert!(after.contains("i32.const 0"));
    }

    // -----------------------------------------------------------------------
    // generate_all includes new generators
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_all_includes_new_helpers() {
        let mut runtime = WasmRuntime::new();
        let wat = runtime.generate_all();
        assert!(wat.contains("func $create_env"));
        assert!(wat.contains("func $env_get"));
        assert!(wat.contains("func $env_set"));
        assert!(wat.contains("func $create_arg"));
        assert!(wat.contains("func $arg_set"));
        assert!(wat.contains("func $arg_get"));
        assert!(wat.contains("func $arg_length"));
        assert!(wat.contains("func $create_closure"));
        assert!(wat.contains("func $call_closure"));
        assert!(wat.contains("func $create_string"));
        assert!(wat.contains("func $string_concat"));
    }
}

// ============================================================================
// IMPLEMENTATION CHECKLIST
// ============================================================================
//
// PHASE 1: Tag Helpers (Start Here)
// [ ] 1. Implement generate_tag_helpers()
//        - [ ] $tag_int
//        - [ ] $untag_int
//        - [ ] $tag_bool
//        - [ ] $untag_bool
//        - [ ] $tag_pointer
//        - [ ] $untag_pointer
//        - [ ] $get_tag
//
// PHASE 2: Type Checking
// [ ] 2. Implement generate_type_check_helpers()
//        - [ ] $is_int
//        - [ ] $is_bool
//        - [ ] $is_pointer
//
// PHASE 3: Arithmetic
// [ ] 3. Implement generate_arithmetic_helpers()
//        - [ ] $add_values
//        - [ ] $sub_values
//        - [ ] $mul_values
//        - [ ] $div_values
//
// PHASE 4: Comparisons
// [ ] 4. Implement generate_comparison_helpers()
//        - [ ] $lt_values
//        - [ ] $gt_values
//        - [ ] $eq_values
//        - [ ] $ne_values
//
// PHASE 5: Heap (Later)
// [ ] 5. Implement generate_heap_alloc()
//        - [ ] $heap_alloc
//
// HOW TO USE:
// In wasm_compiler.rs, do this:
//
//   use crate::wasm_environment::WasmRuntime;
//
//   fn emit_runtime_helpers(&mut self) {
//       let mut runtime = WasmRuntime::new();
//       let helpers_wat = runtime.generate_all();
//       self.output.push_str(&helpers_wat);
//   }
//
