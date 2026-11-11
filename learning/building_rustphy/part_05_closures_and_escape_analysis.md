---
layout: default
title: Part 5 - Closures, Symbol Tables, and Escape Analysis
---

# Part 5: Closures Force Us to Think Like Compiler Engineers

**Building Rustphy: A Compiler From Scratch**

---

## The Bytecode VM Could Delay the Question

When the bytecode VM reads `x`, it looks through environment hashmaps at runtime:

```text
current scope → parent scope → parent scope → ... → global
```

That means the bytecode compiler can emit:

```text
LDS "x"
```

without deciding which `x` declaration it means.

WebAssembly changes the problem. A fast local read looks like:

```wat
local.get $x
```

But consider shadowing:

```javascript
let x = 1;
{
    let x = 2;
    print(x);
};
print(x);
```

There are two source names but two distinct bindings. And closures make storage even harder:

```javascript
let outer = func() {
    let x = 1;
    return func() {
        return x;
    };
};
```

The inner function needs `x` after the outer call may have returned. A normal WASM local belongs to the outer activation and cannot simply be referenced by another function later.

Before code generation, we need two analyses:

1. **Name resolution:** Which declaration does every identifier use refer to?
2. **Escape analysis:** Which bindings must outlive ordinary local storage because a nested function captures them?

---

## Names Are Not Bindings

This distinction is foundational.

A **name** is source text such as `x`.

A **binding** is one specific declaration of that name.

```javascript
let x = 1;        // binding A
{
    let x = 2;    // binding B
    print(x);     // use resolves to B
};
print(x);         // use resolves to A
```

If the compiler keys everything by string alone, both declarations collide. A symbol table gives each declaration a unique identity.

---

## Step 1: Represent Lexical Scopes

Rustphy's symbol table stores an arena of scopes. Each scope has an integer ID:

```rust
pub struct Scope {
    id: usize,
    parent: Option<usize>,
    symbols: HashMap<String, Symbol>,
    scope_type: ScopeType,
    children: Vec<usize>,
}
```

Scope kinds are:

```rust
pub enum ScopeType {
    Program,
    Function,
    Block,
}
```

The arena representation looks like:

```text
scopes[0] Program
├── scopes[1] Function outer
│   ├── scopes[2] Block
│   └── scopes[3] Function inner
│       └── scopes[4] Block
└── scopes[5] Block
```

Parent and child links are IDs rather than Rust references. This avoids lifetime-heavy cyclic structures and makes scopes easy to serialize or inspect.

---

## Step 2: Give Every Declaration a Binding ID

A symbol records:

```rust
pub struct Symbol {
    name: String,
    binding_id: usize,
    scope_id: usize,
    bind_type: BindType,
}
```

Binding types include:

```text
FunctionParam
FunctionDeclaration
VariableDeclaration
```

Every declaration receives a globally unique `binding_id`.

Now the compiler can distinguish:

```text
name "x", binding 7, scope 1
name "x", binding 12, scope 4
```

even though both use the same spelling.

This is analogous to giving each variable declaration a database primary key.

---

## Step 3: Build Side Tables Instead of Mutating the AST

Rustphy's symbol table maintains several maps:

```text
resolved:       AST identifier node ID → binding ID
bindings:       binding ID → Symbol
scope_bindings: scope ID → [binding IDs]
node_scope:     AST node ID → scope ID
```

Why use side tables?

The AST remains a representation of source structure. Analysis results can evolve independently and multiple analyses can attach facts to the same nodes.

Given an identifier node with ID `83`, later code can ask:

```text
resolve(83) → binding 12
get_symbol(12) → { name: "x", scope_id: 4, ... }
```

This pattern appears throughout real compilers:

```text
AST/IR node identity + analysis result table
```

It does depend on node IDs being unique. Synthesized ID `0` nodes and cloned IDs in the current parser are therefore more than cosmetic bugs—they can corrupt analysis associations.

---

## Step 4: Walk the AST and Maintain Current Scope

Building the symbol table is another tree walk.

At a declaration:

```javascript
let x = expression;
```

we:

1. Create a binding for `x` in the current scope.
2. Record its binding and scope.
3. Walk the initializer expression.

At a block:

1. Create a `Block` scope.
2. Set it as current.
3. Walk statements.
4. Restore the parent scope.

At a function:

1. Create a `Function` scope.
2. Declare each parameter in it.
3. Walk the body and nested block scopes.
4. Restore the parent.

The order of declaration versus initializer traversal determines whether a variable can refer to itself and how temporal-dead-zone semantics work. A language must choose this deliberately.

---

## Step 5: Resolve Identifier Uses by Walking Parents

When the builder encounters an identifier expression, it searches:

```text
current scope symbols
    ↓ if absent
parent scope symbols
    ↓ if absent
parent's parent
    ↓
error/unresolved
```

This is compile-time lexical lookup. It mirrors the bytecode VM's runtime environment lookup, but produces a permanent binding decision.

For:

```javascript
let x = 1;
let f = func(y) {
    let z = x + y;
    return z;
};
```

resolution is:

```text
use x → global variable binding
use y → function parameter binding
use z → function-local variable binding
```

After resolution, code generation no longer needs to ask “which `x`?” It asks “where is binding 3 stored?”

---

## Step 6: Collect a Function's Own Bindings

The WASM compiler needs to declare locals for one function, including variables in nested blocks, but excluding variables belonging to nested functions.

Rustphy recursively collects scope bindings with this rule:

```rust
match scope_type {
    ScopeType::Block => {
        bindings.extend(
            self.collect_bindings_recursively(child_scope_id)
        );
    }
    ScopeType::Function => {
        // Stop: these bindings belong to the nested function.
    }
    ScopeType::Program => {}
}
```

This boundary is easy to miss.

Blocks are lexical scopes inside the same function activation. Nested functions are separate activations with their own locals.

---

## The Closure Storage Problem

Consider:

```javascript
let make_adder = func(x) {
    let local_only = 100;

    return func(y) {
        return x + y;
    };
};
```

Two bindings live inside `make_adder`:

```text
x          captured by nested function
local_only never captured
```

If we put both in heap environments, the program works but every local access pays extra allocation and indirection.

If we put both in WASM locals, `x` becomes inaccessible after `make_adder` returns.

The best storage split is:

```text
local_only → WASM local
x          → heap environment slot
```

Escape analysis computes that split.

---

## Step 7: Define Escape for Rustphy

In this compiler, a binding escapes when a nested function refers to a declaration from an outer scope.

The analysis walks the AST while maintaining a stack of current function scope IDs.

At a function node:

```text
push function scope ID
walk body
pop function scope ID
```

At an identifier use:

1. Resolve its binding ID through the symbol table.
2. Retrieve the binding's declaration scope.
3. Compare it with the current function scope.
4. If the declaration is outside the current function, mark it escaping.

The core rule is:

```rust
if let Some(binding_id) = self.symbol_table.resolve(node_id) {
    if let Some(symbol) = self.symbol_table.get_symbol(binding_id) {
        if let Some(&current_func_scope) = self.function_scope_stack.last() {
            if symbol.scope_id < current_func_scope {
                self.escapes.insert(binding_id, true);
            }
        }
    }
}
```

Scope IDs are assigned in traversal order, so an outer scope has a lower ID than its nested function scope.

---

## Is That Analysis Precise?

It is conservative and tailored to the current scope numbering.

The analysis marks a binding as escaping whenever a nested function captures it, even if the closure is called immediately and never outlives the outer function:

```javascript
let outer = func() {
    let x = 1;
    let inner = func() { return x; };
    return inner();
};
```

A more advanced analysis could prove that `inner` does not escape and keep `x` in a different representation. Rustphy chooses the safer rule:

```text
captured by nested function → heap environment
```

That may allocate more than necessary, but it preserves correctness.

This is the right direction for early compiler analysis: **sound before optimal**.

### A Fragile Comparison

Using `symbol.scope_id < current_func_scope` assumes numeric scope order corresponds to lexical ancestry.

A more robust test would explicitly walk from the current function scope through parent links and determine whether the declaration belongs outside that function boundary. IDs should identify scopes, not encode semantic ancestry by magnitude.

---

## Step 8: Split Bindings Into Locals and Environment Slots

The WASM compiler asks for all bindings owned by a function, then classifies them:

```text
Variable declaration + does not escape → WASM local
Escaping binding                       → environment slot
Function declarations/params           → handled by calling/storage rules
```

The compilation context records:

- Which binding IDs map to local names
- Which binding IDs map to environment indices
- The current environment depth
- Parent compilation contexts

A read then chooses one of two paths.

### Non-Escaping Read

```wat
local.get $local_name
```

### Escaping Read

```wat
;; Start from current environment pointer.
local.get $env_ptr

;; Walk parent links to declaration environment if needed.
i32.load
...

;; Load binding slot.
i32.const slot_index
call $env_get
```

Writes make the same decision using `local.set` or `$env_set`.

The symbol table answers **which binding**. Escape analysis answers **which storage class**. Code generation answers **which instructions**.

---

## Step 9: Understand Environment Depth

Suppose:

```javascript
let outer = func(a) {
    let middle = func(b) {
        let inner = func(c) {
            return a + b + c;
        };
        return inner;
    };
    return middle;
};
```

`inner` captures bindings from two outer function environments:

```text
inner env:  c
   │ parent
middle env: b
   │ parent
outer env:  a
```

To load `a`, generated code follows two parent pointers. To load `b`, it follows one. `c` may be a direct parameter/local depending on representation.

This is why environment frames store a parent pointer. Closures form a runtime chain mirroring lexical nesting.

---

## Step 10: Compare the Two Backends

The same source binding takes two very different routes.

### Bytecode VM

Compilation:

```text
LDS "x"
```

Runtime:

```text
hash "x" in current env
if missing, follow parent
repeat
```

Capture:

```text
closure stores entire current Environment Rc
```

### WASM Backend

Compilation:

```text
identifier node → binding ID → storage class
```

Runtime local:

```wat
local.get $x
```

Runtime captured binding:

```wat
local.get $env_ptr
...parent traversal...
i32.const slot
call $env_get
```

Capture:

```text
closure object stores environment pointer
```

The bytecode backend pays lookup cost dynamically. The WASM backend pays analysis complexity statically.

Neither is “the compiler way” in isolation. They are different tradeoffs for different execution models.

---

## Trace: Resolving and Capturing `value`

Return to our counter:

```javascript
let make_counter = func(start) {
    let value = start;
    return func() {
        value = value + 1;
        return value;
    };
};
```

### Symbol Table

```text
Binding 1: make_counter, Program scope, FunctionDeclaration
Binding 2: start, Function scope A, FunctionParam
Binding 3: value, Function scope A, VariableDeclaration

Inner function uses:
node 41 "value" → binding 3
node 42 "value" → binding 3
node 47 "value" → binding 3
```

### Escape Analysis

Current function while visiting those uses: function scope B.

Declaration scope of binding 3: function scope A.

A is outside B, so:

```text
escapes[3] = true
```

### WASM Storage

Instead of:

```wat
(local $value i32)
```

`value` receives an environment slot:

```text
environment A
[parent_ptr, var_count, value_slot, ...]
```

The inner closure stores a pointer to environment A. Every call reads and writes the same slot.

That is the static-analysis-to-runtime bridge.

---

## Why This Is Called Escape Analysis

In general, escape analysis asks whether a value can outlive or move beyond the context where it was created.

Examples:

- A local referenced by a returned closure escapes its function activation.
- An object stored in a global may escape the allocating function.
- A value passed to unknown code may escape.
- A closure invoked immediately might not escape.

Compilers use escape information for:

- Stack versus heap allocation
- Scalar replacement
- Synchronization elimination
- Closure environment layout
- Lifetime optimization

Rustphy implements the closure-capture subset because that is exactly what its WASM storage decision needs.

Good compiler passes answer a concrete downstream question.

---

## What About Capturing the Whole Environment?

The bytecode VM captures an `Rc` to the entire current environment. The WASM backend assigns environment slots to escaping bindings.

A more advanced compiler could compute each function's exact **free-variable set**:

```text
free_vars(inner) = { value }
```

Then construct a closure environment containing only those cells.

Benefits:

- Smaller closures
- Less accidental retention
- Faster parent access
- Clearer closure conversion

Costs:

- More analysis
- Environment layout per closure/function
- Shared mutable captures may need boxed cells
- Recursive/mutually recursive closures complicate construction

Rustphy's environment-chain design is simpler and preserves lexical sharing.

---

## Static Errors We Can Now Detect

Once a symbol table exists, the compiler can report problems before execution:

```javascript
print(missing);  // unresolved identifier
```

It can also validate:

- Duplicate declarations in one scope
- Illegal shadowing rules, if the language has them
- `return` outside a function
- `break` or `continue` outside a loop
- Wrong function declaration/parameter forms

Rustphy does not yet implement all these diagnostics, but the symbol table provides the structural foundation.

---

## What I Would Improve Next

### 1. Use Structural Ancestry, Not Numeric ID Ordering

Determine capture by walking scope parents and function boundaries.

### 2. Make AST Identity Reliable

Allocate unique node IDs for every synthesized occurrence. Never clone identity when cloning structure.

### 3. Compute Free Variables Per Function

Record exactly which outer bindings each function needs. Use that to define compact environment layouts.

### 4. Separate Captured-By-Value and Captured-By-Cell

Immutable captures can copy values. Mutable shared captures need a common cell.

### 5. Add Semantic Diagnostics

Make unresolved names, invalid contextual statements, and duplicate declarations compile-time errors with spans.

### 6. Build One Symbol Table

The current WASM compiler constructor builds symbol-table state redundantly for resolution and escape analysis. Share one completed analysis result.

---

## Checkpoint

We can now answer two questions for every identifier:

```text
Which declaration? → binding ID
Where stored?      → WASM local or environment slot
```

That is enough to generate code without carrying source names into runtime lookup.

Next we enter the WASM backend, where every Rustphy value must fit into an `i32`, closures need an indirect-call table, loops must become structured branches, and strings must live in linear memory.

---

## Quiz

1. What is the difference between a name and a binding?
2. Why does each identifier use need a unique AST identity?
3. Why does binding collection recurse through blocks but stop at nested functions?
4. What makes a Rustphy binding escape?
5. Why can a non-escaping binding use a WASM local?
6. Why is capture-based escape analysis conservative?
7. How do symbol resolution and escape analysis divide responsibilities?

<details>
<summary><strong>Answers</strong></summary>

1. A name is source text; a binding is one particular declaration with scope and identity.
2. Resolution side tables map each occurrence to the declaration it references.
3. Blocks belong to the same function activation; nested functions own separate locals and activations.
4. A nested function references a binding declared outside that function.
5. No closure needs the value after or outside its owning activation.
6. It marks every captured binding as escaping even when the closure can be proved not to outlive the outer call.
7. Resolution identifies the declaration; escape analysis chooses whether that declaration needs closure-reachable storage.

</details>

---

## Practice

### Challenge 1: Print the Scope Tree

For a nested program, display each scope, parent, child, declaration, and resolved identifier use.

### Challenge 2: Remove the Numeric-ID Assumption

Rewrite escape detection using explicit lexical ancestry. Add a test whose scope creation order would break a simplistic numeric comparison.

### Challenge 3: Free-Variable Analysis

Compute a set of free binding IDs for every function. Verify the counter's inner function has `{ value }`.

### Challenge 4: Compile-Time Undefined Names

Reject unresolved identifiers before either backend begins code generation. Include the source span and nearest similarly named declaration.

---

[← Part 4 — Building the Virtual Machine](part_04_virtual_machine.md) | [Course index](index.md) | [Part 6 — Compiling to WebAssembly →](part_06_webassembly.md)
