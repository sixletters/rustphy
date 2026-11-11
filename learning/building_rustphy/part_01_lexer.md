---
layout: default
title: Part 1 - Teaching the Compiler to Read
---

# Part 1: Teaching the Compiler to Read

**Building Rustphy: A Compiler From Scratch**

---

## A Compiler Does Not See Code

When I look at this:

```javascript
let distance = 3.14;
```

I immediately see a variable declaration, a name, an assignment, and a floating-point value.

The compiler begins with none of that. It sees bytes:

```text
l e t _ d i s t a n c e _ = _ 3 . 1 4 ;
```

The lexer's job is to group those bytes into the first useful units of meaning:

```text
Let
Ident("distance")
Assign
Float("3.14")
Semicolon
Eof
```

This phase is called **lexical analysis**, but the name makes it sound more mysterious than it is. The lexer answers one repeated question:

> Starting at this byte, what is the longest valid token I can recognize?

The challenge is that the answer sometimes depends on the next byte. `=` and `==` begin the same way. So do `.` and `..`. So do `/` and `/=`—and, if we add comments, `//` and `/*`.

Let's build that decision-making machine.

---

## Step 1: Design Tokens as Data, Not Labels

Rustphy uses one enum for both token kind and token payload:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Token {
    Illegal(char),
    Eof,

    Ident(String),
    Int(String),
    Str(String),
    Float(String),

    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    // ...
}
```

A token like `Plus` needs no extra information. Every plus sign means the same thing at this stage.

An identifier token is different. `counter`, `value`, and `print` are all identifiers, but later phases need their spelling. That is why `Ident` carries a `String`.

The same is true for literals:

```rust
Int(String)
Float(String)
Str(String)
```

### Why Keep Numbers as Strings?

The lexer recognizes that `18446744073709551616` has the *shape* of an integer. It does not yet need to decide whether that value fits in `i64`, `i128`, or some future bigint type.

Rustphy postpones conversion until parsing:

```text
Lexer:  "123" has valid integer syntax → Int("123")
Parser: Int("123") parses into i64       → Integer(123)
```

That separation keeps the lexer focused on lexical structure rather than language-level numeric representation.

Strings take a slightly different path. Their token payload excludes quotes and already contains decoded escapes. The source text `"hello\n"` becomes a string token containing an actual newline.

---

## Step 2: Represent the Cursor

Rustphy's lexer owns the input and keeps three pieces of cursor state:

```rust
pub struct Lexer {
    input: String,
    pos: usize,
    read_pos: usize,
    ch: u8,
}
```

Think of them like this:

```text
input:       l  e  t     x  =  1  ;
index:       0  1  2  3  4  5  6  7
             ↑
            pos
                ↑
             read_pos
ch = b'l'
```

- `pos` is the index of the current byte.
- `read_pos` is where the next read will happen.
- `ch` caches the current byte.

The constructor immediately reads once so `ch` points at the first input byte.

```rust
fn read_char(&mut self) {
    if self.read_pos >= self.input.len() {
        self.ch = 0;
    } else {
        self.ch = self.input.as_bytes()[self.read_pos];
    }
    self.pos = self.read_pos;
    self.read_pos += 1;
}
```

A zero byte acts as the EOF sentinel.

This is a simple one-byte-lookahead scanner. `peek_char` reads the next byte without moving the cursor:

```rust
fn peek_char(&self) -> u8 {
    if self.read_pos >= self.input.len() {
        0
    } else {
        self.input.as_bytes()[self.read_pos]
    }
}
```

That one helper is enough to distinguish most multi-character operators in Rustphy.

---

## Step 3: Dispatch on the Current Byte

The main loop starts by removing whitespace, then matches on `ch`:

```rust
pub fn next_token(&mut self) -> Token {
    self.eat_whitespace();

    let token = match self.ch {
        b'=' => {
            if matches!(self.peek_char(), b'=') {
                self.read_char();
                self.read_char();
                return Token::Eq;
            }
            Token::Assign
        }
        // ...
    };

    self.read_char();
    token
}
```

The `=` branch tells the whole lookahead story:

```text
Current '=' + next '='  → Eq
Current '=' + anything  → Assign
```

The lexer applies the same pattern to:

- `!` and `!=`
- `&` and `&&`
- `|` and `||`
- `+` and `+=`
- `-` and `-=`
- `*` and `*=`
- `/` and `/=`
- `.` / `..` / `..=`

A single `&` or `|` is illegal in the current language, while doubled versions are logical operators.

### An Implementation Detail Worth Noticing

The current lexer mixes two consumption styles:

1. Build a token, fall through, and call `read_char()` once at the end.
2. Consume all relevant bytes inside a branch and return early.

It works, but it means every new branch must know exactly how many bytes have already been consumed. A future refactor could make the convention more uniform and reduce off-by-one mistakes.

This is one of those details that looks trivial until your lexer skips the first character of the next token!

---

## Step 4: Scan Identifiers Before Recognizing Keywords

How does the lexer know whether `letter` is the keyword `let` followed by `ter`, or one identifier?

It follows the **maximal munch** rule: consume the longest sequence that can form the current token.

```rust
fn read_identifier(&mut self) -> String {
    let position = self.pos;
    while Self::is_letter_or_underscore(self.ch) || Self::is_digit(self.ch) {
        self.read_char();
    }
    self.input[position..self.pos].to_string()
}
```

After scanning the complete spelling, the lexer checks whether it is reserved:

```rust
pub fn lookup_identifier(ident: &str) -> Token {
    match ident {
        "func" => Token::Function,
        "for" => Token::For,
        "let" => Token::Let,
        "true" => Token::True,
        "false" => Token::False,
        "if" => Token::If,
        "else" => Token::Else,
        "return" => Token::Return,
        "break" => Token::Break,
        "continue" => Token::Continue,
        "in" => Token::In,
        _ => Token::Ident(ident.to_string()),
    }
}
```

So:

```text
let       → Let
letter    → Ident("letter")
returning → Ident("returning")
```

Keyword recognition happens *after* identifier recognition, not during it.

### A Current Edge Case

The initial-character check accepts ASCII letters, while the continuation helper accepts letters, digits, and `_`. That means `name_1` works, but `_name` begins with an illegal token.

The lexer is also byte-oriented and ASCII-oriented. UTF-8 identifiers and non-ASCII string contents need a character-aware or byte-slice-aware design. This is not hidden magic: choosing `u8` as the cursor unit is choosing an ASCII-first language for now.

---

## Step 5: The Most Interesting Dot in the Compiler

Now we reach my favourite lexer detail.

Rustphy uses dots for three different things:

```javascript
3.14          // floating-point literal
0..10         // exclusive range
0..=10        // inclusive range
object.field  // member-style access
```

Suppose the lexer is reading a number. After consuming digits, it sees `.`. Should the dot become part of the number?

Rustphy's rule is:

> A dot belongs to a number only when the byte after the dot is a digit.

```rust
fn read_number(&mut self) -> String {
    let pos = self.pos;

    while Lexer::is_digit(self.ch) {
        self.read_char();
    }

    if matches!(self.ch, b'.') && Lexer::is_digit(self.peek_char()) {
        self.read_char();
        while Lexer::is_digit(self.ch) {
            self.read_char();
        }
    }

    self.input[pos..self.pos].to_string()
}
```

Trace `3.14`:

```text
Read digits: 3
Current: '.'
Peek:    '1' (digit)
Decision: decimal point
Result: Float("3.14")
```

Trace `0..10`:

```text
Read digits: 0
Current: '.'
Peek:    '.' (not digit)
Decision: stop the number
Result so far: Int("0")
Next token: DotDot
Next token: Int("10")
```

Trace `object.field`:

```text
Ident("object")
Dot
Ident("field")
```

One lookahead rule protects three syntactic features from colliding.

### Current Number Grammar

The implementation supports:

```text
42       integer
3.14     float
-1       Minus + Int("1")
```

The minus sign is not part of the number. Unary negation belongs to parsing, where `-` can be treated as an operator.

It does not yet support:

- `.5`
- `12.`
- Scientific notation such as `1e9`
- Hexadecimal or binary literals
- Numeric separators such as `1_000`

In fact, `.5` currently reaches a `todo!` path. That is a useful reminder: a lexer needs an explicit language definition, not an intuition that “floats are supported.”

---

## Step 6: Decode Strings Once

String scanning begins after the opening quote has been consumed. The lexer continues until a closing quote or EOF.

Escapes are decoded immediately:

```rust
let escaped_char = match self.ch {
    b'n' => '\n',
    b't' => '\t',
    b'r' => '\r',
    b'"' => '"',
    b'\\' => '\\',
    _ => {
        return Err(format!(
            "Invalid escape sequence: \\{}",
            self.ch as char
        ));
    }
};
result.push(escaped_char);
self.read_char();
```

So the later compiler phases do not need to remember that `\n` means newline. They receive the semantic string contents.

This is an example of an early lowering that is usually safe: escape syntax belongs to source spelling, while the AST cares about the resulting value.

### But the Error Boundary Is Inconsistent

The string helper returns `Result<String, String>`, which suggests recoverable lexing. The public `next_token` currently converts that failure into a panic.

Other invalid characters produce `Token::Illegal` and allow scanning to continue.

That gives the lexer three different failure styles:

```text
Unknown character     → Illegal token
Malformed string      → panic
Leading-dot float     → todo! panic
```

A production-quality design should unify these into structured diagnostics containing at least:

- An error kind
- A byte range
- A line and column for display
- The offending source fragment

We will return to source spans in Part 7.

---

## Step 7: Recognize Ranges

The dot branch uses two bytes of lookahead conceptually:

```text
'.'              → Dot
'.' '.'          → DotDot
'.' '.' '='      → DotDotEquals
```

Because the lexer only exposes one `peek_char`, it consumes the second dot and then checks the new peek/current state to recognize `..=`.

The resulting tokens let the parser build a dedicated range expression:

```javascript
0..10   // end excluded
0..=10  // end included
```

Why use a dedicated range token rather than two `Dot` tokens? Because tokenization should capture stable lexical units. Once the language declares `..=` one operator, no later phase should reconstruct it from three independent characters.

---

## Step 8: EOF Is a Token Too

When `ch` is the zero sentinel, the lexer emits `Eof`.

EOF matters because parsers need to distinguish:

- “I finished the program successfully.”
- “I expected `}` but the input ended.”
- “There are unexpected tokens after this expression.”

Rustphy's current parser does not always make those distinctions perfectly, but the lexer provides the necessary terminal marker.

---

## Full Trace: Lexing a Range Loop

Consider:

```javascript
for (i in 0..=3) {
    print(i);
};
```

The lexer produces conceptually:

```text
For
LParen
Ident("i")
In
Int("0")
DotDotEquals
Int("3")
RParen
LBrace
Ident("print")
LParen
Ident("i")
RParen
Semicolon
RBrace
Semicolon
Eof
```

Notice what the lexer does **not** decide:

- Whether `i` has been declared
- Whether `0..=3` is iterable
- Whether `print` is a builtin
- Whether `break` would be legal here
- How the loop executes

Those are syntactic, semantic, or runtime questions. The lexer only identifies words.

---

## Why Not Use Regex?

You absolutely can build a lexer from regular expressions or use a generated lexer. Rustphy uses a hand-written scanner because it makes the mechanics visible and keeps special cases easy to control.

The tradeoff is manual cursor discipline.

A generated lexer gives you:

- A declarative token specification
- Often better Unicode handling
- Fewer cursor bugs

A hand-written lexer gives you:

- Exact control over ambiguous prefixes
- Straightforward custom literal handling
- No generator dependency
- A direct way to learn finite-state recognition

The correct choice depends on the project. For a learning compiler, writing this layer once is extremely valuable.

---

## What I Would Improve Next

### 1. Add Source Spans

Instead of returning only a token, return:

```rust
pub struct SpannedToken {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}
```

Line and column can be derived from offsets or tracked incrementally.

### 2. Return `Result`

```rust
pub fn next_token(&mut self) -> Result<SpannedToken, LexError>
```

No malformed source should crash the compiler.

### 3. Make UTF-8 Policy Explicit

Either define identifiers as ASCII and validate string bytes correctly, or scan Unicode scalar values. The current byte-to-`char` conversion can corrupt non-ASCII strings.

### 4. Add Comments

The `/` branch currently recognizes `/` and `/=` only. Supporting comments means resolving:

```text
/   /=   //...   /*...*/
```

This is a good exercise in lookahead and stateful scanning.

### 5. Clarify Cursor Consumption

Use one convention: helpers consume their complete token and return with `ch` on the first unconsumed byte. Then make `next_token` follow that contract consistently.

---

## Checkpoint

At the end of lexical analysis, source text has become a stream of typed units:

```text
"let x = 3.14;"
       ↓
[Let, Ident("x"), Assign, Float("3.14"), Semicolon, Eof]
```

We now know what the words are, but not how they relate.

The next part asks:

> Why does `1 + 2 * 3` mean `1 + (2 * 3)` instead of `(1 + 2) * 3`?

That is the parser's job.

---

## Quiz

1. Why does Rustphy store numeric token contents as strings?
2. What does one-byte lookahead let the lexer distinguish?
3. Why must keyword lookup happen after scanning the complete identifier?
4. How does the number scanner distinguish `3.14` from `0..10`?
5. Why is `-42` lexed as two tokens?
6. Which semantic questions should the lexer avoid answering?

<details>
<summary><strong>Answers</strong></summary>

1. It separates recognizing numeric syntax from choosing and validating a numeric representation.
2. Single- and multi-character operators with a shared prefix, such as `=`/`==`, `+`/`+=`, and `.`/`..`.
3. Maximal munch prevents a keyword prefix inside a longer identifier from being split incorrectly.
4. The dot joins the number only if the byte after it is a digit.
5. Negation is an operator handled by the parser; it is not part of the literal spelling.
6. Declaration validity, types, scope, builtin resolution, and execution strategy.

</details>

---

## Practice

### Challenge 1: Comments

Add `//` line comments without breaking `/` and `/=`. Then add `/* ... */` block comments. Decide how unterminated comments report errors.

### Challenge 2: Better Floats

Specify and implement a numeric grammar supporting:

```text
.5
5.
1e9
2.5e-3
```

Before coding, write down how each form interacts with `.` and `..`.

### Challenge 3: Source Spans

Attach byte ranges to every token. Use them to print:

```text
error: invalid character '&'
  --> example.gph:2:8
   |
 2 | let x = a & b;
   |           ^
```

### Challenge 4: Unicode Policy

Choose either ASCII-only identifiers or Unicode identifiers. Implement the choice consistently, and add tests for non-ASCII string literals.

---

[← Part 0 — Why I Built a Programming Language](part_00_why_a_language.md) | [Course index](index.md) | [Part 2 — Giving the Language Structure →](part_02_parser_and_ast.md)
