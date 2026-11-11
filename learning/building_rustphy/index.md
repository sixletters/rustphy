---
layout: default
title: Building Rustphy - A Compiler From Scratch
---

# Building Rustphy: A Compiler From Scratch

**By Harris Maung**

*A narrative, code-first course about turning a programming language from an idea into bytecode, closures, WebAssembly, and a browser playground.*

---

## Why I Wrote This Course

Most compiler tutorials show you the finished architecture first. There is a lexer box, a parser box, an AST box, and a code-generation box. Everything fits together so cleanly that it can feel like the author simply knew where every piece belonged from day one.

That is not how I built Rustphy.

I began with a language, a lexer, a parser, and a stack machine. Then the design started pushing back. Functions needed scopes. Closures needed environments. WebAssembly needed static name resolution. Static name resolution needed a symbol table. Closures on WebAssembly needed escape analysis. A small feature like `for i in 0..10` suddenly touched the lexer, parser, AST, bytecode compiler, virtual machine, symbol table, escape analysis, and WASM backend.

That messy evolution is exactly what makes Rustphy useful as a course.

This series follows the **real implementation and commit history** of Rustphy. I will explain the choices that worked, the choices that still need work, and the point where a simple interpreter grew into a compiler with two very different backends:

```text
                              ┌──→ Bytecode compiler ──→ Stack VM
Source ──→ Lexer ──→ Parser ──→ AST ──┤
                              └──→ Symbol table ──→ Escape analysis ──→ WAT
```

The goal is not merely to read about compilers. By the end, you should understand the decisions well enough to build one yourself.

---

## The Language We Are Building

Rustphy is a dynamically typed language written in Rust. Its syntax is deliberately familiar: part JavaScript, part C, and small enough that I can experiment without spending all my time on surface syntax.

```javascript
let greeting = "Hello";
let numbers = [1, 2, 3];

func add(a, b) {
    return a + b;
};

for (i in 0..3) {
    print(greeting);
    print(add(numbers[i], 10));
};
```

The implementation includes:

- A hand-written lexer
- A recursive-descent and Pratt parser
- A serializable abstract syntax tree
- A custom bytecode instruction set
- A stack-based virtual machine
- Lexical environments and closures
- A compile-time symbol table
- Escape analysis
- A WebAssembly Text (WAT) backend
- A tagged runtime representation for WASM
- A CLI, REPL, and browser playground

This is not a wrapper around LLVM, Cranelift, or a parser generator. The interesting machinery is implemented directly so that we can see it.

---

## Course Map

### [Part 0 — Why I Built a Programming Language](part_00_why_a_language.md)

We begin with the real question: why build a language at all? I introduce Rustphy, explain why I chose Rust, and trace how the project evolved from a bytecode interpreter into a dual-backend compiler.

**You will learn:**

- What a compiler pipeline actually does
- Why the frontend and backend should be separated
- Why Rust enums are such a natural fit for compilers
- Why the second backend changed the architecture
- How to choose a small but meaningful first language

---

### [Part 1 — Teaching the Compiler to Read](part_01_lexer.md)

We turn a raw string into tokens with a hand-written, one-byte-lookahead lexer. The most interesting bug is not recognizing `+`; it is deciding whether a dot belongs to `3.14`, `0..10`, or `object.field`.

**You will learn:**

- How to design payload-bearing tokens
- How current-character and lookahead cursors work
- How keywords, identifiers, strings, and operators are scanned
- How the lexer distinguishes floats from ranges
- Where lexical error handling belongs

---

### [Part 2 — Giving the Language Structure](part_02_parser_and_ast.md)

Tokens are words; the parser turns them into sentences. We build recursive AST enums and a Pratt parser whose core loop is only a few lines long, then use precedence to make `1 + 2 * 3` mean the right thing.

**You will learn:**

- How recursive ASTs are represented with `Box`
- How statements differ from expressions
- How Pratt parsing models precedence
- How `x += 5` is desugared into simpler syntax
- How `a.b` and `a["b"]` converge on one AST node

---

### [Part 3 — From Trees to Bytecode](part_03_bytecode_compiler.md)

We lower the AST into a custom instruction set. This is where high-level control flow becomes jumps, functions become code addresses, and unknown jump targets are repaired with backpatching.

**You will learn:**

- How to design a stack-machine instruction set
- How AST traversal becomes code generation
- How scopes and assignments are represented
- How to compile conditionals and loops
- How function bodies live inside one instruction stream

---

### [Part 4 — Building the Virtual Machine](part_04_virtual_machine.md)

Bytecode is only useful if something can execute it. We build the operand stack, runtime stack, environments, values, builtins, calls, and returns that make Rustphy programs run.

**You will learn:**

- How a fetch–decode–execute loop works
- How runtime values are represented in Rust
- Why calls need a second stack
- How lexical closures capture environments
- How `RESET` unwinds both scopes and calls

---

### [Part 5 — Closures Force Us to Think Like Compiler Engineers](part_05_closures_and_escape_analysis.md)

The bytecode VM can resolve names dynamically, but WebAssembly locals cannot. We build a symbol table, bind every identifier use to a declaration, and use escape analysis to decide which variables can stay as WASM locals and which must move into heap environments.

**You will learn:**

- The difference between names and bindings
- How nested lexical scopes are represented
- How identifier resolution works
- What it means for a variable to escape
- Why closure conversion changes storage decisions

---

### [Part 6 — Compiling to WebAssembly](part_06_webassembly.md)

We emit WAT directly and build the runtime ourselves: tagged integers, heap objects, strings, arrays, environments, closure objects, a function table, and structured control flow.

**You will learn:**

- How to map dynamic values onto WASM's `i32`
- How a bump allocator and heap headers work
- How functions receive both direct and closure entry points
- How `call_indirect` powers first-class functions
- How the browser playground reaches the compiler through `wasm-bindgen`

---

### [Part 7 — What Building a Compiler Actually Taught Me](part_07_lessons.md)

We follow one cross-cutting feature through the entire pipeline, compare the two backends, inspect the unfinished edges, and turn those lessons into a practical roadmap for your own compiler.

**You will learn:**

- Why adding syntax is the smallest part of adding a feature
- Why two backends expose hidden assumptions
- Which shortcuts are worth taking early
- Which technical debt now limits Rustphy
- What I would build next

---

## How to Use This Course

You can read the series in order, but do not only read it. Compiler ideas become much clearer when you make the machine do something new.

For every part:

1. Read the narrative once without trying to memorize it.
2. Trace the relevant Rustphy source alongside the chapter.
3. Run the checkpoint program.
4. Answer the quiz without looking back.
5. Attempt at least one challenge.
6. Change the language—even if your first attempt breaks it.

A compiler rewards active confusion. If a concept feels abstract, reduce it to one input and trace every representation it passes through.

For example:

```text
Source:      let answer = 40 + 2;
Tokens:      Let Ident("answer") Assign Int("40") Plus Int("2") Semicolon
AST:         Let(answer, Infix(Add, Integer(40), Integer(2)))
Bytecode:    LDCN 40, LDCN 2, BINOP Add, ASSIGN answer, POP
Runtime:     environment["answer"] = Number(42)
```

That one habit—following a value across every phase—is one of the most useful debugging tools in compiler engineering.

---

## A Note About Imperfection

Rustphy is a learning compiler, not a production language pretending to be finished.

Some current limitations are intentional teaching opportunities:

- Tokens do not carry source spans yet.
- Lexer errors are not consistently structured.
- The bytecode VM defines a tail-call instruction that it does not execute.
- The two backends do not support exactly the same semantics.
- The WASM closure table has a fixed size.
- The WASM float path is still a stub.
- The heap has no garbage collector.

I will point these out rather than hide them. A clean final diagram tells you what exists. An honest unfinished edge tells you **why the next compiler pass needs to exist**.

---

## Source Code

- [Rustphy repository](https://github.com/sixletters/rustphy)
- [Language grammar](https://github.com/sixletters/rustphy/blob/main/docs/GRAMMAR.md)
- [Compiler source](https://github.com/sixletters/rustphy/tree/main/src)
- [Example programs](https://github.com/sixletters/rustphy/tree/main/examples)
- [MIT compiler course notes](../compiler_course/)

---

[Start the course: Part 0 — Why I Built a Programming Language →](part_00_why_a_language.md)

[← Back to Home](../../)
