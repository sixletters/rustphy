---
layout: default
title: Part 6 - Compiling to WebAssembly
---

# Part 6: Compiling to WebAssembly

**Building Rustphy: A Compiler From Scratch**

---

## The Second Backend Is Not a Port

When I decided to target WebAssembly, my first instinct was to think of it as "another instruction set." Emit WAT instead of bytecode, map the opcodes, done.

That instinct lasted about a day.

WebAssembly is not a general-purpose bytecode with different names. It is a stack machine with **typed locals, structured control flow, linear memory, and no heap, no objects, no closures, and no garbage collector**. Everything the bytecode VM got for free from Rust — hashmaps, `Rc`, enums, recursive environments — has to be *built by hand* in the WASM backend.

That turned out to be the most educational decision in the whole project. The WASM backend is where I stopped writing "a compiler" and started writing a **runtime system**.

---

## The Output: WAT, Not Binary

Rustphy's WASM compiler emits **WebAssembly Text format** — human-readable S-expressions:

```wat
(module
  (import "env" "print" (func $print (param i32)))
  (memory $heap (export "memory") 1)
  ;; ...
  (func $main (export "main")
    ;; compiled program
  )
)
```

This was a deliberate choice. WAT is:

- Readable — I can inspect exactly what the compiler produced
- Debuggable — `wat2wasm` and `wasm-tools print` validate and format it
- Testable — I can diff outputs across compiler changes
- Teachable — you are reading it right now

The cost is speed: emitting strings is slower than emitting bytes, and the text must be parsed again by a WASM toolchain. For a learning compiler, inspectability wins.

The build loop is:

```text
Rustphy source → Parser → AST → wasm_compiler.rs → WAT text
                                                    ↓
                                              wat2wasm
                                                    ↓
                                              WASM binary
                                                    ↓
                                              browser/runtime
```

---

## The Fundamental Problem: Everything Is an i32

WebAssembly has exactly four primitive types: `i32`, `i64`, `f32`, `f64`.

Rustphy has integers, floats, booleans, strings, arrays, hashmaps, closures, and `unassigned`.

How do we fit a dynamic language into a statically typed instruction set?

### The Answer: Tagged Values

Rustphy uses a **1-bit tag scheme** ("nan-boxing lite"). Every Rustphy value is represented as a single `i32`:

```text
If (value & 1) == 1:  it's an immediate integer
                      actual value = value >> 1

If (value & 1) == 0:  it's a heap pointer
                      pointer address = value
```

Why does this work? Heap allocations from our bump allocator are always **even-aligned** (they start on even addresses). So an even `i32` is never confused with an odd tagged integer.

Tagging an integer:

```wat
;; tag 42 → (42 << 1) | 1 = 85
i32.const 42
call $tag_immediate
```

Untagging:

```wat
;; untag 85 → 85 >> 1 = 42
local.get $val
call $untag_immediate
```

The runtime helpers are tiny:

```wat
(func $tag_immediate (param $val i32) (result i32)
  local.get $val
  i32.const 1
  i32.shl          ;; val << 1
  i32.const 1
  i32.or           ;; | 1
)
```

### What Lives on the Heap?

Anything that isn't an integer:

| Rustphy type | Heap tag | Layout |
|---|---|---|
| String | `$TYPE_STRING = 0` | `[tag][size][byte0][byte1]...` |
| Array | `$TYPE_ARRAY = 1` | `[tag][size][elem0][elem1]...` |
| Closure | `$TYPE_CLOSURE = 2` | `[tag][func_idx][env_ptr]` |
| Object | `$TYPE_OBJECT = 3` | `[tag][size][key0][val0]...` |

Every heap object starts with an 8-byte header: a 4-byte type tag and a 4-byte size. The type tag lets runtime helpers distinguish strings from arrays from closures without static type information.

### What About Floats?

This is where I have to be honest. The WASM backend's float support is a stub:

```rust
ExpressionNode::Float { .. } => {
    Ok(String::from("hold on"))
}
```

The literal string `"hold on"` gets emitted into the WAT output. It is not valid WASM. It is a placeholder I wrote when float support landed in the *bytecode* backend, and the WASM backend hasn't caught up yet.

A real implementation would need to decide: do floats get their own tag bit pattern? Do they get boxed on the heap? Or does the whole value representation switch to NaN-boxing on `f64`? Each choice changes every arithmetic and comparison helper.

---

## The Memory Model: One Page and a Bump Pointer

WASM linear memory is a flat, growable array of bytes. Rustphy allocates **one 64KiB page**:

```wat
(memory $heap (export "memory") 1)
(global $heap_ptr (mut i32) (i32.const 1024))
```

The bump pointer starts at offset 1024. The first 1KiB is reserved for string data segments (more on that below).

Allocation is trivially fast:

```wat
(func $alloc (param $size i32) (result i32)
  (local $old_ptr i32)
  global.get $heap_ptr
  local.set $old_ptr
  global.get $heap_ptr
  local.get $size
  i32.add
  global.set $heap_ptr
  local.get $old_ptr    ;; return previous pointer
)
```

That's it. No free list. No compaction. No garbage collection. Memory grows until the page fills up, and then the program crashes.

Is this a good allocator? No. Is it a **correct first allocator** that lets every other feature proceed? Absolutely. You can write a generational GC later; you can't learn closure layout without *some* allocator first.

---

## Strings: Compile-Time Data, Runtime Objects

String literals get special treatment because their bytes are known at compile time.

### Compile-Time: Collect and Deduplicate

The compiler collects every unique string literal and assigns it a byte offset in the data section:

```rust
if let Some(index) = self.string_compilation_context
    .collected_string_literals.get(value)
{
    offset = index.clone();
} else {
    self.string_compilation_context
        .collected_string_literals
        .insert(value.clone(), self.string_compilation_context.next_data_offset);
    self.string_compilation_context.next_data_offset += value.len();
    // emit: (data (i32.const <offset>) "hello")
}
```

The same literal `"hello"` appearing five times gets one data segment. The offsets are packed sequentially starting at address 0.

### Runtime: Copy Into a Heap Object

When the program evaluates a string literal, generated code calls `$create_string` with the data offset and length:

```wat
i32.const 0       ;; data offset
i32.const 5       ;; length
call $create_string
```

The runtime helper allocates a heap object, then copies bytes one at a time from linear memory into the heap string. It returns a tagged pointer.

Why copy at all? Because heap strings are mutable objects with headers, and the data section is read-only compile-time storage. The copy gives us a uniform representation: all strings are heap objects regardless of origin.

### The Limitation

There is no escaping in the data segments — a string containing `"` will produce malformed WAT. And string concatenation via `$add_values` has a fast path for tagged integers but returns `0` for heap operands:

```wat
;; in $add_values, if either operand is a heap pointer:
i32.const 0    ;; "heap object addition not yet supported"
```

So `"hello" + " world"` works on the bytecode VM but not yet on WASM. The playground demo uses string concatenation — which is why it only fully works through the bytecode path.

---

## Functions: The Dual-Compilation Trick

This is the most interesting design decision in the WASM backend.

Rustphy functions are first-class values. They can be called directly:

```javascript
add(2, 3);
```

or stored, passed, and called indirectly:

```javascript
let f = add;
f(2, 3);
```

WASM has two call mechanisms:

- `call $name` — direct call, compile-time known target
- `call_indirect` — indirect call through a function table

We want the fast path when the target is known and the general path when it isn't. So **every function compiles twice**.

### Direct Entry Point

```wat
(func $add_direct (param $env_ptr i32) (param $a i32) (param $b i32) (result i32)
  ;; body uses a and b directly as locals
)
```

The first parameter is always the environment pointer. User parameters follow. When the compiler sees a call to a known function declaration, it uses this:

```wat
local.get $current_env
i32.const 2        ;; tagged
i32.const 3        ;; tagged
call $add_direct
```

### Closure Trampoline

Every function also gets a **uniform-signature wrapper**:

```wat
(func $add_closure (param $env_ptr i32) (param $arg_struct_ptr i32) (result i32)
  local.get $arg_struct_ptr
  i32.const 0
  call $arg_get        ;; extract arg 0
  local.get $arg_struct_ptr
  i32.const 1
  call $arg_get        ;; extract arg 1
  local.get $env_ptr
  ;; swap order and call direct
  call $add_direct
)
```

This trampoline has a fixed signature: `(env_ptr, arg_struct_ptr) → i32`. Every closure trampoline in the program has the **same** signature. That means they can all live in one WASM function table:

```wat
(type $function_type (func (param i32 i32) (result i32)))
(table $closures 5 funcref)
(elem (table $closures) (i32.const 0) func $add_closure $make_counter_closure ...)
```

### The Closure Object

When code creates a closure value, it allocates a 12-byte heap object:

```text
[type_tag = 2][func_idx][env_ptr]
```

To call it:

```wat
;; $call_closure(closure_ptr, arg_struct_ptr)
local.get $closure_ptr
i32.const 4
i32.add
i32.load           ;; load func_idx

local.get $closure_ptr
i32.const 8
i32.add
i32.load           ;; load env_ptr

local.get $arg_struct_ptr

call_indirect (type $function_type)
```

The function table provides the code. The closure object provides the environment. Together they reconstitute a Rustphy closure call.

### The Hardcoded Limit

```wat
(table $closures 5 funcref)
```

Five. If your program has more than five functions, the table overflows. This is a known limitation. The fix is to count functions at compile time and emit the correct table size. I haven't done it yet because my test programs are small. It will absolutely bite someone.

---

## Control Flow: No GOTO

WASM has no `goto`. Instead, it has **structured control flow**: `block`, `loop`, `if`, and `br`.

### If/Else

```javascript
if (x > 5) { "big" } else { "small" }
```

becomes:

```wat
;; compile condition
local.get $x
call $untag_immediate
i32.const 5
i32.gt_s

(if (result i32)
  (then
    ;; compile "big"
    i32.const <tagged_string_big>
  )
  (else
    ;; compile "small"
    i32.const <tagged_string_small>
  )
)
```

The `(result i32)` annotation means the `if` produces a value — this is how Rustphy's `if` expressions work on WASM.

### Loops

A `for` loop maps to nested `block` and `loop`:

```javascript
for (i in 0..3) { print(i); }
```

becomes:

```wat
(block $break_0
  (loop $continue_0
    ;; condition: i < 3
    local.get $i
    i32.const 3
    call $lt_values
    call $untag_immediate
    i32.eqz
    br_if $break_0       ;; exit if condition is false

    ;; body
    local.get $i
    call $print

    ;; increment
    local.get $i
    i32.const 1
    call $add_values
    local.set $i

    br $continue_0       ;; jump back to condition
  )
)
```

- `br $break_N` exits the block (equivalent to `break`)
- `br $continue_N` jumps to the loop header (equivalent to `continue`)

The `loop_counter` provides unique labels. Nested loops get `$break_0`/`$continue_0` inside `$break_1`/`$continue_1`, and `br` can target any enclosing label.

This is fundamentally different from the bytecode VM's `GOTO addr` model. The compiler must map source-level control flow onto WASM's structured blocks — and it does so correctly, even though the source language has `break` and `continue`.

---

## Short-Circuit Logic: WASM Does It Right

The bytecode VM evaluates both sides of `&&` and `||` eagerly. The WASM backend compiles them properly:

```javascript
a && b
```

becomes:

```wat
(if (result i32)
  (then
    ;; evaluate a, untag
    local.get $a
    call $untag_immediate
    (if (result i32)
      (then
        ;; a is truthy, evaluate b
        local.get $b
        call $untag_immediate
      )
      (else
        i32.const 0    ;; a was false, short-circuit
      )
    )
  )
)
```

If `a` is false, `b` is never evaluated. This is the correct semantics.

The bytecode VM does not do this. Both backends compile the same language but have different observable behaviour for side-effecting logical expressions. That is a bug — and a great lesson in why backend parity matters.

---

## Objects: Linear Search Hashmaps

Rustphy's hashmap-like objects compile to a simple runtime:

```wat
;; Object layout: [type_tag][size][key0][val0][key1][val1]...
```

Lookup is linear search:

```wat
(func $object_find_index (param $obj_ptr i32) (param $key_ptr i32) (result i32)
  ;; iterate through key slots, compare with $string_equals
  ;; return index or -1
)
```

Growth doubles the capacity and copies entries. This is the simplest correct implementation — the same "linear scan" strategy you'd use for a hashmap's first prototype before adding hashing.

For a compiler course, this is a feature: the object system is small enough to read in one sitting, and upgrading to a real hash table is a self-contained exercise.

---

## The Runtime Generator: `wasm_environment.rs`

The 2,343-line `wasm_environment.rs` is not a runtime written in WAT. It is a **WAT code generator** written in Rust. It contains:

- `WasmRuntime`: a string builder with indentation tracking
- `FunctionBuilder`: a fluent API for emitting WASM functions
- ~45 helper function generators, called in dependency order

```rust
runtime.func("$tag_immediate")
    .param("$val", "i32")
    .result("i32")
    .body(|f| {
        f.push_inst("local.get $val");
        f.push_inst("i32.const 1");
        f.push_inst("i32.shl");
        f.push_inst("i32.const 1");
        f.push_inst("i32.or");
    })
    .build();
```

This means the runtime is **generated**, not handwritten. If I change the heap object layout, I change it in one Rust function, and every generated helper reflects the new layout.

The tradeoff is that the generated WAT is verbose and harder to read than hand-optimized WAT. But correctness-by-construction beats hand-maintained consistency.

---

## The Browser Playground

The frontend is a single HTML page with a code editor, a compile button, and an output panel. It loads the compiler as a WASM module through `wasm-bindgen`:

```javascript
import init, { compile_to_wasm, run_rustphy_code } from './pkg/rustphy.js';

// Compile Rustphy source to WAT text
const wat = compile_to_wasm(sourceCode);

// Run through the bytecode VM and capture print output
const output = run_rustphy_code(sourceCode);
```

The compiler itself runs inside the browser as WASM. The `run_rustphy_code` path executes through the bytecode VM (compiled to WASM by Rust's own toolchain). The `compile_to_wasm` path shows the generated WAT text.

There is a pleasing recursion here: **Rust code compiled to WASM, running in a browser, compiling Rustphy source to WAT text**. The playground never instantiates the generated WAT as a WASM module in the browser — that would require a `wat2wasm` step client-side — but displaying the output is enough for learning.

---

## What the WASM Backend Taught Me

1. **Value representation is a global decision.** Every helper, every compiled expression, every runtime function must agree on the tagging scheme. Change it in one place, and everything breaks silently.

2. **Closures are where compilers get real.** The direct/closure dual-compilation, the function table, the argument struct, the environment chain — all of these exist because one language feature demands runtime flexibility that the target doesn't provide natively.

3. **The runtime is code too.** Forty-five helper functions for tagging, allocation, strings, arrays, objects, environments, and closures. The compiler emits them all. Without them, the generated code does nothing.

4. **Parity between backends is a feature, not a given.** Floats work on one backend. Short-circuit works on the other. String concat works on one. The language is defined by what both backends agree on — which is less than either supports alone.

5. **Start with the simplest correct thing.** A bump allocator, a linear-search object, a fixed-size closure table. These are all "wrong" in some way. But they are *correct enough* to let the next feature land. Perfect is the enemy of shipped.

---

## Checkpoint

At this point you should be able to trace this Rustphy program:

```javascript
let x = 5;
let f = func(y) { return x + y; };
print(f(3));
```

through the WASM backend:

1. `5` is tagged to `11` (`(5 << 1) | 1`)
2. `x` is stored in a local (it doesn't escape) or an environment slot (it does escape — `f` captures it)
3. `f` compiles to `$f_direct` and `$f_closure`
4. `f(3)` goes through `$call_closure`, which reads `func_idx` and `env_ptr` from the closure object
5. `$f_closure` unpacks the argument struct, passes the captured environment, and calls `$f_direct`
6. `$f_direct` loads `x` from the environment (it's escaping), untags both values, adds them, re-tags the result
7. `$print` receives the tagged result, untags it, and calls the host import

If you can follow that, you understand the WASM backend.

---

## Quiz

1. Why does the tagging scheme work? What property of heap pointers makes it safe?
2. Why does every function compile to two WASM functions?
3. What information does a closure heap object store?
4. How does WASM's structured control flow differ from the bytecode VM's jumps?
5. Why does the WASM backend short-circuit `&&` but the bytecode VM doesn't?
6. What happens when the program allocates more than 64KiB of heap memory?

<details>
<summary><strong>Answers</strong></summary>

1. Heap pointers are even-aligned, so the least significant bit is always 0. Tagged integers set the LSB to 1. The two representations never collide.
2. A direct entry point for compile-time-known calls (fast, typed params) and a uniform-signature closure trampoline for indirect calls through the function table.
3. A type tag, a function table index, and a pointer to the captured environment.
4. WASM uses `block`/`loop`/`br` with labels instead of arbitrary `goto`. Break and continue map to branch instructions targeting enclosing blocks.
5. They were implemented at different times with different care. The WASM backend compiles `&&`/`||` as `if` expressions; the bytecode backend compiles them as eager binary operations. This is a semantic bug.
6. The bump allocator runs out of space. There is no growth, no GC, and no error handling. The program would produce undefined behaviour or trap.

</details>

---

## Practice

### Challenge 1: Dynamic Closure Table

Count the number of function definitions at compile time and emit `(table $closures N funcref)` with the correct size instead of the hardcoded 5.

### Challenge 2: String Concatenation

Implement `$add_values` for two string operands: untag both, read their lengths, allocate a new heap string, copy both byte sequences, return the new tagged pointer.

### Challenge 3: Float Boxing

Choose a float representation (heap-boxed or NaN-boxed) and implement float arithmetic in the WASM backend. Update the tag helpers, arithmetic helpers, and comparison helpers.

### Challenge 4: Run Generated WASM

Add a `wat2wasm` step to the playground and use `WebAssembly.instantiate` to actually run the generated module, providing the `env.print` import from JavaScript.

---

[← Part 5 — Closures and Escape Analysis](part_05_closures_and_escape_analysis.md) | [Course index](index.md) | [Part 7 — What Building a Compiler Actually Taught Me →](part_07_lessons.md)
