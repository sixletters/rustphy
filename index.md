---
layout: default
title: Rustphy
---

# Rustphy

**A Rust-based programming language with WebAssembly support**

*By Harris Maung*

---

## 👋 Welcome!

This is my playground for exploring programming language design, compiler engineering, and WebAssembly. Here you'll find:

- 📚 Learning resources and tutorials
- 🔨 Implementation notes and technical deep-dives
- 💡 Experiments and side projects
- 📝 Documentation of my learning journey

---

## 🚀 About Rustphy

Rustphy is a programming language implementation written in Rust, featuring:
- Bytecode compilation
- WebAssembly (WASM) support  
- Frontend compiler with optimization passes
- Educational focus on compiler design

**Status:** Active development & learning phase

---

## 📚 Learning Series

### [MIT OpenCourseWare - Compiler Engineering](learning/compiler_course/)

I'm working through MIT's 6.035 Computer Language Engineering course and documenting everything as digestible, article-style posts. Each lecture includes clear explanations, diagrams, code examples, quiz questions, and practice problems.

#### Available Lectures

**[Lecture 1: Introduction to Compilers](learning/compiler_course/lecture_1_intro/data.md)**
- Compiler pipeline architecture (Lexer → Parser → AST → IR → Optimizer → Code Gen)
- 8 optimization techniques with step-by-step examples
- Assembly code comparison showing 4x performance improvement
- Interactive quizzes and practice problems

**[Lecture 2: Lexical Analysis and Regular Expressions](learning/compiler_course/lecture_2/data.md)**
- Language definition: lexical structure, syntactic structure, and semantics
- Generative vs recognition approaches to defining languages
- Regular expressions and their properties
- Finite automata (DFA and NFA) theory and implementation
- Converting between regex, NFA, and DFA

**[Lecture 3: Shift-Reduce Parsing](learning/compiler_course/lecture_3/data.md)**
- Context-free grammars, derivations, and handles
- Pushdown automata and why stacks matter for parsing
- Shift vs. reduce operations with a full worked trace
- Coding challenges: `a^n b^n`, balanced brackets, table-driven parsing

[**View all lectures →**](learning/compiler_course/)

### [Building Rustphy: A Compiler From Scratch](learning/building_rustphy/)

A narrative, code-first course that follows the **actual commit history** of this repository — from the first lexer to closures, escape analysis, and a WebAssembly backend. Every design decision is explained with real source code, including the parts that are still broken.

#### Chapters

**[Part 0: Why I Built a Programming Language](learning/building_rustphy/part_00_why_a_language.md)**
- Motivation, language design goals, and why Rust
- One frontend, two backends: bytecode VM and WebAssembly
- The real commit timeline and how features drove architecture

**[Part 1: Teaching the Compiler to Read](learning/building_rustphy/part_01_lexer.md)**
- Hand-written lexer with one-byte lookahead
- The `3.14` vs `0..10` problem: floats vs. ranges
- Keyword recognition, string escapes, and error handling

**[Part 2: Giving the Language Structure](learning/building_rustphy/part_02_parser_and_ast.md)**
- AST design with Rust enums and `Box`
- Pratt parsing: a 10-line precedence engine
- Desugaring `x += 5` and lowering `a.b` to `a["b"]`

**[Part 3: From Trees to Bytecode](learning/building_rustphy/part_03_bytecode_compiler.md)**
- Custom stack-machine instruction set
- Backpatching forward jumps for conditionals and loops
- Function bodies inline in the instruction stream

**[Part 4: Building the Virtual Machine](learning/building_rustphy/part_04_virtual_machine.md)**
- Operand stack, runtime stack, and program counter
- Lexical environments with `Rc<RefCell<...>>`
- Closures as code address + captured environment

**[Part 5: Closures Force Us to Think Like Compiler Engineers](learning/building_rustphy/part_05_closures_and_escape_analysis.md)**
- Symbol tables: names vs. bindings
- Escape analysis: which variables need heap allocation
- The bridge between lexical scoping and WASM locals

**[Part 6: Compiling to WebAssembly](learning/building_rustphy/part_06_webassembly.md)**
- 1-bit tagged values on WASM's `i32`
- Dual compilation: direct calls vs. closure trampolines
- Bump allocator, heap objects, and structured control flow

**[Part 7: What Building a Compiler Actually Taught Me](learning/building_rustphy/part_07_lessons.md)**
- One feature (`for-in`) traced through all seven phases
- What I got right, what I'd do differently
- Roadmap: GC, optimizations, type system, JIT

[**Start the course →**](learning/building_rustphy/)

### [HashMap Implementation Deep Dive](learning/hashmaps/)

A comprehensive guide to understanding and implementing hash tables - from theory to practice.

#### What You'll Learn

**[Complete HashMap Guide](learning/hashmaps/index.md)**
- What hashmaps are and why they're O(1) on average
- Three fundamental implementation strategies:
  - Linear Search (simple, cache-friendly)
  - Separate Chaining (general-purpose)
  - Open Addressing (memory-efficient)
- Hash functions and collision resolution
- Load factor analysis and performance tradeoffs
- Advanced topic: Clustering and its mathematical impact
- Interactive quizzes to test your understanding
- Implementation challenges with full solutions

Perfect for understanding the data structures that power dictionaries in Python, HashMap in Java/Rust, and unordered_map in C++.

---

## 🛠️ Project Resources

- **[GitHub Repository](https://github.com/sixletters/rustphy)** - Source code and examples
- **Documentation** - Language specs, grammar, and quick reference *(Coming to this site soon)*
- **[Building Rustphy Course](learning/building_rustphy/)** - A narrative, code-first series following the real compiler from lexer to WebAssembly
- **[MIT 6.035 Lectures](learning/compiler_course/)** - Compiler theory explained through articles and challenges

---

## 🎯 What's Next?

I'm actively working on:
- Completing the MIT compiler series
- Building out Rustphy's feature set
- Adding more educational content
- Documenting implementation decisions

---

## 🤝 Get Involved

This is a learning-in-public project. Feel free to:
- ⭐ Star the [repo](https://github.com/sixletters/rustphy)
- 💬 Open issues with questions or suggestions
- 🐛 Submit PRs if you spot mistakes
- 📢 Share if you find it helpful

---

## 📖 External Resources

- [MIT 6.035 OpenCourseWare](https://ocw.mit.edu/courses/6-035-computer-language-engineering-spring-2010/)
- [Crafting Interpreters by Robert Nystrom](https://craftinginterpreters.com/)
- [LLVM Documentation](https://llvm.org/docs/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)

---

*Building in public, learning every day* ✨
