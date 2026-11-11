---
layout: default
title: Part 3 - From Trees to Bytecode
---

# Part 3: From Trees to Bytecode

**Building Rustphy: A Compiler From Scratch**

---

## Trees Explain Programs; Machines Execute Sequences

Our parser produces a tree:

```text
Let answer
└── Add
    ├── Integer 40
    └── Integer 2
```

That is an excellent representation for understanding structure. It is not yet a convenient representation for execution.

A machine wants a sequence:

```text
LDCN 40
LDCN 2
BINOP Add
ASSIGN "answer"
POP
```

The bytecode compiler bridges those representations. It walks the tree recursively and emits instructions in the order the stack machine will need them.

This phase is our first real **lowering**: source-language concepts become target-machine operations.

---

## Why Invent Bytecode?

Why not execute the AST directly?

An AST-walking interpreter is a valid and often excellent first implementation. But bytecode gives us a useful middle layer:

- Parsing happens once.
- Instructions are simpler than AST nodes.
- Control flow becomes explicit.
- The VM dispatch loop is compact.
- Bytecode can be serialized and inspected.
- Later optimizations have a lower-level representation to transform.

The tradeoff is that we must design an instruction set and a runtime machine together. An instruction's meaning is a contract between compiler and VM.

---

## Step 1: Design a Stack Machine

Rustphy targets a stack machine. Instructions take operands from an implicit operand stack and push results back.

To evaluate:

```javascript
(2 + 3) * 4
```

we emit:

```text
LDCN 2        stack: [2]
LDCN 3        stack: [2, 3]
BINOP Add     stack: [5]
LDCN 4        stack: [5, 4]
BINOP Multiply stack: [20]
```

There are no explicit temporary registers:

```text
ADD r3, r1, r2
```

The top stack values are the implicit inputs and output.

This makes code generation simple: recursively compile children in stack order, then emit the operation.

---

## Step 2: Represent Instructions as an Enum

Rustphy's bytecode is a serializable Rust enum. Operands live directly inside variants:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    LDCB { val: bool },
    LDCN { val: i128 },
    LDCF { val: f64 },
    LDSL { val: String },
    LDI { val: String },
    LDS { sym: String },
    LDF { addr: usize, params: Vec<String> },
    // ...
}
```

There is no separate encoded byte stream and no constant pool yet. The instruction sequence is a `Vec<Instruction>`.

That is a very practical learning design:

- Pattern matching is easy.
- Debug output is readable.
- Operands retain normal Rust types.
- Serialization requires little work.

It is not compact like a production bytecode format, but compactness is not the first problem to solve.

---

## The Instruction Set

The main families are:

### Constants and Names

| Instruction | Meaning |
|---|---|
| `LDCB { val }` | Push boolean |
| `LDCN { val }` | Push integer |
| `LDCF { val }` | Push float |
| `LDSL { val }` | Push string |
| `LDI { val }` | Push a name as an assignment target |
| `LDS { sym }` | Look up a symbol's runtime value |
| `LDF { addr, params }` | Create a closure for code at `addr` |

### Operators

```text
UNOP Negative
UNOP Not

BINOP Add, Multiply, Minus, Divide, Modulo
BINOP Lt, Le, Ge, Gt, Eq, Neq
BINOP And, Or, Assign, In
```

### Aggregates

| Instruction | Stack behaviour |
|---|---|
| `MKARR { size }` | Pop `size` values, create array, push it |
| `MKHASH { size }` | Pop key/value pairs, create hashmap, push it |
| `LDAI` | Load array/hash index |
| `STAI` | Store array/hash index and leave value |

### Control Flow

| Instruction | Meaning |
|---|---|
| `JOF { addr }` | Pop condition; jump if false |
| `GOTO { addr }` | Unconditional jump |
| `DONE` | Halt program |
| `RESET` | Return from a function |

### Scopes and Calls

| Instruction | Meaning |
|---|---|
| `ENTERSCOPE { syms }` | Extend environment and predeclare names |
| `EXITSCOPE` | Restore parent environment |
| `ASSIGN { sym }` | Assign/decorate current scope name |
| `CALL { arity }` | Invoke callable with arguments |
| `TAILCALL { arity }` | Reserved, but not currently executed |
| `POP` | Discard top operand |

Jump targets are absolute instruction indices. Symbol operands are strings, not local-slot numbers. That tells us something fundamental about this backend: name resolution remains a runtime operation.

---

## Step 3: Track the Current Instruction Address

The compiler keeps:

```rust
pub struct Compiler {
    wc: i128,
    instructions: Vec<Instruction>,
    loop_context_stack: Vec<LoopContext>,
}
```

`wc` is the word counter—the address of the next instruction.

Every emitted instruction increments it. The vector length could often provide the same information, but an explicit counter makes the intended address model clear.

Compilation dispatches on AST variants and appends `DONE` after the root program.

---

## Step 4: Compile Literals and Operators

Literals are direct:

```text
Integer(42)   → LDCN { val: 42 }
Float(3.14)   → LDCF { val: 3.14 }
Boolean(true) → LDCB { val: true }
String("hi") → LDSL { val: "hi" }
```

For a binary expression:

```text
Infix(Add, left, right)
```

the compiler emits:

```text
compile(left)
compile(right)
BINOP Add
```

Order matters. The VM pops the right operand and then the left operand. If the compiler reverses emission, subtraction and division silently produce wrong results.

This is why stack effects should be documented for every instruction.

---

## Step 5: Compile Variable Declarations

A declaration:

```javascript
let x = 40 + 2;
```

becomes:

```text
LDCN 40
LDCN 2
BINOP Add
ASSIGN "x"
POP
```

Why `POP`?

`ASSIGN` leaves the assigned value on the operand stack. That is useful because assignment is also an expression:

```javascript
let y = (x = 10);
```

But a standalone declaration does not need its value to remain, so the compiler discards it.

This is a common stack-machine pattern: expression compilation leaves exactly one result; statement compilation restores stack balance by dropping unused results.

---

## Step 6: Enter Scopes With Their Names Already Known

The bytecode backend does not use Rustphy's compile-time symbol table. Instead, it scans each block for `let` declarations before compiling its statements.

Conceptually:

```rust
fn scan(block: &StatementNode) -> Vec<String> {
    // Walk direct statements and collect declared names.
}
```

Then a block becomes:

```text
ENTERSCOPE { syms: ["x", "y"] }
...block instructions...
EXITSCOPE
```

At runtime, `ENTERSCOPE` creates a child environment and pre-binds each name to `Unassigned`.

Why predeclare names?

It makes the scope's set of bindings exist from scope entry, even if their values are assigned later. It is a lightweight form of declaration hoisting and lets assignment distinguish “declared but not initialized” from “never declared.”

### Runtime Name Resolution

Reading `x` emits:

```text
LDS { sym: "x" }
```

The compiler does not decide whether `x` is local, captured, or global. The VM walks the current environment and its parents.

That simplicity is one of the bytecode backend's defining choices.

---

## Step 7: Compile `if` With Backpatching

Consider:

```javascript
if (condition) {
    print("yes");
} else {
    print("no");
};
```

The instruction shape is:

```text
compile condition
JOF else_address
compile then block
GOTO end_address
else_address:
compile else block
end_address:
```

The problem is timing.

When we emit `JOF`, we have not compiled the then block yet. We do not know where the else block will begin.

So we lie temporarily:

```rust
let jof_idx = self.wc as usize;
self.instructions.push(Instruction::JOF { addr: 0 });
self.wc += 1;
```

After compiling the then block, we know the else address and replace the placeholder:

```rust
self.instructions[jof_idx] = Instruction::JOF {
    addr: self.wc as usize,
};
```

The same happens for the jump over the else branch:

```rust
let goto_idx = self.wc as usize;
self.instructions.push(Instruction::GOTO { addr: 0 });
// compile else...
self.instructions[goto_idx] = Instruction::GOTO {
    addr: self.wc as usize,
};
```

This technique is **backpatching**: emit an incomplete forward reference, remember where it lives, and repair it when the target exists.

---

## Step 8: Compile Loops as Jumps

A loop conceptually becomes:

```text
condition_address:
    compile condition
    JOF end_address
    compile body
    GOTO condition_address
end_address:
```

For:

```javascript
for (i < 3) {
    i = i + 1;
};
```

we might emit:

```text
0000 LDS "i"           ← condition address
0001 LDCN 3
0002 BINOP Lt
0003 JOF 10             ← patched after body
0004 LDS "i"
0005 LDCN 1
0006 BINOP Add
0007 ASSIGN "i"
0008 POP
0009 GOTO 0
0010 ...                ← loop end
```

### `break` and `continue`

The compiler maintains a stack of loop contexts:

```rust
struct LoopContext {
    continue_addr: usize,
    break_patchs: Vec<usize>,
}
```

- `continue` can immediately jump to the known continuation address.
- `break` needs the loop end, which is unknown until body compilation finishes. Each `break` emits a placeholder `GOTO` and records its instruction index.

After compiling the loop, every recorded break is patched to the same end address.

Why a **stack** of loop contexts? Nested loops. The innermost `break` must target the innermost loop.

The current classic `For` implementation has a rough edge: its context is read at the end rather than popped, unlike `ForIn`. That can leave stale context behind. Compiler bookkeeping structures need the same disciplined push/pop invariants as runtime stacks.

---

## Step 9: Desugar `for-in` in the Backend

Rustphy's range loop:

```javascript
for (i in 0..3) {
    print(i);
};
```

is lowered approximately to:

```javascript
i = 0;
for (i < 3) {
    print(i);
    i = i + 1;
};
```

For `0..=3`, the condition uses `<=` instead of `<`.

The bytecode compiler emits:

1. Range start and assignment to `i`
2. Condition address
3. Load `i` and range end
4. `Lt` or `Le`
5. Placeholder `JOF`
6. Body
7. Load `i`, push `1`, add, assign
8. Jump to condition
9. Patch exits

The `In` binary opcode exists, but a well-formed range loop is lowered before runtime. If `In` reaches the VM, it is an implementation error.

This raises a design question: should `for-in` have been desugared once in a shared lowering pass rather than independently in both backends? We will revisit that in Part 7.

---

## Step 10: Put Function Bodies Inside the Instruction Stream

Functions introduce a new challenge. We want executing a function expression to create a closure, not immediately run its body.

Rustphy emits this shape:

```text
LDF function_body_address, params
GOTO after_function_body
function_body_address:
    ...body instructions...
    RESET
after_function_body:
```

The compiler code captures that pattern:

```rust
self.instructions.push(Instruction::LDF {
    addr: (self.wc + 2) as usize,
    params: params_to_string,
});

self.instructions.push(Instruction::GOTO {
    addr: (self.wc + 1) as usize,
});
let goto_idx = self.wc;

self.compile_statement(body)?;
self.instructions.push(Instruction::RESET);

self.instructions[goto_idx as usize] = Instruction::GOTO {
    addr: self.wc as usize,
};
```

At runtime:

- `LDF` packages the body address, parameters, and current environment into a closure value.
- `GOTO` skips over the inline body during normal sequential execution.
- `CALL` later sets the program counter to the body address.
- `RESET` returns to the saved caller state.

An explicit `return expression;` compiles to:

```text
compile expression
RESET
```

An implicit trailing `RESET` ensures falling off the body also returns.

---

## Step 11: Compile Calls in Stack Order

For a call:

```javascript
add(2, 3)
```

the compiler must agree with the VM about operand order.

Conceptually:

```text
load callable
compile argument 1
compile argument 2
CALL { arity: 2 }
```

The VM pops the arguments and callable according to that convention, binds arguments to parameter names, and jumps into closure code.

This is a recurring compiler principle:

> There is no correct instruction order in isolation. There is only an order that matches the runtime contract.

---

## Step 12: Compile Assignment Targets Differently

A variable read:

```javascript
x
```

emits:

```text
LDS { sym: "x" }
```

A simple assignment expression:

```javascript
x = value
```

uses `LDI` to put the identifier *name* on the stack, compiles the value, then executes `BINOP Assign`.

An indexed assignment:

```javascript
array[i] = value
```

compiles:

```text
compile array
compile i
compile value
STAI
```

This is why lvalue representation matters. Different target shapes require different storage mechanics.

---

## No Constant Pool—Yet

Literal values are embedded directly:

```text
LDCN { val: 42 }
LDSL { val: "hello" }
```

A production bytecode format often stores constants once:

```text
constants[0] = "hello"
LDC 0
```

Benefits include smaller instructions, deduplication, and compact binary encoding.

Rustphy's direct operands keep the first compiler easier to inspect. This is a good example of delaying an optimization until the representation needs it.

---

## A Complete Compilation Trace

Source:

```javascript
let max = func(a, b) {
    if (a > b) {
        return a;
    } else {
        return b;
    };
};
print(max(4, 7));
```

Conceptual bytecode:

```text
0000 ENTERSCOPE []
0001 LDF addr=3 params=["a", "b"]
0002 GOTO 14
0003 ENTERSCOPE []
0004 LDS "a"
0005 LDS "b"
0006 BINOP Gt
0007 JOF 10
0008 LDS "a"
0009 RESET
0010 LDS "b"
0011 RESET
0012 EXITSCOPE
0013 RESET
0014 ASSIGN "max"
0015 POP
0016 LDS "print"
0017 LDS "max"
0018 LDCN 4
0019 LDCN 7
0020 CALL 2
0021 CALL 1
0022 POP
0023 EXITSCOPE
0024 DONE
```

Exact output can vary with AST and statement handling, but the core lowering is visible:

- Tree nesting became addresses.
- Conditionals became jumps.
- A function became closure construction plus skipped inline code.
- Names survived into runtime.

---

## The Defined-But-Unfinished `TAILCALL`

The instruction enum includes:

```rust
TAILCALL { arity: usize }
```

But the compiler does not emit it, and the VM does not have a concrete execution arm for it.

This is a useful artifact of compiler evolution. An opcode declaration is not a feature. A complete feature requires:

1. Detection or explicit generation
2. Defined stack effects
3. Runtime execution semantics
4. Tests
5. Interaction with errors and debugging

Tail-call optimization would reuse the current call frame instead of pushing a new one. That requires careful argument movement and environment handling—not merely adding a name to an enum.

---

## What I Would Improve Next

### 1. Centralize Instruction Emission

Instead of manually pushing and incrementing `wc` everywhere:

```rust
fn emit(&mut self, instruction: Instruction) -> usize {
    let index = self.instructions.len();
    self.instructions.push(instruction);
    index
}
```

Then patch through typed helpers.

### 2. Make Stack Effects Explicit

Document or verify:

```text
BINOP: [.., left, right] → [.., result]
JOF:   [.., condition]   → [..]
CALL:  [.., fn, args...] → [.., result]
```

A debug verifier could detect inconsistent stack heights across control-flow joins.

### 3. Use a Constant Pool

Once compact serialization matters, intern strings and large constants.

### 4. Resolve Names to Slots

A slot-based bytecode backend could replace repeated hashmap lookup with local indices while retaining environments for captures.

### 5. Add a Shared Lowering IR

Loops, compound assignments, and other sugar could be normalized once before both backends. This would reduce duplicated semantics.

---

## Checkpoint

We now have a complete instruction sequence, but instructions are still inert data.

The next part builds the machine that gives them meaning:

```text
AST
  ↓ bytecode compiler
Vec<Instruction>
  ↓ virtual machine
Values, output, side effects
```

We need an operand stack, a call stack, lexical environments, a program counter, and one very important invariant: the compiler and VM must agree about everything.

---

## Quiz

1. Why is bytecode useful compared with directly walking the AST?
2. How does a stack machine represent temporary expression values?
3. What is backpatching, and why do forward jumps need it?
4. Why are loop contexts stored as a stack?
5. Why does function compilation emit a `GOTO` immediately after `LDF`?
6. Why does declaration compilation emit `POP` after `ASSIGN`?
7. What architectural choice is revealed by `LDS { sym: String }`?

<details>
<summary><strong>Answers</strong></summary>

1. It makes control flow explicit, reduces runtime traversal complexity, supports inspection/serialization, and provides a lower-level optimization boundary.
2. Operands and intermediate results live on an implicit operand stack.
3. It emits a placeholder target and repairs it after the destination address is known.
4. Nested `break` and `continue` must target the innermost active loop.
5. Function bodies live inline in the instruction stream and must be skipped during ordinary execution.
6. Assignment leaves its value for expression semantics, but a declaration statement does not need it.
7. Variables are resolved by name at runtime rather than to static local slots during bytecode compilation.

</details>

---

## Practice

### Challenge 1: Disassembler

Write a formatter that prints instruction addresses, operands, and stack effects. Label jump targets for readability.

### Challenge 2: Add `while`

Introduce a `while` statement and compile it using the existing jump machinery. Support nested `break` and `continue`.

### Challenge 3: Stack Verifier

Build a control-flow analysis that checks every instruction has enough operands and every branch into one address agrees on stack height.

### Challenge 4: Constant Pool

Replace embedded string operands with indices into a constant pool. Serialize the program as constants plus instructions.

---

[← Part 2 — Giving the Language Structure](part_02_parser_and_ast.md) | [Course index](index.md) | [Part 4 — Building the Virtual Machine →](part_04_virtual_machine.md)
