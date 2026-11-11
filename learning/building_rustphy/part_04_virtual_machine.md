---
layout: default
title: Part 4 - Building the Virtual Machine
---

# Part 4: Building the Virtual Machine

**Building Rustphy: A Compiler From Scratch**

---

## We Invented a Machine. Now We Have to Build It.

Our bytecode says:

```text
LDCN 40
LDCN 2
BINOP Add
ASSIGN "answer"
POP
DONE
```

Those instructions have no universal meaning. We invented them, so we must build the machine that interprets them.

A virtual machine is a software CPU for our own instruction set. It has registers—although ours are Rust fields—stacks, memory, a program counter, and an execution loop.

Rustphy's machine is small enough to understand end to end, but capable enough to run closures, recursion, mutable arrays, hashmaps, builtins, and lexical scopes.

---

## Step 1: Define the Machine State

Rustphy's VM stores:

```rust
pub struct Machine {
    os: Vec<Value>,
    rts: Vec<StackFrame>,
    pc: usize,
    env: Rc<RefCell<Environment>>,
    is_done: bool,
    output: Vec<String>,
}
```

Each field has one job:

| Field | Role |
|---|---|
| `os` | Operand stack: values used by instructions |
| `rts` | Runtime stack: saved environments and return addresses |
| `pc` | Program counter: index of the next instruction |
| `env` | Current lexical environment |
| `is_done` | Halt flag |
| `output` | Captured output for CLI/browser integration |

Two stacks are important. Expression evaluation and function control flow are different concerns.

---

## The Operand Stack

The operand stack holds program values:

```text
LDCN 2       [2]
LDCN 3       [2, 3]
BINOP Add    [5]
```

It is temporary dataflow.

## The Runtime Stack

The runtime stack holds machine continuation state:

```rust
pub struct StackFrame {
    pc: usize,
    env: Rc<RefCell<Environment>>,
    is_call_frame: bool,
}
```

A frame remembers:

- Where execution should resume
- Which environment should become current again
- Whether this frame represents a function call or only a nested scope

It is control flow and scope restoration.

Combining both uses in one `StackFrame` type makes `RESET` capable of unwinding block scopes until it reaches the caller.

---

## Step 2: Represent Runtime Values

The VM needs one type capable of holding every language value:

```rust
pub enum Value {
    Number(i128),
    Float(f64),
    Bool(bool),
    String(String),
    Identifier(String),
    Symbol(String),
    Closure {
        params: Vec<String>,
        addr: usize,
        env: Rc<RefCell<Environment>>,
    },
    Builtin { name: BuiltinFn },
    Unassigned,
    Array(Rc<RefCell<Vec<Value>>>),
    HashMap(Rc<RefCell<HashMap<String, Value>>>),
}
```

This enum is the dynamic type system at runtime.

### Why `Rc<RefCell<...>>` for Arrays and Hashmaps?

Consider:

```javascript
let a = [1, 2];
let b = a;
push_arr(b, 3);
print(a);
```

If assignment copied the whole vector, mutating `b` would not affect `a`. Rustphy instead gives both values shared ownership of the same mutable object.

```text
Value::Array ──┐
               ├──→ Rc<RefCell<Vec<Value>>>
Value::Array ──┘
```

- `Rc` lets multiple values own the allocation.
- `RefCell` permits mutation through shared references with runtime borrow checks.

Closures use the same ownership pattern for environments.

### Why `Unassigned`?

When entering a block, declared names are preinstalled before their initializer executes. `Unassigned` distinguishes:

```text
Name exists, but has no value yet
```

from:

```text
Name does not exist in any scope
```

---

## Step 3: Build Lexical Environments

An environment is a hashmap plus an optional parent:

```rust
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}
```

Nested blocks form a chain:

```text
Current block: { y: 2 }
      │ parent
      ▼
Function:      { x: 1, param: 10 }
      │ parent
      ▼
Global:        { print: ..., make_counter: ... }
```

### Reading a Name

`get(name)` checks the current hashmap. If absent, it recursively checks the parent.

```text
lookup y → current scope → found
lookup x → current scope → parent → found
lookup z → current → parent → global → error
```

### Declaring a Name

`set_declare(name, value)` writes to the current environment. This creates shadowing:

```javascript
let x = 1;
{
    let x = 2;
    print(x); // 2
};
print(x);     // 1
```

### Assigning a Name

`set_assign(name, value)` searches upward for the environment that already owns the name, then mutates that environment.

This distinction is critical:

```text
declare → current scope
assign  → nearest existing binding
```

Without it, assigning a captured variable would accidentally create a new local instead of mutating the outer binding.

---

## Step 4: Execute Until Halt

The heart of the VM is tiny:

```rust
while !self.is_done {
    self.execute(&instructions[self.pc])?;
}
```

`execute` matches on the current instruction. Every arm is responsible for updating `pc`.

Most instructions do:

```rust
self.pc += 1;
```

Jumps assign a new value:

```rust
self.pc = addr;
```

Forgetting to update the program counter means an infinite loop. Updating it twice skips an instruction. This is the VM equivalent of the lexer's cursor discipline.

---

## Step 5: Execute Constants and Arithmetic

A constant instruction pushes a value:

```text
LDCN { val: 42 } → os.push(Value::Number(42))
```

A binary operation conceptually does:

```rust
let right = os.pop()?;
let left = os.pop()?;
let result = apply(operator, left, right)?;
os.push(result);
pc += 1;
```

Rustphy's arithmetic helpers support integer/float promotion:

```text
Number + Number → Number
Number + Float  → Float
Float  + Number → Float
Float  + Float  → Float
```

`+` also handles string concatenation in the bytecode VM.

Type mismatches, division by zero, modulo by zero, and stack underflow become runtime errors.

Error messages include the program counter, which immediately tells us which emitted instruction failed.

---

## Step 6: Define Truthiness

Control flow needs to interpret values as conditions.

Rustphy treats these as false:

```text
Bool(false)
Number(0)
Float(0.0)
String("")
```

Other supported scalar values are true; unsupported categories in boolean context can produce an error.

`JOF` pops its condition and jumps when it is falsey:

```rust
let condition = self.os.pop().ok_or("stack underflow")?;
if !is_truthy(&condition)? {
    self.pc = addr;
} else {
    self.pc += 1;
}
```

The bytecode compiler emits both sides of `&&` and `||` as ordinary operands, so this backend does not currently short-circuit. The WASM backend does. This is a semantic mismatch we will examine later.

---

## Step 7: Enter and Exit Scopes

`ENTERSCOPE`:

1. Pushes a non-call frame containing the current environment.
2. Creates a child environment.
3. Predeclares every supplied symbol as `Unassigned`.
4. Advances the program counter.

Conceptually:

```rust
rts.push(StackFrame {
    pc,
    env: current_env.clone(),
    is_call_frame: false,
});
current_env = Environment::extend(current_env);
for symbol in symbols {
    current_env.set_declare(symbol, Value::Unassigned);
}
```

`EXITSCOPE` pops the saved frame and restores its environment.

Why save `pc` on a scope frame if normal scope exit is sequential? Because the same frame type is shared with calls. The field is meaningful primarily for call frames.

---

## Step 8: Create a Closure

When the VM executes:

```text
LDF { addr, params }
```

it pushes:

```rust
Value::Closure {
    params,
    addr,
    env: self.env.clone(),
}
```

That last line is the entire lexical-capture mechanism.

A closure is:

```text
code address + parameter list + defining environment
```

Not the calling environment. Not a copy of every current value. A shared reference to the lexical environment active when the function expression was evaluated.

### Why Capture by Shared Reference?

Our counter example needs mutation to persist:

```javascript
let make_counter = func(start) {
    let value = start;
    return func() {
        value = value + 1;
        return value;
    };
};
```

If the inner function copied `value` on every call, it would always return `41`. Instead, every invocation reaches the same captured environment cell:

```text
closure ──→ environment { value: 40 }
call 1  mutates value → 41
call 2  mutates value → 42
```

---

## Step 9: Call a Closure

`CALL { arity }` pops arguments and the callable.

For a user closure, the VM:

1. Pushes a call frame with the caller's next PC and environment.
2. Extends the closure's **captured environment**.
3. Declares parameters in that child environment.
4. Sets `pc` to the function body address.

The key code is:

```rust
Value::Closure { params, addr, env } => {
    self.rts.push(StackFrame {
        pc: self.pc + 1,
        env: self.env.clone(),
        is_call_frame: true,
    });

    self.env = Environment::extend(env.clone());

    for (i, parameter) in params.iter().enumerate() {
        self.env
            .borrow_mut()
            .set_declare(parameter.clone(), args[i].clone());
    }

    self.pc = addr;
}
```

Notice this line:

```rust
self.env = Environment::extend(env.clone());
```

It extends the environment captured by the function, not the current caller environment.

That is **lexical scoping**.

If it extended the caller's environment, Rustphy would use dynamic scoping: a function's variable references would depend on who called it. That is almost never what programmers expect in modern languages.

---

## Step 10: Return With `RESET`

A function body can enter nested block scopes before returning.

The runtime stack might look like:

```text
[top] block scope inside if       is_call_frame = false
      function body scope         is_call_frame = false
      caller frame                is_call_frame = true
      outer block                 is_call_frame = false
```

`RESET` pops frames until it finds a call frame. It restores that frame's environment and return PC.

This design means an early return does not need to execute every pending `EXITSCOPE`. `RESET` unwinds them automatically.

The return value stays on the operand stack, ready for the caller.

This is an elegant use of one runtime stack to restore both lexical and call state.

### What Happens at Top Level?

A malformed `RESET` without a call frame must become a runtime error. This is one reason a semantic analysis pass should reject `return` outside a function before execution.

---

## Step 11: Execute Builtins

When `LDS` sees a builtin name, Rustphy can create a builtin callable:

```text
print
push_arr
len
```

`print`:

- Formats its argument
- Appends the text to `Machine.output`
- Uses `println!` natively
- Uses browser console logging under `wasm32`
- Returns the printed value

Capturing output is important for the browser API. `run_rustphy_code(source)` can return a string rather than requiring the host to intercept stdout.

`len` supports arrays, hashmaps, and strings. `push_arr` mutates its shared array object.

Builtins currently have special lookup behaviour rather than ordinary global bindings. A more uniform design could install builtin values in the global environment, allowing normal shadowing rules.

---

## Step 12: Index Shared Objects

`LDAI` loads from arrays or hashmaps. `STAI` stores and pushes the stored value back.

For arrays, runtime checks include:

- Index must be numeric
- Index must be non-negative and in range
- Container must be indexable

For hashmaps, keys are converted to strings in the current implementation. This is a semantic choice, not merely an implementation detail. If the language should distinguish numeric key `1` from string key `"1"`, the representation must change.

---

## Trace: A Closure Call From Start to Finish

Source:

```javascript
let x = 10;
let add_x = func(y) {
    return x + y;
};
print(add_x(5));
```

### Creating `add_x`

```text
Current env: global { x: 10 }
Execute LDF:
Closure {
    addr: function_body,
    params: ["y"],
    env: global_env
}
```

### Calling `add_x(5)`

```text
1. Pop argument 5
2. Pop closure
3. Save caller frame
4. Create call env whose parent is captured global_env
5. Bind y = 5
6. Jump to function_body
```

Environment chain inside call:

```text
call env { y: 5 }
   │
   ▼
global env { x: 10, add_x: closure }
```

### Evaluating `x + y`

```text
LDS x → not in call env → parent → 10
LDS y → call env → 5
BINOP Add → 15
```

### Returning

```text
RESET:
- pop block/scope frames if any
- find call frame
- restore caller PC and env
- leave 15 on operand stack
```

`print` receives `15`.

---

## Why Runtime Name Lookup Is Both Nice and Expensive

The bytecode VM's environment chain makes closures almost natural. A closure holds an environment, and every lookup walks parents.

Advantages:

- Compiler is simple.
- Shadowing follows environment structure.
- Captures need no dedicated free-variable list.
- Debugging environments is intuitive.

Costs:

- Every read hashes a string.
- Deep lexical nesting requires repeated parent traversal.
- Misspelled or unresolved names fail at runtime.
- Locals cannot easily map to fast indexed slots.
- Capturing the whole environment can retain more memory than needed.

The WebAssembly backend responds by moving more work to compile time. That is the subject of Part 5.

---

## Error Handling

Machine operations return `Result<_, String>`. Typical errors include:

- Operand stack underflow
- Runtime stack underflow
- Undefined variable
- Use of an unassigned binding
- Wrong operand type
- Division or modulo by zero
- Array index out of bounds
- Calling a non-callable value
- Wrong argument count
- Unsupported instruction

The PC is included in runtime errors:

```text
Runtime error at PC 17: division by zero
```

That is useful, but source spans are still missing. A better toolchain would retain a mapping:

```text
bytecode instruction range → source span
```

Then runtime errors could point back to the original program.

---

## The Incomplete Tail-Call Story

The instruction set includes `TAILCALL`, but the machine's dispatcher does not implement it.

A real tail call would:

- Avoid pushing a new call frame
- Rebind parameters or build the next call environment
- Jump to the callee
- Preserve the caller's caller as the eventual return target

This matters for recursive programs that would otherwise grow the runtime stack indefinitely.

The unfinished opcode is a good lesson: optimization touches calling convention and runtime state. It cannot be completed entirely in code generation.

---

## What I Would Improve Next

### 1. Split Scope and Call Frames

Separate enums can make invalid states unrepresentable:

```rust
pub enum RuntimeFrame {
    Scope { env: EnvRef },
    Call { return_pc: usize, env: EnvRef },
}
```

### 2. Resolve Locals to Numeric Slots

Keep named globals for debugging if desired, but compile local reads to indexed operations.

### 3. Normalize Builtins

Install builtins as ordinary global bindings. Then lexical shadowing and call behaviour are uniform.

### 4. Add Source Maps

Associate every emitted instruction with its originating AST node/span.

### 5. Define Equality and Hash Keys Precisely

Dynamic languages need explicit semantics for mixed numeric equality, object identity, and hashmap keys.

### 6. Implement Short-Circuit Bytecode

Compile `&&` and `||` as conditional control flow rather than eager binary operations.

---

## Checkpoint

We have completed one entire executable pipeline:

```text
Source
  ↓ Lexer
Tokens
  ↓ Parser
AST
  ↓ Bytecode compiler
Instructions
  ↓ Stack VM
Result
```

At this point, Rustphy can already teach us a great deal about language implementation.

But our next target is not a machine we fully control. WebAssembly has its own locals, memory, function signatures, and structured control flow. Before generating it, we need to answer a question the bytecode VM postponed until runtime:

> Which declaration does this identifier refer to, and where should its value live?

---

## Quiz

1. What is the difference between the operand stack and runtime stack?
2. Why do arrays use shared mutable storage?
3. What is the difference between declaration and assignment in an environment chain?
4. What three pieces make up a closure value?
5. Why must calls extend the captured environment rather than the caller's environment?
6. How can `RESET` return through nested block scopes?
7. What are the costs of resolving names at runtime?

<details>
<summary><strong>Answers</strong></summary>

1. The operand stack holds temporary program values; the runtime stack holds continuation and scope-restoration state.
2. Aliases should observe mutations to the same language object.
3. Declaration writes to the current scope; assignment mutates the nearest existing binding up the parent chain.
4. Code address, parameter list/calling information, and captured lexical environment.
5. Lexical scope depends on where a function was defined, not who called it.
6. It pops non-call scope frames until it reaches a call frame, then restores the caller.
7. Hashing strings, walking parent chains, runtime unresolved-name failures, and less efficient local access/capture.

</details>

---

## Practice

### Challenge 1: VM Trace Mode

Print `pc`, instruction, operand stack, and environment depth before every execution step. Use it to trace recursion.

### Challenge 2: Short-Circuit Logic

Change bytecode generation so `false && fail()` never calls `fail`, and `true || fail()` never calls it.

### Challenge 3: Install Builtins in Globals

Remove special builtin lookup. Populate the root environment with builtin `Value`s and test shadowing behaviour.

### Challenge 4: Tail Calls

Specify the exact stack and frame transformation for a tail call. Only then implement compiler detection and VM execution.

---

[← Part 3 — From Trees to Bytecode](part_03_bytecode_compiler.md) | [Course index](index.md) | [Part 5 — Closures and Escape Analysis →](part_05_closures_and_escape_analysis.md)
