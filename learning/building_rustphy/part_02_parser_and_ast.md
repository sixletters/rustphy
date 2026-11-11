---
layout: default
title: Part 2 - Giving the Language Structure
---

# Part 2: Giving the Language Structure

**Building Rustphy: A Compiler From Scratch**

---

## Tokens Are Words, Not Sentences

In the previous part, we turned this:

```javascript
let result = 1 + 2 * 3;
```

into this:

```text
Let Ident("result") Assign Int("1") Plus Int("2") Asterisk Int("3") Semicolon
```

That token stream is correct, but it still does not tell us the structure of the expression.

These two trees use the same tokens:

```text
    +                    *
   / \                  / \
  1   *                +   3
     / \              / \
    2   3            1   2

1 + (2 * 3)          (1 + 2) * 3
```

Only the first tree matches normal arithmetic precedence.

The parser's job is to transform a flat token stream into a tree that makes those relationships explicit.

---

## Step 1: Design the Tree Around Meaning

Rustphy separates statements from expressions.

An **expression** produces a value:

```javascript
42
x + 1
func(a) { return a; }
condition ? left : right
```

A **statement** controls evaluation or introduces a binding:

```javascript
let x = 42;
return x;
break;
```

Some language constructs blur the boundary. Rustphy's `if` is represented as an expression, while a `for` loop is a statement. Assignment is also an expression.

That decision is encoded in two enums.

### Statement Nodes

Conceptually, `StatementNode` contains:

```rust
pub enum StatementNode {
    Program { statements: Vec<Box<Node>>, /* ... */ },
    Let { name: ExpressionNode, value: ExpressionNode, /* ... */ },
    Return { return_value: ExpressionNode, /* ... */ },
    Expression { expression: ExpressionNode, /* ... */ },
    Block { statements: Vec<Box<StatementNode>>, /* ... */ },
    For { condition: ExpressionNode, for_block: Box<StatementNode>, /* ... */ },
    FuncDeclr { identifier: ExpressionNode, func: ExpressionNode, /* ... */ },
    Break { /* ... */ },
    Continue { /* ... */ },
    ForIn { variable: ExpressionNode, iterable: Box<ExpressionNode>, body: Box<StatementNode>, /* ... */ },
}
```

### Expression Nodes

`ExpressionNode` includes:

```text
Identifier   Integer   Float   String   Boolean
Prefix       Infix     Ternary
If           Function Call
Array        HashMap  Index
Range
```

This is the first major design benefit of an AST: later phases can match on **language concepts**, not punctuation.

The bytecode compiler does not need to ask whether it just saw `let`, an identifier, `=`, and an expression. It receives a `Let` node.

---

## Step 2: Use `Box` for Recursive Nodes

An AST is recursive. An infix expression contains expressions, which may contain more infix expressions forever.

This cannot be stored directly:

```rust
// Conceptually impossible: infinite size
pub enum Expression {
    Integer(i64),
    Infix {
        left: Expression,
        right: Expression,
    },
}
```

To know the size of `Expression`, Rust would need to know the size of `Expression` inside it, which requires knowing the size of `Expression`, and so on.

`Box<T>` solves this by storing a fixed-size pointer in the enum:

```rust
Prefix {
    token: Token,
    operator: PrefixOp,
    right: Box<ExpressionNode>,
    id: i32,
},

Infix {
    token: Token,
    left: Box<ExpressionNode>,
    operator: InfixOp,
    right: Box<ExpressionNode>,
    id: i32,
},
```

Now every recursive edge has a known pointer size.

Rustphy also boxes elements inside vectors, even where `Vec<T>` already provides indirection. That is not always necessary, but it keeps ownership and movement of node elements simple.

---

## Step 3: Keep Operator Meaning Separate From Token Spelling

Tokens tell us what characters appeared:

```rust
Token::Plus
Token::Minus
Token::Asterisk
```

The AST uses semantic operator enums:

```rust
pub enum PrefixOp {
    Negative,
    Not,
}

pub enum InfixOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Lt,
    Gt,
    Eq,
    NotEq,
    And,
    Or,
    Assign,
    In,
}
```

That separation matters because multiple token spellings can lower into the same semantic structure.

For example, `+=` does not survive as a special AST operator. The parser desugars it into ordinary assignment plus addition.

```text
x += 5
  ↓
x = x + 5
```

The AST only needs `Assign` and `Add`.

---

## Step 4: Give Every Node an Identity

Most Rustphy AST nodes carry an `id: i32`. The parser assigns IDs monotonically with `consume_id()`.

At first, IDs may look like debug metadata. They become important later when the symbol table needs to record:

```text
identifier-use node ID → declaration binding ID
```

Consider:

```javascript
let x = 1;
{
    let x = 2;
    print(x);
};
print(x);
```

The spelling `x` is not enough to identify a variable. Each use needs to resolve to the correct declaration according to lexical scope. A stable node ID lets a side table store that resolution without mutating the AST.

The current implementation has a couple of synthesized nodes that use ID `0`, and compound-assignment desugaring clones node IDs. We will discuss why true uniqueness matters in Part 5.

---

## Step 5: Serialize the Tree

Tokens and AST enums derive `serde::Serialize`, so Rustphy can print the tree as JSON.

That is one of the highest-leverage features in the project.

```bash
cargo run --bin rustphy-cli -- parse examples/03_functions.gph
```

Why serialize the AST?

- You can verify parser structure before debugging code generation.
- Browser tools can display it.
- Tests can compare structure.
- Node IDs and copied tokens become visible.
- The AST becomes a public boundary rather than an invisible implementation detail.

A parser that only feeds the next compiler stage is much harder to understand.

---

## Step 6: Maintain a Two-Token Cursor

The parser stores:

```rust
pub struct Parser {
    lexer: Lexer,
    curr_token: Token,
    peek_token: Token,
    next_node_id: i32,
}
```

Construction starts both token slots at EOF and calls `consume_token()` twice. After that:

```text
curr_token = first real token
peek_token = second real token
```

Most parsing decisions inspect `curr_token`. `peek_token` behaves as a buffered next token, and consuming advances both slots.

The parser follows an important contract:

> A parse function consumes its own construct and returns with `curr_token` pointing at the first token that does not belong to that construct.

Parser bugs often come from violating this contract by one token.

---

## Step 7: Parse Statements by Their First Token

Statement parsing is a recursive-descent dispatch:

```text
let       → parse variable declaration
return    → parse return statement
break     → parse break
continue  → parse continue
for       → parse loop
func      → parse named function declaration
{         → parse block
anything else → parse expression statement
```

This is where context resolves ambiguity.

### `{` Can Mean Two Things

At statement position:

```javascript
{
    let x = 1;
};
```

`{` begins a block.

At expression position:

```javascript
let point = { "x": 10, "y": 20 };
```

`{` begins a hashmap literal.

The token is the same. The parser knows what it means because of where parsing began.

### `func` Can Mean Two Things

At statement position, a leading `func` is parsed as a named declaration:

```javascript
func add(a, b) {
    return a + b;
};
```

At expression position, it can be anonymous:

```javascript
let add = func(a, b) {
    return a + b;
};
```

This context-sensitive distinction is implemented in statement dispatch versus prefix-expression parsing.

---

## Step 8: Use Pratt Parsing for Expressions

Recursive descent is straightforward for statements because each form usually begins with a distinctive keyword.

Expressions are harder:

```javascript
-a.b(1)[0] + 2 * 3 == 7 && active ? yes : no
```

We need to handle:

- Prefix operators
- Binary operators at many precedence levels
- Calls
- Indexing
- Member access
- Assignment
- Ternary conditionals
- Parentheses

A Pratt parser solves this with two ideas:

1. Parse something that can **start** an expression.
2. While the next operator binds more tightly than the surrounding context, extend the expression.

Rustphy's core is remarkably small:

```rust
let mut left_exp = self.parse_prefix()?;

while !matches!(self.curr_token, Token::Semicolon)
    && precedence < self.get_precedence(&self.curr_token)
{
    if matches!(self.curr_token, Token::Conditional) {
        left_exp = self.parse_mixfix(&left_exp)?;
    } else {
        left_exp = self.parse_infix(&left_exp)?;
    }
}

Ok(left_exp)
```

That loop is the engine of the expression grammar.

---

## Step 9: Encode Binding Power in an Enum

Rustphy defines precedence in ascending order:

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Precedence {
    Lowest = 1,
    TERNARY,
    ASSIGN,
    LOGICAL,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    POSTFIX,
    CALL,
}
```

Because the enum derives `Ord`, declaration order becomes the binding-power table.

From weakest to strongest:

| Precedence | Operators/forms |
|---|---|
| Ternary | `? :` |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=` |
| Logical | `&&`, `||` |
| Equality | `==`, `!=` |
| Comparison | `<`, `>`, `in` |
| Sum/range | `+`, `-`, `..`, `..=` |
| Product | `*`, `/` |
| Prefix | `-x`, `!x` |
| Call | `f()`, `a[i]`, `a.b` |

### Trace: `1 + 2 * 3`

Start at `Lowest`:

1. Parse prefix `1`.
2. Current `+` has higher precedence than `Lowest`, so extend.
3. To parse the right side of `+`, recurse with `SUM` precedence.
4. Parse prefix `2`.
5. Current `*` has higher precedence than `SUM`, so extend again.
6. Parse `3` as the right side of `*`.
7. Return `2 * 3` to the `+` parse.

Result:

```text
Infix Add
├── Integer 1
└── Infix Multiply
    ├── Integer 2
    └── Integer 3
```

No separate function such as `parse_multiplication_expression` is required. Binding power controls recursion.

---

## Step 10: Prefix Parsing Creates the First Left-Hand Side

`parse_prefix()` handles anything that can begin an expression:

```text
Identifier      x
Integer         42
Float           3.14
String          "hello"
Boolean         true
Prefix          -x, !x
Grouping        (x + y)
Array           [1, 2, 3]
Hashmap         {"x": 1}
If              if (condition) {...}
Function        func(a) {...}
```

The name “prefix” can be confusing. It includes prefix operators, but it really means **null denotation**: what does this token mean when there is no expression to its left?

For `-`, the answer is unary negation.

For `42`, the answer is an integer literal.

For `(`, the answer is a grouped expression or the start of a function-call continuation depending on context.

---

## Step 11: Infix Parsing Extends an Existing Expression

Once there is a left expression, `parse_infix()` handles forms that follow it:

```text
left + right
left == right
left = right
left(args)
left[index]
left.field
left..right
```

Calls, indexing, and member access all have the highest `CALL` precedence, so they chain naturally:

```javascript
factory()(x).items[0]
```

Each continuation wraps the tree built so far.

---

## Step 12: Lower Member Access Into Indexing

Rustphy does not have a separate `MemberAccess` AST node.

The parser lowers:

```javascript
user.name
```

into the same tree shape as:

```javascript
user["name"]
```

Conceptually:

```text
Index
├── object: Identifier("user")
└── index:  String("name")
```

This is a smart reduction in backend complexity. Arrays and objects can share subscript machinery, and every later phase handles one access form.

It does mean pretty-printing the AST may not reproduce the original dot syntax. That is acceptable because an AST preserves meaning, not exact formatting.

---

## Step 13: Desugar Compound Assignment

The parser turns:

```javascript
x += 5
```

into:

```javascript
x = x + 5
```

The implementation clones the left expression into the right-hand operation:

```rust
let right_exp = match desugar_op {
    Some(infix_op) => ExpressionNode::Infix {
        token: curr_token.clone(),
        left: Box::new(left_exp.clone()),
        operator: infix_op,
        right: Box::new(self.parse_expression(current_precedence)?),
        id: self.consume_id(),
    },
    None => self.parse_expression(current_precedence)?,
};
```

This reduces the number of operations later phases must understand.

### The Semantic Trap

What about:

```javascript
array[next_index()] += 1
```

Naive desugaring gives:

```javascript
array[next_index()] = array[next_index()] + 1
```

Now `next_index()` runs twice.

This demonstrates an important rule:

> Desugaring must preserve evaluation count and order, not only produce a similar-looking expression.

A robust lowering would evaluate the lvalue location once, perhaps using temporaries or a dedicated read-modify-write representation.

Rustphy's current design is fine for simple identifiers but reveals where semantic analysis should validate and normalize assignable locations.

---

## Step 14: Parse `for-in` Through Ordinary Expressions

Rustphy parses the contents of `for (...)` as an expression.

For:

```javascript
for (i in 0..10) {
    print(i);
};
```

the Pratt parser uses precedence to build:

```text
Infix In
├── Identifier("i")
└── Range
    ├── Integer(0)
    ├── Integer(10)
    └── inclusive: false
```

Why does this group correctly?

- `in` has comparison precedence.
- `..` has sum precedence, which binds more tightly.

So the right side becomes the complete range before the `In` expression finishes.

The statement parser then inspects the root. If it is `In`, the parser creates a `ForIn` statement. Otherwise it creates a regular `For` statement.

This is a nice example of letting the general expression parser do most of the work.

---

## Step 15: Ternary Is Mixfix

A binary operator has one fixed piece between two operands:

```text
left + right
```

A ternary has two fixed pieces surrounding two later operands:

```text
condition ? then_value : else_value
```

Rustphy handles this in `parse_mixfix()`. After the condition has already been parsed:

1. Consume `?`.
2. Parse the then expression.
3. Require `:`.
4. Parse the else expression.
5. Build `Ternary { condition, then, else }`.

The current missing-colon path panics rather than returning `Err`, which is another diagnostic boundary to improve.

---

## Associativity Is Hidden in One Comparison

The Pratt loop continues when:

```rust
current_context_precedence < next_operator_precedence
```

Right-hand operands are parsed using the current operator's precedence. Equal-precedence operators therefore stop the recursive right side and associate left:

```text
10 - 3 - 2
→ (10 - 3) - 2
```

This is correct for subtraction.

But assignment is usually right-associative:

```text
a = b = 1
→ a = (b = 1)
```

Rustphy's generic rule currently makes equal-precedence assignment left-associative. A production Pratt parser often assigns separate left and right binding powers or special-cases right-associative operators.

The lesson is subtle: precedence decides **which operator binds first**; associativity decides **how equal-precedence operators group**.

---

## Semicolons and Implicit Returns

Rustphy's statement parser currently requires a semicolon whenever `consume_semicolon` is true:

```rust
if consume_semicolon {
    if !matches!(self.curr_token, Token::Semicolon) {
        return Err(String::from(
            "; expected at the end of a statement"
        ));
    }
    self.consume_token();
}
```

Program and block parsing use this mode, so even control-flow constructs and final block expressions need terminators.

The AST has `implicit_return` fields on programs and blocks, but the parser currently leaves them as `None`.

That is a useful example of the AST anticipating a semantic feature that the grammar does not yet produce.

---

## Error Handling: Parsing Is Also Diagnostics

Most parser functions return `Result<_, String>` and stop at the first error.

This is enough to build the compiler, but not enough for a friendly language. Current limitations include:

- No source spans
- No error accumulation
- No synchronization after a bad statement
- Some panic paths
- Unterminated blocks can be handled inconsistently
- `parse_program_expression` does not require trailing EOF
- Parameters and assignment targets are not fully validated
- `break`, `continue`, and `return` placement is not statically checked

A mature frontend usually separates three concerns:

1. **Parsing:** Can these tokens form the grammar?
2. **AST validation:** Are structural restrictions satisfied?
3. **Semantic analysis:** Do names, types, and contexts make sense?

Rustphy currently performs some of these checks late or not at all. That gives us a clear roadmap.

---

## Full Trace: From Tokens to Tree

Input:

```javascript
let answer = add(20, 2 * 11);
```

Tokens:

```text
Let
Ident("answer")
Assign
Ident("add")
LParen
Int("20")
Comma
Int("2")
Asterisk
Int("11")
RParen
Semicolon
```

AST:

```text
Let
├── name: Identifier("answer")
└── value: Call
    ├── function: Identifier("add")
    └── arguments
        ├── Integer(20)
        └── Infix(Multiply)
            ├── Integer(2)
            └── Integer(11)
```

The parser discarded commas, parentheses, and the assignment punctuation around `let`. Those tokens did their job by establishing structure. The AST retains the meaning needed downstream.

That is why it is called an **abstract** syntax tree.

---

## What I Would Improve Next

### 1. Make Node IDs Truly Unique

Synthesized nodes and cloned subtrees should receive fresh IDs or use stable arena keys. Side tables depend on identity meaning one occurrence.

### 2. Separate Parse Errors From Validation Errors

Introduce typed diagnostics:

```rust
pub enum ParseErrorKind {
    ExpectedToken { expected: TokenKind, found: TokenKind },
    UnexpectedToken(TokenKind),
    UnterminatedBlock,
}
```

Then add an AST validation pass for lvalues, parameter forms, and contextual statements.

### 3. Fix Associativity Explicitly

Give operators left and right binding powers. Assignment and ternary can then be right-associative without disturbing arithmetic.

### 4. Preserve Compound-Assignment Semantics

Represent an lvalue or lower through temporaries so side-effecting targets are evaluated exactly once.

### 5. Decide Whether Blocks Produce Values

If implicit returns are desired, define and parse them consistently. If not, remove the unused AST fields until the feature exists.

---

## Checkpoint

We can now transform source into a semantic tree:

```text
Characters
  ↓ lexer
Tokens
  ↓ recursive descent + Pratt parser
AST
```

The AST knows that multiplication binds tighter than addition. It knows a function call's callee and arguments. It knows a range is inclusive or exclusive.

But the AST is not executable.

In the next part, we will lower tree structure into a linear instruction stream. An `if` tree will become conditional and unconditional jumps. A function expression will become an address. And because forward jump targets do not exist yet, we will learn how to patch the past.

---

## Quiz

1. Why do recursive enum fields need indirection such as `Box`?
2. What is the difference between a token operator and an AST operator?
3. What are the two main stages of a Pratt expression parser?
4. Why do calls bind more tightly than arithmetic?
5. How does Rustphy make `i in 0..10` group correctly?
6. What semantic bug can naive compound-assignment desugaring introduce?
7. What is the difference between precedence and associativity?

<details>
<summary><strong>Answers</strong></summary>

1. Without indirection, the recursive type would have infinite size.
2. A token records source spelling; an AST operator records semantic meaning after syntax has been interpreted.
3. Parse a prefix/start form, then repeatedly extend it while the current continuation has sufficient binding power.
4. So `f(x) + 1` groups as `(f(x)) + 1`; otherwise arithmetic could incorrectly enter the call target or arguments.
5. Range precedence is higher than `in`, so the full range becomes the right operand.
6. A side-effecting lvalue can be evaluated twice.
7. Precedence orders different operators; associativity groups operators with equal precedence.

</details>

---

## Practice

### Challenge 1: Add Modulo

Add `%` from token through AST parsing. Decide where it belongs in the precedence ladder and explain why.

### Challenge 2: Right-Associative Assignment

Change the Pratt parser so:

```javascript
a = b = 3;
```

becomes:

```text
a = (b = 3)
```

Add tests showing subtraction remains left-associative.

### Challenge 3: Validate Assignment Targets

Reject:

```javascript
(1 + 2) = 3;
```

Accept identifiers and index expressions. Decide whether validation belongs inside parsing or in a separate pass.

### Challenge 4: AST Explorer

Run the CLI parser over examples containing calls, indexes, ternaries, ranges, and closures. Draw the tree before looking at JSON, then compare your answer.

---

[← Part 1 — Teaching the Compiler to Read](part_01_lexer.md) | [Course index](index.md) | [Part 3 — From Trees to Bytecode →](part_03_bytecode_compiler.md)
