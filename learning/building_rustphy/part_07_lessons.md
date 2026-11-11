---
layout: default
title: Part 7 - What Building a Compiler Actually Taught Me
---

# Part 7: What Building a Compiler Actually Taught Me

**Building Rustphy: A Compiler From Scratch**

---

## One Feature, Every Phase

I want to end this series by tracing a single feature through the entire compiler. Not a toy feature. A real one that forced me to change seven files.

The feature is `for-in` ranges:

```javascript
for (i in 0..10) {
    print(i);
}
```

It looks simple. It took a cross-cutting commit (`bf4e70a`, May 15) that touched the lexer (+105 lines), parser (+254 lines), AST, symbol table (+96 lines), escape analysis (+73 lines), bytecode compiler (+120 lines), VM, and WASM compiler (+133 lines).

Let me walk through every phase.

---

### The Lexer: Two New Tokens

The lexer needed to recognize `..` and `..=`:

```rust
b'.' => {
    if matches!(self.peek_char(), b'.') {
        self.read_char(); // consume second dot
        if matches!(self.peek_char(), b'=') {
            self.read_char(); // consume =
            self.read_char(); // move past
            return Token::DotDotEquals;
        }
        self.read_char(); // move past second dot
        return Token::DotDot;
    }
    // single dot handling...
}
```

This interacts with the float rule from Part 1. `3.14` must stay a float, `0..10` must become `Int(0) DotDot Int(10)`, and `3..5` must not be confused with `3.` followed by `.5`.

The rule from the lexer chapter handles this: a dot only joins a number if a digit follows it. `3.` followed by `.` is not a digit, so `3` stays an integer and the dots become range tokens.

**Lesson:** Lexer features are never truly isolated. Every new token interacts with existing prefix-sharing tokens.

---

### The AST: A New Expression Node

Ranges got their own AST node:

```rust
Range {
    token: Token,
    start: Box<ExpressionNode>,
    end: Box<ExpressionNode>,
    inclusive: bool,
    id: i32,
}
```

Why not desugar ranges into something else at parse time? Because `0..10` is an expression — it could theoretically be assigned to a variable, passed to a function, or used outside a `for-in` loop. The AST should represent what the programmer wrote, not what one backend happens to do with it.

**Lesson:** Resist desugaring in the parser. Preserve meaning; lower later.

---

### The Parser: One Precedence Decision

The Pratt parser needed to know where `..` sits in the precedence ladder. I put it at the same level as `+` and `-`:

```text
LOWEST < TERNARY < ASSIGN < LOGICAL < EQUALS < LESSGREATER < SUM(+, -, .., ..=) < PRODUCT < PREFIX < CALL
```

And `in` went at comparison precedence (`LESSGREATER`).

This means `i in 0..10` parses as:

```text
Infix(In)
├── Identifier("i")
└── Range(0, 10, exclusive)
```

The `..` binds tighter than `in`, so the range forms first. Then `in` wraps it.

The `for` statement parser recognizes this structure after parsing the parenthesized expression:

```rust
// After parsing the for-condition expression:
if root is Infix { operator: In, left, right } {
    // It's a for-in loop
    StatementNode::ForIn { variable: left, iterable: right, body }
} else {
    // It's a regular for loop
    StatementNode::For { condition: root, for_block: body }
}
```

One precedence table entry, one structural check — the parser changes were small. But they had to be exactly right.

**Lesson:** Precedence is a language design decision, not an implementation detail. Changing one rank changes how every program groups.

---

### The Symbol Table: A New Binding

The loop variable `i` is a variable declaration that lives in the loop body scope. The symbol table had to handle `ForIn` by:

1. Creating a scope for the loop body
2. Declaring the loop variable in that scope
3. Resolving uses of the loop variable within the body

This required 96 new lines because `ForIn` has its own scoping rule: the loop variable is scoped to the loop body, not the enclosing block.

**Lesson:** Every new binding form needs symbol table support. You cannot resolve names you haven't declared.

---

### Escape Analysis: A New Capture Path

If a closure inside the loop body captures the loop variable:

```javascript
let handlers = [];
for (i in 0..3) {
    handlers[i] = func() { return i; };
};
```

Then `i` escapes. The escape analysis needed 73 new lines to walk `ForIn` nodes, enter the loop body scope, and mark the loop variable as escaping when captured.

This matters for the WASM backend: the loop variable must live in a heap environment slot, not a WASM local, so that closures created in different iterations can access it.

**Lesson:** Every new binding form is also a new capture opportunity. Analysis passes must handle every syntactic form that introduces names.

---

### The Bytecode Compiler: Desugar to a While Loop

The bytecode backend desugars `for (i in start..end)` into:

```javascript
i = start;
while (i < end) {    // or i <= end for ..=
    body;
    i = i + 1;
};
```

This emitted 120 new lines of instruction sequences:

```text
;; i = start
<compile start>
ASSIGN "i"
POP

;; loop condition
loop_start:
LDS "i"
<compile end>
BINOP Lt            ; or Le for ..=
JOF loop_end

;; body
<compile body>

;; increment
LDS "i"
LDCN 1
BINOP Add
ASSIGN "i"
POP

GOTO loop_start
loop_end:
```

The `In` binary operator exists in the instruction set but is never executed — a well-formed `for-in` is fully desugared before runtime. If `In` reaches the VM, it means the compiler has a bug.

**Lesson:** Desugaring in the backend works, but it duplicates logic if you have multiple backends. A shared lowering pass would compute this once.

---

### The VM: Nothing New

The bytecode VM needed zero changes. `for-in` was fully desugared into instructions the VM already knew.

This is the payoff of desugaring: the runtime stays small.

**Lesson:** The fewer concepts your runtime needs to understand, the less code you maintain. Desugar aggressively if you can afford the compile-time complexity.

---

### The WASM Compiler: A Different Lowering

The WASM backend needed its own `for-in` implementation (+133 lines). It also desugars, but to WASM's structured control flow:

```wat
;; i = start
<compile start>
local.set $i

(block $break_N
  (loop $continue_N
    ;; condition
    local.get $i
    <compile end>
    call $lt_values      ;; or $le_values
    call $untag_immediate
    i32.eqz
    br_if $break_N

    ;; body
    <compile body>

    ;; increment
    local.get $i
    i32.const 1
    call $add_values
    local.set $i

    br $continue_N
  )
)
```

The logic is the same as the bytecode lowering, but the target instructions are completely different. Block/loop structure instead of jumps. Runtime helper calls instead of `BINOP`. Local variables instead of environment hashmaps.

**Lesson:** Two backends mean two lowerings. A shared IR or lowering pass would eliminate the duplication — but building one is a design project in itself.

---

## The Full Cost of One Feature

| Phase | Lines changed | What was needed |
|---|---|---|
| Lexer | +105 | Recognize `..` and `..=` without breaking floats |
| Parser | +254 | Precedence, range AST, for-in detection |
| AST | +~50 | New `Range` node, `ForIn` statement |
| Symbol table | +96 | Loop variable scoping |
| Escape analysis | +73 | Loop variable capture detection |
| Bytecode compiler | +120 | Desugar to jump-based loop |
| VM | 0 | Reuse existing instructions |
| WASM compiler | +133 | Desugar to structured control flow |
| **Total** | **~830 lines** | |

For `for (i in 0..10)`.

This is what compiler engineering actually looks like. The idea fits in one sentence. The implementation touches everything.

---

## What I Got Right

### Starting With a Working Interpreter

The bytecode VM gave me a runnable language before I understood all the theory. Having a working system to modify is infinitely better than designing in a vacuum.

### Making Everything Observable

`rustphy parse` prints the AST as JSON. `rustphy compile bytecode` shows instructions. `rustphy compile wasm` shows WAT. Every stage is inspectable. This is the single most important debugging tool I built.

### Using Rust

The type system caught more bugs than my tests did. `match` exhaustiveness checking meant that adding a new AST node forced me to handle it in every phase. The compiler became my checklist.

### Building the Second Backend

The WASM backend forced me to build the symbol table, escape analysis, and a runtime system. Without it, I would have a working interpreter and a shallow understanding of what compilers actually do.

---

## What I Would Do Differently

### Add Source Spans Earlier

Every token should have a byte offset. Every AST node should have a span. Every error should point to a location. Retrofitting this is painful because every struct gains a field and every constructor gains a parameter. Do it first.

### Use Structured Errors From Day One

`Result<_, String>` works, but it collapses all error information into text. A proper `CompilerError` enum with kinds, spans, and severity would make diagnostics testable and user-friendly.

### Design the IR Between Front and Back Ends

The AST goes directly to two different backends. A shared intermediate representation — after desugaring, after name resolution, before target-specific code generation — would eliminate duplicated lowering logic.

### Test Backend Parity

Floats work on the VM but not WASM. Short-circuit works on WASM but not the VM. String concat works on the VM but not WASM. I need a conformance test suite that runs the same programs through both backends and compares outputs.

### Validate Node IDs

The parser clones node IDs during compound-assignment desugaring and uses ID `0` for synthesized nodes. Analysis passes that key on node identity will silently misbehave. Unique IDs should be an invariant enforced by the type system or a validation pass.

---

## The Roadmap

If I were continuing this project (and I am), here is what comes next:

### Short Term

1. **Fix the closure table size** — count functions at compile time
2. **String concatenation on WASM** — implement `$add_values` for heap strings
3. **Source spans** — add position tracking to tokens and AST nodes
4. **Comment support** — teach the lexer to skip `//` and `/* */`
5. **Backend parity tests** — run the same test suite through both backends

### Medium Term

6. **Float support on WASM** — choose a boxing strategy and implement it
7. **Short-circuit on bytecode** — compile `&&`/`||` as conditional jumps
8. **Shared lowering pass** — desugar `for-in`, compound assignment, and other sugar once
9. **Better error messages** — structured diagnostics with source context
10. **Tail calls** — actually implement `TAILCALL` in the VM

### Long Term

11. **Garbage collection** — mark-and-sweep over the WASM heap
12. **Optimization passes** — constant folding, dead code elimination, inlining
13. **Module system** — multi-file programs with imports
14. **Type system** — gradual typing or full static analysis
15. **JIT compilation** — profile hot paths in the VM and compile them to native code

Each of these is a course in itself. But having built the foundation, none of them feel impossible.

---

## What I Tell People Who Want to Build a Compiler

### Start Stupid

My first lexer was a giant `match` statement. My first parser was recursive descent with no error recovery. My first VM was a loop over a `Vec`. None of this was clever. All of it worked.

### Make It Observable

The most valuable code in Rustphy is not the cleverest. It is the `serde::Serialize` derive on the AST and the `println!` in the VM debug mode. If you can see what your compiler is doing, you can fix it. If you can't, you are guessing.

### Let Features Drive Architecture

Don't design the perfect compiler upfront. Build the simplest thing that runs `print(1 + 2)`. Then add functions. Then closures. Then a second backend. Each feature will force you to generalize the design, and you will understand *why* the generalization exists because you felt the pain of not having it.

### Read Other People's Compilers

I learned more from reading the source of [Crafting Interpreters](https://craftinginterpreters.com/), the [Monkey interpreter](https://interpreterbook.com/), and the MIT 6.035 lectures than from any single tutorial. Rustphy borrows ideas from all of them.

### Write About It

This series exists because writing about my design choices forced me to understand them. The gaps I found while writing — the float stub, the short-circuit inconsistency, the hardcoded closure table — are all things I discovered by trying to explain the code to someone else.

---

## The Honest Ending

Rustphy is not a production language. It has bugs I know about and bugs I don't. The two backends disagree on semantics. The WASM heap has no garbage collector. The error messages say "expected token" without telling you where.

But it compiles. It runs. It has closures, recursion, first-class functions, and a browser playground. It taught me more about computer science than any course I have taken.

And now it can teach you.

Go build something.

---

## Final Quiz

1. Why did `for-in` require changes to the symbol table?
2. Why did the bytecode VM not need changes for `for-in`?
3. What is the advantage of desugaring in the backend?
4. What is the disadvantage of desugaring in both backends independently?
5. Why is observability more important than cleverness in a learning compiler?
6. What is the most impactful single improvement to make next?

<details>
<summary><strong>Answers</strong></summary>

1. The loop variable is a new binding that needs scope tracking, resolution, and capture analysis.
2. `for-in` was fully desugared into instructions the VM already understood (`LDS`, `LDCN`, `BINOP`, `ASSIGN`, `JOF`, `GOTO`).
3. The runtime stays small — it only needs to understand primitive operations, not syntactic sugar.
4. The same lowering logic is duplicated, and the two implementations can diverge (as they did with short-circuit evaluation).
5. You cannot debug what you cannot see. JSON AST output, bytecode disassembly, and WAT text output turn every bug into a phase-localization problem.
6. Source spans. They unlock useful error messages, which unlock faster debugging, which accelerates every future feature.

</details>

---

## One More Thing

If you followed this series, you now understand:

- How source text becomes tokens
- How tokens become trees
- How trees become instructions
- How instructions become running programs
- How closures capture their environment
- How escape analysis decides where variables live
- How a dynamic language maps onto a statically typed instruction set
- How a compiler runs in a browser

That is not a small thing.

Now go build your own. Start stupid. Make it observable. Let features drive the design. And write about what you learn.

---

[← Part 6 — Compiling to WebAssembly](part_06_webassembly.md) | [Course index](index.md)

---

*Thank you for reading. If you found this helpful, star the [repository](https://github.com/sixletters/rustphy) or share it with someone learning compilers.*

*— Harris*
