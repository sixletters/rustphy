---
layout: default
title: MIT OpenCourseWare - Compiler Engineering Series
---

# MIT OpenCourseWare - Compiler Engineering Series

**By Harris Maung**

_Based on MIT 6.035 - Computer Language Engineering_

---

## About This Series

I'm working through MIT's Computer Language Engineering course and documenting everything in digestible, article-style posts. Each lecture includes:

- 📖 Clear explanations with real-world examples
- 🎨 Diagrams and visualizations
- 💻 Code walkthroughs and optimizations
- ✅ Quiz questions to test understanding
- 🏋️ Practice problems with detailed solutions

---

## Lecture Series

### [Lecture 1: Introduction to Compilers](lecture_1_intro/data.md)

**Topics covered:**

- Why study compilers?
- Programming language requirements (precision, expressiveness, abstraction)
- Anatomy of a compiler pipeline (Lexer → Parser → AST → IR → Optimizer → Code Gen)
- What are AST and IR? (with detailed examples)
- 8 compiler optimization techniques:
  - Constant propagation & folding
  - Dead code elimination
  - Common subexpression elimination
  - Loop invariant code motion
  - Strength reduction (bit shifts)
  - Induction variable optimization
- Assembly code comparison showing **4x performance improvement**
- Quiz questions + 3 practice problems with solutions

[**📖 Read Lecture 1 →**](lecture_1_intro/data.md)

---

### [Lecture 2: Lexical Analysis and Regular Expressions](lecture_2/data.md)

**Topics covered:**

- Regular expressions and their building blocks
- Finite-state automata (FSA) - NFAs vs DFAs
- Converting regex to NFA (Thompson's construction)
- NFA to DFA conversion (subset construction)
- Understanding non-determinism and ε-transitions
- Real-world regex engine implementations
- Context-free grammars and why regex isn't enough
- Parse trees, ambiguous grammars, and operator precedence
- Concrete vs abstract syntax
- Quiz questions + 5 practice problems + 7 coding challenges!

[**📖 Read Lecture 2 →**](lecture_2/data.md)

---

### Lecture 3: Parsing _(Coming soon...)_

**Topics:** Context-free grammars, top-down vs bottom-up parsing, AST construction

---

### Lecture 4: Semantic Analysis _(Coming soon...)_

**Topics:** Type checking, symbol tables, scope resolution

---

### Lecture 5: Code Generation _(Coming soon...)_

**Topics:** Instruction selection, register allocation, code emission

---

## How to Use This Series

1. **Read sequentially** - Each lecture builds on previous concepts
2. **Try the quizzes** - Test your understanding before moving on
3. **Do the practice problems** - Hands-on coding solidifies learning
4. **Take your time** - Compilers are complex; don't rush
5. **Ask questions** - Open an issue if something isn't clear

---

## Additional Resources

- [MIT 6.035 OpenCourseWare](https://ocw.mit.edu/courses/6-035-computer-language-engineering-spring-2010/) - Official course materials
- [Dragon Book](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools) - Classic compiler textbook
- [Crafting Interpreters](https://craftinginterpreters.com/) - Hands-on interpreter/compiler book
- [LLVM Documentation](https://llvm.org/docs/) - Modern compiler infrastructure

---

[← Back to Home](/)
