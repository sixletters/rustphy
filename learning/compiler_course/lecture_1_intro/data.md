# Lecture 1 - Introduction to Compilers

**Author: Harris Maung**

_Based on MIT OpenCourseWare 6.035 - Computer Language Engineering_

---

Hey everyone! Welcome to my compiler series where I learn and digest the MIT OpenCourseWare on Computer Language Engineering. Rather than giving you raw notes, these will be readable, digestible articles! I'll also include problem sets, solutions, and a Q&A section after each lecture.

## Why Study Compilers?

Because they're cool! But more seriously, compilers enable us to program in high-level, human-understandable languages. Can you imagine writing binary every time you want to write a web server? The world would be a bleak, bleak place!

### Why Not Use Natural Language?

While natural language is powerful, it's highly ambiguous—the same expression can describe many possible actions. Imagine two developers giving the same input to an AI, but getting two different code implementations based on context!

Programming languages need to be:

- **Precise** - No ambiguity in meaning
- **Concise** - Express ideas efficiently
- **Expressive** - Capable of describing complex operations
- **High-level** - Provide abstractions so we don't have to write pure machine code

### Abstraction Levels: A Mail Delivery Analogy

Think of programming abstractions like mailing a letter:

```
High-level (User): "I want to deliver this mail to this address"
    ↓
Mid-level (Local Mailman): "Take this mail to the mail station"
    ↓
Mid-level (Sorter): "Sort mail into corresponding outbox slots"
    ↓
Low-level (Shipper): "Transport to destination facility"
    ↓
Low-level (Destination Mailman): "Deliver to exact address"
```

Each level handles its own abstraction, with details hidden from the layers above.

## What Microprocessors Actually Understand

Remember: microprocessors talk in assembly language (or binary) and expose Instruction Set Architectures (ISAs). A CPU is actually just a very fast, very dumb machine that takes numbers and does specific things with them!

The CPU operates on:

- **Registers** - Small, fast storage locations
- **Memory** - Flat address space for data
- **Machine code** - Load/store architecture
  - Load/store instructions
  - Arithmetic operations
  - Logical operations

## What Does a Compiler Do?

A compiler:

1. Reads the high-level language you write
2. Performs various checks (syntax, semantics, type checking)
3. Figures out how to carry out those actions efficiently
4. Produces a binary list of instructions that the CPU can execute

## Anatomy of a Compiler

Now that sounds complicated, but let's break down the steps a compiler takes to transform your code into optimized machine instructions:

```
┌─────────────────┐
│  Source Code    │
│  (High-level)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Lexical         │  Converts source into tokens
│ Analyzer        │  (keywords, identifiers, operators)
│ (Scanner)       │
└────────┬────────┘
         │ Token Stream
         ▼
┌─────────────────┐
│ Syntax          │  Builds parse tree / AST
│ Analyzer        │  (checks grammatical structure)
│ (Parser)        │
└────────┬────────┘
         │ Abstract Syntax Tree (AST)
         ▼
┌─────────────────┐
│ Semantic        │  Type checking, scope resolution
│ Analyzer        │  (ensures program makes sense)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Intermediate    │  Platform-independent representation
│ Representation  │  (e.g., LLVM IR, Three-Address Code)
│ (IR)            │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Code            │  Optimization passes
│ Optimizer       │  (makes code faster/smaller)
└────────┬────────┘
         │ Optimized IR
         ▼
┌─────────────────┐
│ Code            │  Generates target assembly
│ Generator       │  (platform-specific)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Assembly Code   │
│ (Machine Code)  │
└─────────────────┘
```

### Key Concepts: AST and IR

Let's demystify two critical data structures in the compilation process:

#### Abstract Syntax Tree (AST)

An **Abstract Syntax Tree** is a tree representation of the syntactic structure of your source code. Each node in the tree represents a construct in the code (like an expression, statement, or declaration).

**Example:** The simple expression `a = b + 3 * 2`

```
Source Code: a = b + 3 * 2

AST:
         =
        / \
       a   +
          / \
         b   *
            / \
           3   2
```

The AST captures the **structure** and **precedence** of operations without worrying about formatting details like whitespace or parentheses (hence "abstract").

**Another example:**
```c
if (x > 10) {
    y = 5;
}
```

```
AST:
    IfStatement
        /    \
   Condition  Body
      |        |
      >     Assignment
     / \      /    \
    x  10    y      5
```

The AST is language-specific and closely tied to the source language's grammar.

#### Intermediate Representation (IR)

**Intermediate Representation** is a lower-level, platform-independent code representation that sits between the source code and machine code. It's designed to be easy to optimize and transform.

**Why use IR?**
- **Decouples frontend from backend** - One IR can target multiple architectures (x86, ARM, RISC-V)
- **Optimization-friendly** - Easier to analyze and transform than source code or assembly
- **Language-agnostic** - Different languages can compile to the same IR

**Example:** The same `a = b + 3 * 2` in Three-Address Code (a common IR form):

```
Source Code: a = b + 3 * 2

Three-Address Code IR:
    t1 = 3 * 2
    t2 = b + t1
    a = t2
```

**LLVM IR Example:**
```llvm
Source: int add(int a, int b) { return a + b; }

LLVM IR:
define i32 @add(i32 %a, i32 %b) {
entry:
  %result = add i32 %a, %b
  ret i32 %result
}
```

**Fun fact:** LLVM IR is used by languages like Rust, Swift, C/C++ (via Clang), and many others. Write a compiler frontend once, and you get optimization + code generation for dozens of platforms for free!

## Why Optimization Matters

If you think a compiler is just a glorified translator, you'd be wrong! A simple one-to-one mapping from high-level code to assembly would produce **highly inefficient execution**. The problem gets worse with higher levels of abstraction—the more abstract the language, the more potential inefficiency.

If compilation isn't efficient, it defeats the whole point of having abstractions!

## Compiler Optimizations in Action

Let's walk through how a compiler optimizes code step-by-step. Starting with this function:

### Original Code

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, y;
    x = 0;
    y = 0;
    for(i = 0; i <= N; i++) {
        x = x + (4 * a/b) * i + (i+1) * (i+1);
        x = x + b * y;
    }
    return x;
}
```

Compilers apply optimizations incrementally, making micro-changes at each step:

### 1. Constant Propagation

Notice that `y` is assigned `0` and never reassigned. We can replace all uses of `y` with `0`:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, y;
    x = 0;
    y = 0;
    for(i = 0; i <= N; i++) {
        x = x + (4 * a/b) * i + (i+1) * (i+1);
        x = x;  // b * y = b * 0 = 0, so x = x + 0 = x
    }
    return x;
}
```

### 2. Copy Propagation

The statement `x = x` does nothing, so we can remove it:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, y;
    x = 0;
    y = 0;
    for(i = 0; i <= N; i++) {
        x = x + (4 * a/b) * i + (i+1) * (i+1);
    }
    return x;
}
```

### 3. Common Subexpression Elimination

We compute `(i+1)` twice in the expression `(i+1) * (i+1)`. Calculate it once and reuse:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, y, t;
    x = 0;
    y = 0;
    for(i = 0; i <= N; i++) {
        t = i + 1;
        x = x + (4 * a/b) * i + t * t;
    }
    return x;
}
```

### 4. Dead Code Elimination

The variable `y` is never used, so we can remove it:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, t;
    x = 0;
    for(i = 0; i <= N; i++) {
        t = i + 1;
        x = x + (4 * a/b) * i + t * t;
    }
    return x;
}
```

### 5. Loop Invariant Code Motion

The expression `(4 * a/b)` doesn't change inside the loop—it's **loop invariant**. Compute it once before the loop:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, t, u;
    x = 0;
    u = (4 * a/b);
    for(i = 0; i <= N; i++) {
        t = i + 1;
        x = x + u * i + t * t;
    }
    return x;
}
```

### 6. Strength Reduction

Addition is cheaper than multiplication on most processors. Since `v = u * i`, and `i` increments by 1 each iteration, we can replace multiplication with addition:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, t, u, v;
    x = 0;
    u = (4 * a/b);
    v = 0;  // v = u * 0 initially
    for(i = 0; i <= N; i++) {
        t = i + 1;
        x = x + v + t * t;
        v = v + u;  // Instead of v = u * i
    }
    return x;
}
```

### 7. Bit Shift Optimization

Bit shifting is cheaper than multiplication. We can replace `4 * a/b` with `(a/b) << 2`:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, t, u, v;
    x = 0;
    u = (a/b) << 2;  // Left shift by 2 is equivalent to multiply by 4
    v = 0;
    for(i = 0; i <= N; i++) {
        t = i + 1;
        x = x + v + t * t;
        v = v + u;
    }
    return x;
}
```

### 8. Induction Variable Optimization (Loop Index Transformation)

Notice that `t` is always `i + 1`. Instead of computing `t` on every iteration, we can adjust the loop bounds and use `i` directly:

```c
int sumcalc(int a, int b, int N)
{
    int i;
    int x, u, v;
    x = 0;
    u = (a/b) << 2;
    v = 0;
    for(i = 1; i <= N + 1; i++) {  // Start at 1 instead of 0
        x = x + v + i * i;         // Use i directly instead of t
        v = v + u;
    }
    return x;
}
```

By changing the loop to start at `1` and go to `N + 1`, we eliminate the need for the temporary variable `t` entirely!

### The Result

Through these **8 incremental optimizations**, we've transformed the original code into something **significantly faster**, with:

- Fewer variables (from 5 to 4 - eliminated `y` and `t`)
- Fewer memory accesses
- Fewer operations inside the loop
- Cheaper operations (shifts instead of multiplies, additions instead of multiplies)
- Eliminated redundant calculations

**Original:** 5 variables, multiple multiplications per iteration, repeated subexpressions  
**Optimized:** 4 variables, mostly additions, loop-invariant code moved out

This is the power of compiler optimizations! 🚀

### Assembly Code Comparison

To truly appreciate the impact, let's look at the generated assembly code:

#### Unoptimized Assembly (~35 instructions per loop iteration)

```asm
.L2:
    movl    -4(%rbp), %eax      # Load i
    leal    0(,%rax,4), %edx    # i * 4
    movl    -8(%rbp), %eax      # Load a
    imull   %edx, %eax          # 4 * a
    movl    %eax, %edx
    movl    -12(%rbp), %eax     # Load b
    idivl   %edx                # Divide
    movl    -4(%rbp), %edx      # Load i again
    imull   %edx, %eax          # Multiply
    movl    %eax, -20(%rbp)     # Store temp
    movl    -4(%rbp), %eax      # Load i
    addl    $1, %eax            # i + 1
    imull   %eax, %eax          # (i+1) * (i+1)
    movl    -20(%rbp), %edx     # Load temp
    addl    %eax, %edx          # Add
    movl    -16(%rbp), %eax     # Load x
    addl    %edx, %eax          # x = x + ...
    movl    %eax, -16(%rbp)     # Store x
    movl    -12(%rbp), %eax     # Load b
    imull   -24(%rbp), %eax     # b * y
    movl    -16(%rbp), %edx     # Load x
    addl    %eax, %edx          # x = x + b*y
    movl    %edx, -16(%rbp)     # Store x
    addl    $1, -4(%rbp)        # i++
    # ... loop condition check
```

#### Optimized Assembly (~8 instructions per loop iteration)

**Main loop body:**
```asm
.L5:
    movl    %edi, %eax          # i to eax
    imull   %edi, %eax          # i * i
    addl    %ecx, %eax          # + v
    addl    %eax, %edx          # x = x + ...
    addl    %r8d, %ecx          # v = v + u
    addl    $1, %edi            # i++
    cmpl    %esi, %edi          # Compare i with N+1
    jle     .L5                 # Loop if <=
```

**Loop-invariant code (computed once before the loop):**
```asm
    movl    %esi, %eax          # a
    sarl    $31, %edx           # Arithmetic setup
    idivl   %ecx                # a / b
    sall    $2, %eax            # << 2 (multiply by 4)
    movl    %eax, %r8d          # u = (a/b) << 2
```

**The difference is staggering:**
- **Unoptimized:** ~35 instructions per iteration with multiple memory loads/stores
- **Optimized:** ~8 instructions per iteration, mostly register operations

For a loop running 1,000,000 iterations:
- **Unoptimized:** ~35,000,000 instructions
- **Optimized:** ~8,000,000 instructions + one-time setup

**That's over 4x faster!** And this is just from source-level optimizations—modern compilers apply even more at the backend.

---

**Note:** Modern compilers like GCC and Clang apply hundreds of optimization passes, far beyond what we've covered here. This is just a taste of what happens under the hood!

---

## Quiz Questions

Test your understanding of the concepts covered in this lecture!

### Question 1: Multiple Choice
Which of the following is NOT a requirement for programming languages?
- A) Precision
- B) Ambiguity
- C) Expressiveness
- D) High-level abstractions

<details>
<summary>Answer</summary>
B) Ambiguity - Programming languages must be unambiguous, unlike natural languages!
</details>

### Question 2: Ordering
Put the following compiler phases in the correct order:
- A) Code Generator
- B) Lexical Analyzer
- C) Semantic Analyzer
- D) Parser (Syntax Analyzer)
- E) Code Optimizer

<details>
<summary>Answer</summary>
B → D → C → E → A

(Lexical Analyzer → Parser → Semantic Analyzer → Code Optimizer → Code Generator)
</details>

### Question 3: Short Answer
What is the difference between an AST (Abstract Syntax Tree) and IR (Intermediate Representation)?

<details>
<summary>Answer</summary>
An AST is a tree structure representing the grammatical structure of the source code, closely tied to the source language's syntax. IR is a lower-level, platform-independent representation that's closer to machine code but still abstract enough to enable optimizations. IR often uses forms like three-address code or SSA (Static Single Assignment).
</details>

### Question 4: Optimization Identification
For each optimization, identify its type:

1. Replacing `y * 0` with `0`
2. Computing `MAX_VALUE` once before a loop instead of in every iteration
3. Replacing `x * 8` with `x << 3`
4. Removing an unused variable `temp`
5. Calculating `sqrt(a)` once instead of twice in the same expression
6. Changing `for(i=0; i<N; i++) { ... i+1 ...}` to `for(i=1; i<=N; i++) { ... i ...}` to eliminate a temporary variable

<details>
<summary>Answers</summary>

1. **Constant Propagation/Constant Folding**
2. **Loop Invariant Code Motion**
3. **Strength Reduction**
4. **Dead Code Elimination**
5. **Common Subexpression Elimination**
6. **Induction Variable Optimization (Loop Index Transformation)**
</details>

### Question 5: Conceptual
Why do compilers perform optimizations incrementally (in multiple passes) rather than all at once?

<details>
<summary>Answer</summary>
Incremental optimizations make it easier to reason about correctness, enable optimizations to build on each other (one optimization may expose opportunities for another), and make the compiler more maintainable. For example, dead code elimination is more effective after constant propagation has simplified expressions.
</details>

---

## Practice Problems

Try optimizing these code snippets yourself! Apply the optimization techniques you learned.

### Practice 1: Basic Optimizations

```c
int calculate(int n) {
    int result = 0;
    int unused = 42;
    int factor = 10 * 5;
    
    for (int i = 0; i < n; i++) {
        result = result + factor;
        result = result + 0;
    }
    
    return result;
}
```

**Your task:** Identify and apply at least 3 optimizations.

<details>
<summary>Optimized Solution</summary>

```c
int calculate(int n) {
    int result = 0;
    int factor = 50;  // Constant folding: 10 * 5
    
    for (int i = 0; i < n; i++) {
        result = result + factor;  // Copy propagation: removed + 0
    }
    // Dead code elimination: removed 'unused' variable
    
    return result;
}
```

**Further optimization:**
```c
int calculate(int n) {
    return 50 * n;  // The loop just adds 50, n times!
}
```
</details>

### Practice 2: Loop Optimization

```c
double physics_simulation(double mass, double velocity, int iterations) {
    double energy = 0.0;
    double constant = 0.5;
    
    for (int i = 0; i < iterations; i++) {
        energy = energy + (constant * mass * velocity * velocity);
        energy = energy + (constant * mass * velocity * velocity);
    }
    
    return energy;
}
```

**Your task:** Optimize this function. Look for common subexpressions and loop-invariant code.

<details>
<summary>Optimized Solution</summary>

```c
double physics_simulation(double mass, double velocity, int iterations) {
    double energy = 0.0;
    double kinetic = 0.5 * mass * velocity * velocity;  // Loop invariant
    
    for (int i = 0; i < iterations; i++) {
        energy = energy + kinetic;  // Common subexpression elimination
        energy = energy + kinetic;
    }
    
    return energy;
}
```

**Further optimization:**
```c
double physics_simulation(double mass, double velocity, int iterations) {
    double kinetic = 0.5 * mass * velocity * velocity;
    return 2.0 * iterations * kinetic;  // Loop eliminated entirely!
}
```
</details>

### Practice 3: Strength Reduction Challenge

```c
int array_multiply(int arr[], int size, int multiplier) {
    int sum = 0;
    
    for (int i = 0; i < size; i++) {
        sum = sum + (arr[i] * 16);
        sum = sum + (multiplier * 32);
    }
    
    return sum;
}
```

**Your task:** Apply loop-invariant code motion, strength reduction (bit shifts), and any other optimizations you can find.

<details>
<summary>Optimized Solution</summary>

```c
int array_multiply(int arr[], int size, int multiplier) {
    int sum = 0;
    int mult_contribution = multiplier << 5;  // Loop invariant: multiplier * 32
    
    for (int i = 0; i < size; i++) {
        sum = sum + (arr[i] << 4);           // Strength reduction: * 16 → << 4
        sum = sum + mult_contribution;
    }
    
    return sum;
}
```

**Even further:**
```c
int array_multiply(int arr[], int size, int multiplier) {
    int sum = 0;
    int mult_contribution = (multiplier << 5) * size;  // Computed once
    
    for (int i = 0; i < size; i++) {
        sum = sum + (arr[i] << 4);
    }
    
    return sum + mult_contribution;  // Add constant after loop
}
```
</details>

---

## Q&A Section

**Q: Do I need to manually apply these optimizations in my code?**  
A: Generally, no! Modern compilers (GCC with `-O2` or `-O3`, Clang, Rustc, etc.) automatically apply these and many more optimizations. However, understanding them helps you write compiler-friendly code and debug performance issues.

**Q: Can optimizations ever make code slower?**  
A: Rarely, but yes! Over-optimization can increase code size, causing instruction cache misses. This is why compilers have different optimization levels (`-O1`, `-O2`, `-O3`, `-Os` for size).

**Q: What's the difference between the AST and IR?**  
A: The AST represents the source code's structure (syntactic), while IR is a lower-level representation designed for optimization and code generation. Multiple source languages can compile to the same IR (e.g., C, C++, and Rust all compile to LLVM IR).

**Q: Why is LLVM IR so popular?**  
A: LLVM IR provides a clean separation between language frontends and architecture backends. Write a frontend for your language once, and you get code generation for x86, ARM, RISC-V, etc., for free! Plus, you inherit LLVM's sophisticated optimization passes.

---

**Next Lecture:** Lexical Analysis and Scanning - How compilers break source code into tokens!

*If you found this helpful, star the repo and check out the next lecture!*
