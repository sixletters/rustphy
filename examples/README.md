# Rustphy Examples

This directory contains example programs demonstrating various features of the Rustphy language.

## Running Examples

To run any example:

```bash
rustphy run examples/01_hello_world.gph
```

Or to see the bytecode:

```bash
rustphy compile bytecode examples/01_hello_world.gph
```

Or to see the AST:

```bash
rustphy parse examples/01_hello_world.gph
```

## Example Overview

### [01_hello_world.gph](01_hello_world.gph)
**Topics:** Variables, Strings, Print

The classic first program. Shows basic variable declaration and string concatenation.

```javascript
let greeting = "Hello, World!";
print(greeting);
```

**Run:**
```bash
rustphy run examples/01_hello_world.gph
```

---

### [02_variables_operators.gph](02_variables_operators.gph)
**Topics:** Arithmetic, Compound Assignment, Boolean Logic, Comparisons

Comprehensive demonstration of all operators in Rustphy.

**Key Concepts:**
- Arithmetic: `+`, `-`, `*`, `/`
- Compound assignments: `+=`, `-=`, `*=`, `/=`
- Boolean operators: `&&`, `||`, `!`
- Comparisons: `>`, `<`, `==`, `!=`

**Example:**
```javascript
let counter = 0;
counter += 5;  // 5
counter *= 2;  // 10
```

---

### [03_functions.gph](03_functions.gph)
**Topics:** Function Definition, Parameters, Return Values, Function Calls

Learn how to define and call functions.

**Example:**
```javascript
fn add(a, b) {
    return a + b;
}

let sum = add(5, 3);  // 8
```

---

### [04_conditionals.gph](04_conditionals.gph)
**Topics:** If/Else, Nested Conditionals, Complex Conditions

Control flow with conditional statements.

**Example:**
```javascript
if (age >= 18) {
    print("Adult");
} else {
    print("Minor");
}
```

---

### [05_loops.gph](05_loops.gph)
**Topics:** For Loops, Nested Loops, Loop Control

Iteration and repetition.

**Example:**
```javascript
for (let i = 1; i <= 5; i += 1) {
    print(i);
}
```

**Advanced:**
```javascript
// Multiplication table
for (let i = 1; i <= 3; i += 1) {
    for (let j = 1; j <= 3; j += 1) {
        print(i + " * " + j + " = " + (i * j));
    }
}
```

---

### [06_arrays.gph](06_arrays.gph)
**Topics:** Array Creation, Indexing, Iteration, 2D Arrays

Working with arrays.

**Example:**
```javascript
let numbers = [1, 2, 3, 4, 5];
let first = numbers[0];  // 1

// Sum array elements
let sum = 0;
for (let i = 0; i < 5; i += 1) {
    sum += numbers[i];
}
```

---

### [07_recursion.gph](07_recursion.gph)
**Topics:** Recursive Functions, Base Cases, Recursive Algorithms

Functions that call themselves.

**Examples:**
- **Factorial:** `5! = 5 * 4 * 3 * 2 * 1 = 120`
- **Fibonacci:** `fib(n) = fib(n-1) + fib(n-2)`
- **GCD:** Euclidean algorithm

**Example:**
```javascript
fn factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

print(factorial(5));  // 120
```

---

### [08_advanced.gph](08_advanced.gph)
**Topics:** Closures, Higher-Order Functions, Algorithms

Advanced programming techniques.

**Examples:**
- Prime number checker
- Bubble sort
- Binary search
- First-class functions

**Example:**
```javascript
fn isPrime(n) {
    if (n <= 1) return false;
    if (n <= 3) return true;

    for (let i = 2; i * i <= n; i += 1) {
        if ((n - (n / i) * i) == 0) {
            return false;
        }
    }
    return true;
}
```

---

## Learning Path

**Beginner:**
1. Start with `01_hello_world.gph`
2. Move to `02_variables_operators.gph`
3. Learn functions with `03_functions.gph`

**Intermediate:**
4. Control flow: `04_conditionals.gph`
5. Iteration: `05_loops.gph`
6. Data structures: `06_arrays.gph`

**Advanced:**
7. Recursion: `07_recursion.gph`
8. Algorithms: `08_advanced.gph`

## Experimenting

Try modifying the examples:

1. **Change values:**
   ```bash
   # Edit 02_variables_operators.gph
   # Change let x = 10; to let x = 25;
   rustphy run examples/02_variables_operators.gph
   ```

2. **Add your own functions:**
   ```javascript
   // Add to 03_functions.gph
   fn subtract(a, b) {
       return a - b;
   }
   ```

3. **Create new examples:**
   ```bash
   echo 'fn hello() { print("My function!"); } hello();' > my_example.gph
   rustphy run my_example.gph
   ```

## Common Patterns

### Loop Pattern
```javascript
for (let i = 0; i < n; i += 1) {
    // Do something with i
}
```

### Array Sum Pattern
```javascript
let sum = 0;
for (let i = 0; i < size; i += 1) {
    sum += array[i];
}
```

### Recursive Pattern
```javascript
fn recursive(n) {
    if (base_case) {
        return base_value;
    }
    return recursive(n - 1);  // or some modification
}
```

### Max/Min Pattern
```javascript
let max = array[0];
for (let i = 1; i < size; i += 1) {
    if (array[i] > max) {
        max = array[i];
    }
}
```

## Next Steps

After working through these examples:

1. Read [docs/GRAMMAR.md](../docs/GRAMMAR.md) to understand the formal language specification
2. Explore the [source code](../src/) to see how the language is implemented
3. Try implementing your own features or algorithms
4. Compile examples to different targets (bytecode, WASM) to see the output
