# Rustphy Language Grammar

This document describes the complete grammar of the Rustphy programming language in **Backus-Naur Form (BNF)**.

## Table of Contents

1. [How to Read BNF](#how-to-read-bnf)
2. [Complete Grammar Specification](#complete-grammar-specification)
3. [Grammar Walkthrough](#grammar-walkthrough)
4. [Examples](#examples)

---

## How to Read BNF

### What is BNF?

**Backus-Naur Form (BNF)** is a notation technique for describing the syntax of programming languages. It was developed by John Backus and Peter Naur in the 1960s.

### Basic Symbols and Notation

| Symbol | Meaning | Example |
|--------|---------|---------|
| `::=` | "is defined as" or "produces" | `Expression ::= Term` |
| `\|` | "OR" (alternative choices) | `Value ::= Int \| String` |
| `<>` | Non-terminal (can be expanded) | `<expression>` |
| `""` | Terminal (literal text) | `"let"`, `"+"` |
| `[]` | Optional (0 or 1 occurrence) | `[";"]` |
| `{}` | Repetition (0 or more) | `{Statement}` |
| `()` | Grouping | `("+" \| "-")` |

### Terminals vs Non-Terminals

**Terminals** are the basic symbols/tokens that cannot be broken down further:
- Keywords: `let`, `fn`, `if`, `for`, `return`
- Operators: `+`, `-`, `*`, `/`, `==`, `&&`
- Literals: `42`, `"hello"`, `true`
- Punctuation: `(`, `)`, `{`, `}`, `;`, `,`

**Non-terminals** are abstract concepts that can be expanded into other rules:
- `<Program>`, `<Statement>`, `<Expression>`, `<FunctionCall>`

### Reading Example

```bnf
<IfStatement> ::= "if" "(" <Expression> ")" <BlockStatement> ["else" <BlockStatement>]
```

**How to read this:**
- An `<IfStatement>` **is defined as** (`::=`)
- The keyword `"if"` (terminal, must appear exactly)
- Followed by `"("` (terminal)
- Followed by an `<Expression>` (non-terminal, will be expanded)
- Followed by `")"`
- Followed by a `<BlockStatement>`
- **Optionally** (`[]`) followed by the keyword `"else"` and another `<BlockStatement>`

**Examples that match:**
```javascript
if (x > 5) { print(x); }
if (x > 5) { print(x); } else { print("too small"); }
```

### Precedence and Associativity

In grammar, **operator precedence** is encoded by having separate non-terminals for different precedence levels:

```bnf
<Expression>     ::= <LogicalOr>
<LogicalOr>      ::= <LogicalAnd> {"||" <LogicalAnd>}
<LogicalAnd>     ::= <Equality> {"&&" <Equality>}
<Equality>       ::= <Comparison> {("==" | "!=") <Comparison>}
<Comparison>     ::= <Term> {("<" | ">") <Term>}
<Term>           ::= <Factor> {("+" | "-") <Factor>}
<Factor>         ::= <Unary> {("*" | "/") <Unary>}
```

**How this works:**
- To parse `2 + 3 * 4`, the grammar forces you to:
  1. Start at `<Expression>` (top level)
  2. Drill down through `<LogicalOr>`, `<LogicalAnd>`, `<Equality>`, `<Comparison>` to get to `<Term>`
  3. Parse `2` as a `<Factor>`, see `+`, then parse the rest
  4. When parsing `3`, you drill down to `<Factor>` again
  5. At `<Factor>`, you see `*`, which binds tighter, so `3 * 4` is grouped first
  6. Result: `2 + (3 * 4)` ✓

**Lower in the grammar = higher precedence**

---

## Complete Grammar Specification

### Program Structure

```bnf
<Program>           ::= {<Statement>}

<Statement>         ::= <LetStatement>
                      | <ReturnStatement>
                      | <ExpressionStatement>
                      | <BlockStatement>
                      | <IfStatement>
                      | <ForStatement>
                      | <WhileStatement>
                      | <FunctionStatement>

<BlockStatement>    ::= "{" {<Statement>} "}"
```

### Statements

```bnf
<LetStatement>      ::= "let" <Identifier> "=" <Expression> [";"]

<ReturnStatement>   ::= "return" [<Expression>] [";"]

<ExpressionStatement> ::= <Expression> [";"]

<IfStatement>       ::= "if" "(" <Expression> ")" <BlockStatement>
                        ["else" <BlockStatement>]

<ForStatement>      ::= "for" "(" <ForInit> ";" <Expression> ";" <ForUpdate> ")"
                        <BlockStatement>

<ForInit>           ::= <LetStatement> | <Expression> | ε

<ForUpdate>         ::= <Expression> | ε

<WhileStatement>    ::= "while" "(" <Expression> ")" <BlockStatement>

<FunctionStatement> ::= "fn" <Identifier> "(" [<Parameters>] ")"
                        <BlockStatement>

<Parameters>        ::= <Identifier> {"," <Identifier>}
```

### Expressions (by Precedence)

```bnf
<Expression>        ::= <Assignment>

<Assignment>        ::= <Identifier> "=" <Assignment>
                      | <Identifier> "+=" <Assignment>
                      | <Identifier> "-=" <Assignment>
                      | <Identifier> "*=" <Assignment>
                      | <Identifier> "/=" <Assignment>
                      | <LogicalOr>

<LogicalOr>         ::= <LogicalAnd> {"||" <LogicalAnd>}

<LogicalAnd>        ::= <Equality> {"&&" <Equality>}

<Equality>          ::= <Comparison> {("==" | "!=") <Comparison>}

<Comparison>        ::= <Term> {("<" | ">") <Term>}

<Term>              ::= <Factor> {("+" | "-") <Factor>}

<Factor>            ::= <Unary> {("*" | "/") <Unary>}

<Unary>             ::= ("-" | "!") <Unary>
                      | <Postfix>

<Postfix>           ::= <Primary> {<PostfixOp>}

<PostfixOp>         ::= "(" [<Arguments>] ")"     -- Function call
                      | "[" <Expression> "]"       -- Array indexing
                      | "." <Identifier>            -- Member access

<Primary>           ::= <Integer>
                      | <String>
                      | <Boolean>
                      | <Identifier>
                      | <ArrayLiteral>
                      | <HashLiteral>
                      | <FunctionLiteral>
                      | "(" <Expression> ")"

<Arguments>         ::= <Expression> {"," <Expression>}
```

### Literals

```bnf
<Integer>           ::= ["-"] <Digit> {<Digit>}

<String>            ::= '"' {<Character>} '"'

<Boolean>           ::= "true" | "false"

<ArrayLiteral>      ::= "[" [<ArrayElements>] "]"

<ArrayElements>     ::= <Expression> {"," <Expression>}

<HashLiteral>       ::= "{" [<HashPairs>] "}"

<HashPairs>         ::= <HashPair> {"," <HashPair>}

<HashPair>          ::= <Expression> ":" <Expression>

<FunctionLiteral>   ::= "fn" "(" [<Parameters>] ")" <BlockStatement>

<Identifier>        ::= <Letter> {<Letter> | <Digit> | "_"}

<Digit>             ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

<Letter>            ::= "a" | "b" | ... | "z" | "A" | "B" | ... | "Z"

<Character>         ::= any printable ASCII character except '"'
```

---

## Grammar Walkthrough

### 1. Understanding a Simple Program

**Program:**
```javascript
let x = 5;
print(x);
```

**Parse Tree:**
```
<Program>
├── <Statement> (LetStatement)
│   ├── "let"
│   ├── <Identifier> "x"
│   ├── "="
│   ├── <Expression>
│   │   └── <Primary>
│   │       └── <Integer> "5"
│   └── ";"
└── <Statement> (ExpressionStatement)
    ├── <Expression>
    │   └── <Postfix>
    │       ├── <Primary>
    │       │   └── <Identifier> "print"
    │       └── <PostfixOp> (FunctionCall)
    │           ├── "("
    │           ├── <Arguments>
    │           │   └── <Expression>
    │           │       └── <Primary>
    │           │           └── <Identifier> "x"
    │           └── ")"
    └── ";"
```

### 2. Expression Precedence Example

**Expression:** `2 + 3 * 4 == 14`

**How the grammar parses it:**

1. Start at `<Expression>` → `<Assignment>` → `<LogicalOr>` → `<LogicalAnd>` → `<Equality>`
2. At `<Equality>`, parse left side: `<Comparison>` for `2 + 3 * 4`
3. At `<Comparison>`, go to `<Term>` for `2 + 3 * 4`
4. At `<Term>`, parse `2` as `<Factor>`, see `+`, parse `3 * 4`
5. For `3 * 4`, go to `<Factor>` level where `*` is handled
6. `<Factor>` parses `3 * 4` as a single unit (higher precedence)
7. Return to `<Term>` with result: `2 + (3 * 4)` → `2 + 12` → `14`
8. Back at `<Equality>`, see `==`, parse right side: `14`
9. Final result: `(2 + 3 * 4) == 14` → `14 == 14` → `true`

**Parse Tree (simplified):**
```
<Equality>
├── <Term>
│   ├── <Factor> "2"
│   ├── "+"
│   └── <Factor>
│       ├── <Unary> "3"
│       ├── "*"
│       └── <Unary> "4"
├── "=="
└── <Primary> "14"
```

### 3. Control Flow Example

**Program:**
```javascript
if (x > 5) {
    print("big");
} else {
    print("small");
}
```

**Applying the grammar:**
```
<IfStatement> ::= "if" "(" <Expression> ")" <BlockStatement> ["else" <BlockStatement>]
```

**Breakdown:**
1. `"if"` - matches keyword ✓
2. `"("` - matches ✓
3. `<Expression>` - matches `x > 5` ✓
4. `")"` - matches ✓
5. `<BlockStatement>` - matches `{ print("big"); }` ✓
6. `["else" <BlockStatement>]` - matches `else { print("small"); }` ✓

### 4. Function Definition and Call

**Program:**
```javascript
fn add(a, b) {
    return a + b;
}

let result = add(5, 3);
```

**First statement (function definition):**
```
<FunctionStatement> ::= "fn" <Identifier> "(" [<Parameters>] ")" <BlockStatement>
```
- `"fn"` ✓
- `<Identifier>` → `"add"` ✓
- `"("` ✓
- `<Parameters>` → `"a" "," "b"` ✓
- `")"` ✓
- `<BlockStatement>` → `{ return a + b; }` ✓

**Second statement (function call):**
```
<Postfix> ::= <Primary> {<PostfixOp>}
<PostfixOp> ::= "(" [<Arguments>] ")"
```
- `<Primary>` → `"add"` (identifier)
- `<PostfixOp>` → `"(" "5" "," "3" ")"` (function call)

---

## Examples

### Example 1: Fibonacci Function

**Code:**
```javascript
fn fib(n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

print(fib(10));
```

**Grammar trace for `fib(n - 1)`:**
1. `<Postfix>` starts with `<Primary>` → `fib` (identifier)
2. `{<PostfixOp>}` → function call: `"(" <Arguments> ")"`
3. `<Arguments>` → `<Expression>`
4. `<Expression>` → ... → `<Term>`
5. `<Term>` → `n` (Factor) `-` `1` (Factor)
6. Result: `fib((n - 1))`

### Example 2: Array Operations

**Code:**
```javascript
let numbers = [1, 2, 3, 4, 5];
let first = numbers[0];
```

**Grammar for array literal:**
```
<ArrayLiteral> ::= "[" [<ArrayElements>] "]"
<ArrayElements> ::= <Expression> {"," <Expression>}
```
- Matches: `"[" 1 "," 2 "," 3 "," 4 "," 5 "]"` ✓

**Grammar for array indexing:**
```
<Postfix> ::= <Primary> {<PostfixOp>}
<PostfixOp> ::= "[" <Expression> "]"
```
- `<Primary>` → `numbers`
- `<PostfixOp>` → `"[" 0 "]"`
- Result: `numbers[0]` ✓

### Example 3: Complex Expression

**Code:**
```javascript
let result = (10 + 20) * 2 - 5 / 5 == 59;
```

**Precedence breakdown:**
1. **Parentheses** (highest): `(10 + 20)` → `30`
2. **Multiplication/Division**: `30 * 2` → `60`, `5 / 5` → `1`
3. **Addition/Subtraction**: `60 - 1` → `59`
4. **Equality** (lowest): `59 == 59` → `true`

**Grammar path:**
```
<Expression>
└── <Assignment>
    └── <LogicalOr>
        └── <LogicalAnd>
            └── <Equality>
                ├── <Comparison>
                │   └── <Term>
                │       ├── <Factor>
                │       │   ├── <Unary>
                │       │   │   └── <Postfix>
                │       │   │       └── <Primary> "(" <Expression> ")"  -- (10 + 20)
                │       │   ├── "*"
                │       │   └── <Unary> "2"
                │       ├── "-"
                │       └── <Factor>
                │           ├── <Unary> "5"
                │           ├── "/"
                │           └── <Unary> "5"
                ├── "=="
                └── <Comparison> "59"
```

---

## Operator Precedence Table

From **lowest** to **highest** precedence:

| Precedence Level | Operators | Associativity | Example |
|------------------|-----------|---------------|---------|
| 1 | `=`, `+=`, `-=`, `*=`, `/=` | Right | `x = y = 5` → `x = (y = 5)` |
| 2 | `\|\|` | Left | `a \|\| b \|\| c` → `(a \|\| b) \|\| c` |
| 3 | `&&` | Left | `a && b && c` → `(a && b) && c` |
| 4 | `==`, `!=` | Left | `a == b != c` → `(a == b) != c` |
| 5 | `<`, `>` | Left | `a < b > c` → `(a < b) > c` |
| 6 | `+`, `-` | Left | `a + b - c` → `(a + b) - c` |
| 7 | `*`, `/` | Left | `a * b / c` → `(a * b) / c` |
| 8 | `-x`, `!x` (prefix) | Right | `!!x` → `!(!x)` |
| 9 | `f()`, `a[i]`, `obj.prop` | Left | `f()[0].x` → `((f())[0]).x` |

**Key Points:**
- **Left associativity**: `a + b + c` = `(a + b) + c`
- **Right associativity**: `a = b = c` = `a = (b = c)`
- **Higher precedence binds tighter**: `2 + 3 * 4` = `2 + (3 * 4)`

---

## Summary

This grammar defines Rustphy as a:
- **Imperative** language (statements, control flow)
- **Procedural** language (functions)
- **Expression-based** language (everything has a value)
- With **C-style syntax** and **JavaScript-like semantics**

The BNF notation provides a precise, unambiguous specification that can be used to:
- Understand the language syntax
- Implement parsers
- Validate programs
- Generate test cases
- Create language tooling (syntax highlighters, linters, etc.)
