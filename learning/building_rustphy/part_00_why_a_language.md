---
layout: default
title: Part 0 - Why I Built a Programming Language
---

# Part 0: Why I Built a Programming Language

**Building Rustphy: A Compiler From Scratch**

---

## The First Lie Compiler Diagrams Tell You

Every compiler course eventually shows a diagram like this:

```text
Source → Lexer → Tokens → Parser → AST → Semantic Analysis → IR → Code Generation
```

It is a good diagram. It is also dangerously tidy.

When I first started Rustphy, I did not sit down with a perfectly separated frontend, intermediate representation, optimization pipeline, and two carefully designed backends. I wanted to understand what actually happens when source code becomes something executable, so I built the pieces directly.

First came the lexer and parser. Then an AST. Then bytecode and a stack machine. Then functions. Then closures. Then I wanted WebAssembly.

That last sentence changed everything.

A bytecode interpreter can get away with carrying variable names into runtime and looking them up in chained hashmaps. WebAssembly does not naturally work that way. It wants locals, indices, function tables, linear memory, and structured control flow. As soon as I added a second target, assumptions that had been invisible in the first backend became design problems I had to solve explicitly.

This course is the story of those problems.

---

## Why Build a Compiler From Scratch?

Because compilers are cool! But more importantly, building one forces several areas of computer science to meet in one place:

- **Formal languages:** What strings are valid programs?
- **Data structures:** How do we represent programs as trees and graphs?
- **Algorithms:** How do we parse precedence, resolve names, and analyze lifetimes?
- **Runtime systems:** How are values, functions, arrays, and environments stored?
- **Computer architecture:** What can the target machine actually execute?
- **Software design:** Where should one phase stop and the next phase begin?

A compiler takes a vague statement like “I understand functions” and keeps asking more specific questions.

What is a function in the source language? Is it a statement or an expression? Where do parameters live? What happens when a nested function uses a variable from its parent? How is that variable still alive after the parent returns? How does an indirect call know which code and which environment belong together?

You can read the answers. Building a compiler makes you earn them.

---

## The Language I Wanted

I wanted Rustphy's syntax to be familiar enough that syntax itself would not dominate the project. A small JavaScript-like language gave me the constructs I wanted to study:

```javascript
let x = 10;
let active = true;
let message = "hello";

func square(n) {
    return n * n;
};

if (active) {
    print(square(x));
};
```

The language grew to include:

- Integers, floats, booleans, and strings
- Arrays and hashmap-like objects
- Prefix and infix operators
- Variable declarations and assignment
- `if` expressions and ternaries
- Functions, recursion, and closures
- Classic `for` loops and range-based `for-in`
- `break` and `continue`
- Multiple compilation targets

This was a deliberate size. Too small, and I would never hit the interesting runtime questions. Too large, and I would spend months implementing syntax before learning anything about compilation.

A useful first language needs one feature that makes each layer uncomfortable. For Rustphy, closures did that better than anything else.

---

## Why Rust?

Rust is not the easiest language in which to build a tree full of parent pointers and shared mutable environments. That is precisely why it is educational.

Compiler structures fit Rust's strengths extremely well:

### 1. Enums Model Language Alternatives

A token is one of several token kinds. An expression is one of several expression forms. An instruction is one of several opcodes. Rust enums let those alternatives carry their own data.

```rust
pub enum Value {
    Number(i128),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Closure {
        params: Vec<String>,
        addr: usize,
        env: Rc<RefCell<Environment>>,
    },
}
```

The compiler can then `match` exhaustively on every kind of value. If I add a new AST variant, Rust shows me the phases that do not understand it yet. That is a powerful property in a multi-stage compiler.

### 2. Ownership Makes Runtime Decisions Visible

An integer can be copied. A closure environment must be shared. An array must be mutable through multiple references. Rust will not let those differences remain hand-wavy.

Rustphy uses `Rc<RefCell<T>>` in its bytecode runtime:

- `Rc` gives multiple runtime values shared ownership.
- `RefCell` moves borrow checking to runtime so shared objects can still mutate.

That is not the only possible design, but it makes the semantics explicit. Closures really do hold a reference to the same environment object.

### 3. `Result` Makes Phase Boundaries Honest

Lexing, parsing, compilation, and execution can all fail for different reasons. Rust's `Result<T, E>` makes the happy path and failure path part of the function's type.

Rustphy currently uses `Result<_, String>` in many places. Later, we will see why replacing those strings with structured, span-aware diagnostics is an important next step.

---

## One Frontend, Two Very Different Backends

The architecture eventually became:

```text
                  ┌──────────────────────────────────────────┐
                  │              Shared frontend             │
                  │                                          │
source ──→ lexer ──→ tokens ──→ Pratt parser ──→ AST          │
                  └────────────────┬─────────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │                             │
                    ▼                             ▼
          Bytecode compiler              Symbol table
                    │                             │
                    ▼                             ▼
             Vec<Instruction>             Escape analysis
                    │                             │
                    ▼                             ▼
              Stack machine                 WAT compiler
                                                  │
                                                  ▼
                                          WebAssembly runtime
```

The shared frontend answers: **What program did the user write?**

Each backend answers: **How can my machine carry out that program?**

Those questions sound similar, but they are not.

### The Bytecode Backend

The bytecode compiler emits instructions such as:

```text
LDCN 40
LDCN 2
BINOP Add
ASSIGN "answer"
POP
```

The VM carries names like `"answer"` into runtime. When it executes `LDS { sym: "answer" }`, it walks a chain of environments until it finds that name.

This is flexible and simple. It is also work performed every time the program reads a variable.

### The WebAssembly Backend

The WASM backend wants to turn the same binding into something like:

```wat
(local $answer i32)
local.set $answer
local.get $answer
```

That requires knowing at compile time which declaration every identifier refers to. And if a closure captures `answer`, a normal WASM local is no longer enough—the value needs storage that the closure can reach later.

That is why Rustphy's symbol table and escape analysis belong to the WASM path, not the bytecode path. The second backend did not merely add another output format. It exposed a new class of questions.

---

## The Real Timeline

The commit history tells a more useful story than a polished architecture diagram.

### April 10, 2026 — The Compiler Lands in One Large Push

The first sequence of commits brought in most of the interpreter at once: tokens, lexer, AST, parser, bytecode compiler, machine, environment, symbol table, and early WASM experiments. It was broad rather than perfectly layered.

That is a normal way for a learning project to begin. You need enough vertical functionality to make a source program run before every abstraction has proved its worth.

### April 17 — WebAssembly Becomes a Real Backend

The old compiler work was reorganized into `wasm_compiler.rs`. The symbol table grew substantially. Escape analysis appeared. The REPL and CLI also became clearer.

This was the point where Rustphy stopped being only an interpreter project and became a multi-target compiler project.

### April 20 — Closures

Closure support introduced the design that still defines the WASM backend: a direct function entry point for known calls and a uniform closure trampoline for indirect calls.

### April 22–23 — Loops, Conditionals, and Objects

Control flow had to map to WASM's structured `block`/`loop` model. Objects required hundreds of lines of runtime helpers and a concrete memory layout.

### May 1 — The Browser Playground

`wasm-bindgen` exposed parsing, bytecode compilation, execution, and WAT generation to JavaScript. The compiler could now explain itself interactively in a web page.

### May 15 — `for-in` Ranges

One apparently small feature touched almost the whole compiler. We will use this commit as a case study in Part 7.

### September 2 — Floats Across the Pipeline

Float support reached tokens, AST nodes, bytecode, values, and VM arithmetic. The WASM backend received a placeholder rather than a complete representation. This is an honest example of feature parity lagging across backends.

The compiler did not grow in textbook chapter order. It grew when a feature made the current design insufficient.

---

## The Most Important Boundary: Meaning vs. Mechanism

Here is a distinction that will return throughout the course.

The AST should represent the **meaning and structure** of the source language:

```text
Infix(Add, Integer(40), Integer(2))
```

The backend should choose the **mechanism** used by the target:

```text
Bytecode VM:   LDCN 40; LDCN 2; BINOP Add
WebAssembly:   tag(40); tag(2); call $add_values
```

If target-specific mechanics leak into the parser, adding another backend becomes painful. If source-language details survive too late, every backend must repeatedly rediscover the same meaning.

Rustphy sometimes lowers syntax early—for example, compound assignments become normal assignments in the parser. It sometimes lowers later—for example, `for-in` ranges are handled by each backend. Those choices give us concrete tradeoffs to analyze rather than one universal rule.

---

## Our Running Example

Throughout the course, we will trace this program:

```javascript
let make_counter = func(start) {
    let value = start;
    return func() {
        value = value + 1;
        return value;
    };
};

let counter = make_counter(40);
print(counter());
print(counter());
```

Expected output:

```text
41
42
```

It looks small. It requires almost everything:

1. The lexer recognizes keywords, identifiers, integers, operators, and delimiters.
2. The parser builds nested function and assignment expressions.
3. The AST preserves lexical nesting.
4. The bytecode compiler emits function bodies and return instructions.
5. The VM creates a closure that captures `value`'s environment.
6. The symbol table binds the inner `value` uses to the outer declaration.
7. Escape analysis decides that `value` cannot be only a WASM local.
8. The WASM backend stores it in an environment reachable by the closure.
9. Indirect calls carry both a function index and an environment pointer.

This is why closures are such an excellent compiler feature: they force syntax, static analysis, code generation, and runtime representation to agree.

---

## Step Zero: Make the Pipeline Observable

Before adding clever optimizations, expose each stage.

Rustphy's CLI supports the same source in several modes:

```bash
# Inspect the AST
cargo run --bin rustphy-cli -- parse examples/01_hello_world.gph

# Inspect custom bytecode
cargo run --bin rustphy-cli -- compile bytecode examples/01_hello_world.gph

# Generate WebAssembly Text
cargo run --bin rustphy-cli -- compile wasm examples/01_hello_world.gph

# Execute through the bytecode VM
cargo run --bin rustphy-cli -- run examples/01_hello_world.gph
```

This is not just convenience. Observability is part of compiler design.

When a program produces the wrong answer, ask:

- Are the tokens wrong?
- Is the tree grouped incorrectly?
- Did the compiler emit the wrong jump target?
- Is the VM stack in the wrong order?
- Did static resolution select the wrong binding?
- Did a value receive the wrong runtime tag?

A black-box compiler turns one bug into six guesses. An observable compiler turns it into a phase boundary.

---

## Design Principles for This Course

As we build, I will keep returning to five principles.

### 1. Make One Representation Boring Before Adding the Next

Do not debug lexing, parsing, and code generation simultaneously if you can avoid it. Print tokens. Serialize the AST. Display bytecode.

### 2. Lower Complexity Deliberately

`x += 1` does not need a unique opcode if it can become `x = x + 1`. But early desugaring has semantic costs when an lvalue can have side effects. Every lowering should state what it preserves.

### 3. Let the Target Shape the Backend, Not the Language

The bytecode VM uses jumps. WASM uses structured control flow. The source language should not care.

### 4. Treat Runtime Representation as Part of the Compiler

The generated code and runtime helpers form one contract. A tagged integer is only useful because every arithmetic, comparison, condition, and print operation agrees on the tag.

### 5. Preserve the Mistakes Long Enough to Learn From Them

A fixed-size closure table, a missing source span, or a float stub is not a design recommendation. It is evidence of where the current architecture runs out.

---

## Checkpoint

At this point, you should be able to explain the full Rustphy pipeline without implementation detail:

```text
Characters
  ↓ lexer
Tokens
  ↓ parser
AST
  ├─→ bytecode instructions → virtual machine
  └─→ symbol resolution → escape analysis → WAT → WebAssembly runtime
```

The next part begins with the smallest possible question:

> How does the compiler recognize a word?

The answer will lead us from individual bytes to the first meaningful representation of a program.

---

## Quiz

1. Why did adding a WebAssembly backend require more static analysis than the bytecode backend?
2. What is the difference between source-language meaning and target-machine mechanism?
3. Why are Rust enums useful for tokens, AST nodes, instructions, and values?
4. What does a closure need in addition to a function's code address?
5. Why should a compiler expose intermediate representations?

<details>
<summary><strong>Answers</strong></summary>

1. The bytecode VM can carry names into runtime and resolve them through environment hashmaps. WASM locals and environment slots need compile-time binding and storage decisions.
2. Meaning describes what the program says; mechanism describes how a particular target carries it out.
3. Enums model a closed set of alternatives with variant-specific data and support exhaustive pattern matching.
4. It needs the lexical environment containing its captured variables.
5. Intermediate output localizes bugs to a phase and makes the compiler teachable and debuggable.

</details>

---

## Practice

### Challenge 1: Choose Your Language Surface

Write ten programs your own language should support. Do not implement anything yet. Use those programs to list required tokens, statements, expressions, values, and runtime features.

### Challenge 2: Draw Two Pipelines

Draw one backend that interprets names dynamically and one that resolves variable slots statically. Where does the symbol table enter each design?

### Challenge 3: Trace One Closure

For the `make_counter` example, write down:

- Where `value` is declared
- Where it is read
- Where it is written
- Which function owns it
- Which function captures it
- Why its lifetime extends beyond the call to `make_counter`

---

[← Course index](index.md) | [Part 1 — Teaching the Compiler to Read →](part_01_lexer.md)
