# Rustphy Quick Reference

A quick reference guide for the Rustphy programming language.

## Table of Contents

- [Comments](#comments)
- [Variables](#variables)
- [Data Types](#data-types)
- [Operators](#operators)
- [Control Flow](#control-flow)
- [Functions](#functions)
- [Arrays](#arrays)
- [Built-in Functions](#built-in-functions)

---

## Comments

```javascript
// Single-line comment (not implemented in lexer yet)
// Currently, no comment support
```

---

## Variables

### Declaration

```javascript
let x = 10;
let name = "Alice";
let flag = true;
```

### Assignment

```javascript
x = 20;          // Simple assignment
x += 5;          // Add and assign (x = x + 5)
x -= 3;          // Subtract and assign (x = x - 3)
x *= 2;          // Multiply and assign (x = x * 2)
x /= 4;          // Divide and assign (x = x / 4)
```

---

## Data Types

### Integer

```javascript
let age = 25;
let negative = -10;
```

### String

```javascript
let greeting = "Hello, World!";
let message = "Rust" + "phy";  // Concatenation
```

### Boolean

```javascript
let isTrue = true;
let isFalse = false;
```

### Array

```javascript
let numbers = [1, 2, 3, 4, 5];
let mixed = [1, "hello", true];
let nested = [[1, 2], [3, 4]];
```

### Hash/Object (if supported)

```javascript
let person = {
    "name": "Alice",
    "age": 30
};
```

---

## Operators

### Arithmetic

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Addition | `5 + 3` | `8` |
| `-` | Subtraction | `5 - 3` | `2` |
| `*` | Multiplication | `5 * 3` | `15` |
| `/` | Division | `10 / 2` | `5` |
| `-x` | Negation | `-5` | `-5` |

### Comparison

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `==` | Equal | `5 == 5` | `true` |
| `!=` | Not equal | `5 != 3` | `true` |
| `>` | Greater than | `5 > 3` | `true` |
| `<` | Less than | `3 < 5` | `true` |

### Logical

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `&&` | Logical AND | `true && false` | `false` |
| `\|\|` | Logical OR | `true \|\| false` | `true` |
| `!` | Logical NOT | `!true` | `false` |

### Operator Precedence (High to Low)

1. Function call `f()`, Array index `a[i]`, Member access `obj.prop`
2. Prefix `-x`, `!x`
3. `*`, `/`
4. `+`, `-`
5. `<`, `>`
6. `==`, `!=`
7. `&&`
8. `||`
9. `=`, `+=`, `-=`, `*=`, `/=`

---

## Control Flow

**Important:** All control flow blocks (if, else, for) require a semicolon at the end!

### If Statement

```javascript
if (condition) {
    // code
};

if (condition) {
    // code
} else {
    // code
};

// Nested
if (x > 10) {
    print("big");
} else {
    if (x > 5) {
        print("medium");
    } else {
        print("small");
    };
};
```

### For Loop

```javascript
// Basic loop
for (let i = 0; i < 10; i += 1) {
    print(i);
};

// Custom step
for (let i = 0; i < 100; i += 10) {
    print(i);
};

// Countdown
for (let i = 10; i > 0; i -= 1) {
    print(i);
};

// Nested loops
for (let i = 0; i < 3; i += 1) {
    for (let j = 0; j < 3; j += 1) {
        print(i + "," + j);
    };
};
```

### While Loop (if supported)

```javascript
while (condition) {
    // code
};
```

---

## Functions

### Definition

**Important:** Function declarations require a semicolon at the end!

```javascript
// Basic function
func greet(name) {
    return "Hello, " + name;
};

// Multiple parameters
func add(a, b) {
    return a + b;
};

// No parameters
func sayHello() {
    print("Hello!");
};
```

### Calling Functions

```javascript
let result = add(5, 3);
greet("Alice");
```

### Recursive Functions

```javascript
func factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
};
```

### Function Expressions (First-class Functions)

```javascript
let double = func(x) {
    return x * 2;
};

print(double(5));  // 10
```

### Higher-Order Functions

```javascript
func applyTwice(f, x) {
    return f(f(x));
};

func increment(x) {
    return x + 1;
};

print(applyTwice(increment, 5));  // 7
```

---

## Arrays

### Creation

```javascript
let arr = [1, 2, 3, 4, 5];
let empty = [];
```

### Accessing Elements

```javascript
let first = arr[0];      // 1
let second = arr[1];     // 2
```

### Modifying Elements

```javascript
arr[0] = 10;
arr[1] = arr[0] + 5;
```

### Iterating

```javascript
for (let i = 0; i < 5; i += 1) {
    print(arr[i]);
}
```

### Multidimensional Arrays

```javascript
let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
let element = matrix[1][2];  // 6
```

---

## Built-in Functions

### print

Outputs a value to stdout.

```javascript
print("Hello");
print(42);
print(x + y);
```

---

## Common Patterns

### Sum of Array

```javascript
let sum = 0;
for (let i = 0; i < size; i += 1) {
    sum += arr[i];
};
```

### Find Maximum

```javascript
let max = arr[0];
for (let i = 1; i < size; i += 1) {
    if (arr[i] > max) {
        max = arr[i];
    };
};
```

### Count Loop

```javascript
for (let i = 1; i <= n; i += 1) {
    // do something n times
};
```

### Swap Variables

```javascript
let temp = a;
a = b;
b = temp;
```

### Check Even/Odd

```javascript
func isEven(n) {
    return (n - (n / 2) * 2) == 0;
};
```

### Modulo Operation (since % not available)

```javascript
func mod(a, b) {
    return a - (a / b) * b;
};
```

---

## Tips & Tricks

### String Concatenation

```javascript
let message = "Hello, " + name + "!";
let result = "Result: " + (x + y);  // Use parentheses for expressions
```

### Boolean to String

```javascript
let flag = true;
print("Flag is: " + flag);  // "Flag is: true"
```

### Integer Division

```javascript
let quotient = 10 / 3;   // 3 (integer division)
```

### Multiple Conditions

```javascript
if (x > 0 && x < 10) {
    print("Single digit positive");
};

if (x == 0 || x == 1) {
    print("Zero or one");
};
```

---

## Language Limitations

Current limitations to be aware of:

1. **No modulo operator** - Use `a - (a / b) * b` instead
2. **Integer-only arithmetic** - No floating-point numbers
3. **No comments** - Not yet implemented in lexer
4. **Limited string operations** - Only concatenation with `+`
5. **No break/continue** - In loops
6. **No switch/match** - Use if/else chains
7. **No object methods** - Functions are standalone

---

## Example Program

```javascript
// Fibonacci sequence generator
func fibonacci(n) {
    if (n <= 1) {
        return n;
    };
    return fibonacci(n - 1) + fibonacci(n - 2);
};

// Print first 10 Fibonacci numbers
for (let i = 0; i < 10; i += 1) {
    print("fib(" + i + ") = " + fibonacci(i));
};

// Output:
// fib(0) = 0
// fib(1) = 1
// fib(2) = 1
// fib(3) = 2
// fib(4) = 3
// fib(5) = 5
// fib(6) = 8
// fib(7) = 13
// fib(8) = 21
// fib(9) = 34
```

---

## Further Reading

- [Complete Grammar (BNF)](GRAMMAR.md) - Formal language specification
- [Examples](../examples/) - Comprehensive example programs
- [Main README](../README.md) - Project overview and setup

---

**Rustphy Version:** 0.1.0
**Last Updated:** April 2026
