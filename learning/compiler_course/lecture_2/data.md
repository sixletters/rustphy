# Lecture 2 - Lexical Analysis and Regular Expressions

**Author: Harris Maung**

_Based on MIT OpenCourseWare 6.035 - Computer Language Engineering_

---

Hey everyone! Today I'll be talking about lexical analysis and regular expressions. Wow, those are big words! But do we really know what they mean and what problem we're trying to solve?

## The Language Definition Problem

How do we define a programming language? What makes a language? Without a proper way to define it, any jibberish could be considered a programming language!

We usually break it into three levels:

### 1. Lexical Structure

This identifies **words** in a language (each word is a sequence of characters). Think of this as picking out words in an English sentence. For example, in the sentence "This is a fish," the words "this", "is", "a", and "fish" are the tokens!

**Important:** The lexical structure is NOT the alphabet itself! It's the **rules/patterns** that define what valid words/tokens look like.

Think of it this way:

**The alphabet** = the raw building blocks (individual characters)

- In English: a, b, c, d, ..., z
- In programming: a-z, 0-9, symbols like `+`, `-`, `*`, `(`, `)`, etc.

**The lexical structure** = the rules for how those characters can be combined into valid words/tokens

- In English: "a word is a sequence of letters separated by spaces or punctuation"
- In programming:
  - "An identifier must start with a letter, then can have letters/digits/underscores" → `[a-zA-Z][a-zA-Z0-9_]*`
  - "A number is one or more digits" → `[0-9]+`

### 2. Syntactic Structure

If the lexical structure defines how words can be formed, the syntactic structure defines how **sentences** can be formed!

For example, in English, you have grammar rules to put words together to make valid sentences:

- ✅ "This is a fish" makes sense
- ❌ "fish this a is" makes no sense!

Whenever you write programs and see a **"syntax error"**, this is exactly what it means - you've got valid tokens in an invalid order!

### 3. Semantics

This defines whether the sentence actually **makes sense** - does it have valid meaning?

**Syntactically correct but semantically nonsense:**

```
"Colorless green ideas sleep furiously"
```

- ✅ Grammar is perfect (adjective + adjective + noun + verb + adverb)
- ❌ Meaning is nonsense (how can ideas be colorless AND green? How do ideas sleep?)

**Semantics checks things like:**

- Type checking (are you adding int + int, not int + string?)
- Scope checking (is this variable declared? Is it in scope?)
- Function signatures (right number/types of arguments?)
- Return types (does the function return what it promises?)
- Logical consistency (did you initialize before using?)

**In short:** Semantics validates that the syntactically-correct program actually **means something valid**!

---

## Two Approaches to Defining Languages

There are two main ways to define a language:

### 1. Generative Approach

Grammars or regexes that **generate** all valid strings in the language! You define a pattern (like a regex), and from that pattern you can produce/derive any valid string that matches it.

### 2. Recognition Approach

Automata that **recognize** whether a string is valid! You take an input string and the automaton tells you: "Yes, this is valid" or "No, this isn't valid."

**In practice:**

- You **write** the language spec using the **generative approach** (regex/grammar)
- You **implement** the compiler using the **recognition approach** (automata/parser)

**Think of it like:**

- **Generative** = Recipe that can create all valid dishes
- **Recognition** = Food inspector that checks if a dish follows the recipe

---

## What Is a Regex?

We've been saying "regex" a lot! So what is it?

A **regular expression (regex)** is a pattern that describes a set of strings. It's a formal way to specify what valid tokens look like in your language.

## Building Blocks

Given an alphabet (set of valid characters), regular expressions are built from rules:
| Construct | Notation | Meaning |
| ---------------- | ----------------- | ------------------------------------------------- |
| Empty string | ε (epsilon) | Matches the empty string |
| Single character | `a` (where a ∈ Σ) | Matches that character |
| Sequence | `r1r2` | Regex r1 followed by r2 |
| Choice | `r1 \| r2` | Either r1 or r2 |
| Kleene star | `r*` | Zero or more repetitions: ε \| r \| rr \| rrr ... |
| Grouping | `(r)` | Parentheses for precedence |

### Examples

**Alphabet:** Σ = {0, 1, .}

| Regex                   | Language Description                                            |
| ----------------------- | --------------------------------------------------------------- |
| `(0\|1)*.(0\|1)*`       | Binary floating-point numbers (e.g., "1.0", "101.001")          |
| `1*(01*01*)*`           | Strings with an even number of zeros                            |
| `[a-zA-Z][a-zA-Z0-9_]*` | Identifiers (start with letter, then letters/digits/underscore) |

---

## Generating Strings from Regular Expressions

How do we generate strings from regular expressions?

We follow a set of **derivation rules**! Think of it as a game - the rules show you what steps/moves you can make at each point!

### Derivation Rules

1. **`r1 | r2 → r1`** - Choose the left alternative
2. **`r1 | r2 → r2`** - Choose the right alternative
3. **`r* → rr*`** - Expand Kleene star (one or more repetitions)
4. **`r* → ε`** - Kleene star becomes empty (zero repetitions)

### Example: Deriving "1.0" from `(0 | 1)*.(0|1)*`

```
(0 | 1)*.(0|1)*              (starting pattern)
    ↓ rule 3: expand first *
(0 | 1)(0 | 1)*.(0|1)*
    ↓ rule 2: choose 1
1(0|1)*.(0|1)*
    ↓ rule 4: * becomes ε (empty)
1.(0|1)*
    ↓ rule 3: expand second *
1.(0|1)(0|1)*
    ↓ rule 4: * becomes ε
1.(0|1)
    ↓ rule 1: choose 0
1.0                           (final string!)
```

**Key insights:**

- You are NOT reducing the pattern! You're **deriving** or **generating** a concrete string from the abstract pattern.
- Is generation deterministic? **Absolutely not!** If you chose a different move/rule at any point, the resulting string may be completely different. Different rule applications in different orders yield different final strings!

---

## Key Terminology

**Language** - The set of all strings generated by a regular expression.

⚠️ **Important note:** This is NOT the same as a "programming language"! It's simply an overloaded term. At the lexical level, a "language" is just a **set of valid tokens** - nothing more. We'll talk about full programming languages later when we get to syntax and semantics!

**Token** - A single valid string in the language.

Languages can be **countably infinite** (unbounded number of strings).

---

## Finite-State Automata (FSA)

Now that you know how to generate all strings using the generative approach, what about the **recognition** approach? What about automata?

Introducing the **Finite-State Automata (FSA)**!

### Definition

An **FSA** is a mathematical model of computation - a state machine that reads input and decides whether to accept or reject it.

Think of it as a robot that takes in an input string and decides: "Is this acceptable or not?"

### Components

An FSA consists of:

1. **Alphabet** - Valid characters/input symbols in a string
2. **Finite set of states** - Nodes in the state diagram
3. **Start state** - Where the machine begins
4. **Accept state(s)** - If the machine ends here, the input is accepted
5. **Transitions** - Edges labeled with input symbols

You'll understand this better with an example!

### Example: FSA for `(0|1)*.(0|1)*`

```
        0,1           .           0,1
      ┌───┐                     ┌───┐
      │   │                     │   │
      ↓   │                     ↓   │
    ┌─────┴─┐       .        ┌──────┴┐
    │ Start │──────────────→ │ Accept│
    └───────┘                └───────┘
```

**How it works:**

1. Start in the **Start state**
2. Self-loops labeled "0" and "1" → read any number of 0s/1s and stay in Start
3. Transition labeled "." → when you see a period, move to Accept state
4. Self-loops labeled "0" and "1" on Accept → read any number of 0s/1s and stay there
5. If you end in **Accept state** → string is accepted ✅

### Accepted Example: "1.0"

```
Input: "1.0"

Step 1: Start in Start state
Step 2: Read '1' → self-loop, stay in Start
Step 3: Read '.' → transition to Accept state
Step 4: Read '0' → self-loop, stay in Accept
Step 5: End in Accept state → ACCEPTED! ✅
```

### Rejected Example: "10" (no period)

```
Input: "10"

Step 1: Start in Start state
Step 2: Read '1' → stay in Start (self-loop)
Step 3: Read '0' → stay in Start (self-loop)
Step 4: End in Start state (NOT accept state)
Result: REJECTED ❌
```

### Other Rejected Inputs

- **Empty string `""`** - Never reaches Accept state
- **Two periods `"1.0.1"`** - No transition for '.' from Accept state
- **Wrong alphabet `"1a0"`** - No transition for 'a'

---

## How FSAs Work

**Algorithm for running a string through an automaton:**

1. **Initialize:** current state = start state, current position = first character
2. **Repeat:**
   - Match current character against transitions from current state
   - If transition exists → move to next state and advance to next character
   - If no transition exists → **REJECT**
3. **If you reach the end of the string:**
   - If current state is an accept state → **ACCEPT**
   - Otherwise → **REJECT**

---

## Regex vs. Automata

| Aspect   | Regular Expression                   | Finite-State Automaton                            |
| -------- | ------------------------------------ | ------------------------------------------------- |
| Purpose  | **Generate** all strings in language | **Recognize** if a specific string is in language |
| Use case | Language definition                  | Implementation                                    |
| Form     | Pattern notation                     | State machine                                     |

**In practice:** We usually define a language using the **generative approach** (regex) and implement it using the **recognition approach** (automata)!

Since we define languages using regex but implement them using automata, how do we convert from regex to automata?

---

## Converting Regex to Automata

### Strategy: Structural Induction

We use something called **structural induction**. For those who don't know, induction is a proof technique. There are two types: **mathematical induction** (on numbers) and **structural induction** (on recursive structures). We're only concerned with structural induction here.

**Induction** = reasoning from specific cases to general rules

Think of it as **building up** from small, simple truths to bigger, more complex truths.

**How structural induction works:**

1. **Base cases:** Show how to handle the simplest building blocks (like a single character `a` or empty string `ε`)
2. **Inductive cases:** Show how to combine smaller pieces (if you can convert `r1` and `r2` to automata, show how to convert `r1|r2`, `r1r2`, and `r*`)
3. **Conclusion:** You can now convert ANY regex to an automaton!

**Key idea:** Build automata compositionally from regex building blocks.

**Assumption:** Every sub-regex converts to an automaton with:

- One start state
- One accept state

**Goal:** Show how to convert each regex constructor into an automaton.

### Basic Constructs

#### 1. Empty String (ε)

```
Regex: ε

Automaton:
    ε
  ●────→◎
Start   Accept
```

#### 2. Single Character (a)

```
Regex: a

Automaton:
    a
  ●────→◎
Start   Accept
```

### Compound Constructs

#### 3. Sequence (r1r2)

```
Regex: r1r2

Automaton:
  ●────→◎  ε  ●────→◎
  Start₁  Accept₁  Start₂  Accept₂
           (merge these)

Result:
  ●────→○────→◎
  Start  (r1)  (r2)  Accept
```

**Construction:**

1. Build automaton for r1
2. Build automaton for r2
3. Merge r1's accept state with r2's start state using ε-transition

#### 4. Choice (r1 | r2)

```
Regex: r1 | r2

Automaton:
           ε   ●────→◎  ε
          ┌───→ (r1)  ───┐
          │              │
    ●─────┤              ├────→◎
   Start  │              │    Accept
          └───→ (r2)  ───┘
           ε   ●────→◎  ε
```

**Construction:**

1. Create new start state with ε-transitions to both r1 and r2 start states
2. Add ε-transitions from both r1 and r2 accept states to a new single accept state

#### 5. Kleene Star (r\*)

```
Regex: r*

Automaton:
           ┌─────ε─────┐
           ↓           │
    ●─────→●────→◎────→◎
   Start    (r)  Accept
    │                  ↑
    └────────ε─────────┘
```

**Construction:**

1. Create new start state
2. ε-transition from new start to old start (for r)
3. ε-transition from new start directly to new accept (for ε, zero repetitions)
4. ε-transition from r's accept back to r's start (for repetition)
5. ε-transition from r's accept to final accept

---

## Putting It All Together: Thompson's Construction Example

Now let's see how this all works in practice! We'll convert the regex `(a|b)*c` to an NFA step-by-step using Thompson's construction.

**Our regex:** `(a|b)*c`

**The breakdown:**

- Inner: `a|b` (choice between a and b)
- Middle: `(a|b)*` (Kleene star - zero or more of the choice)
- Outer: `(a|b)*c` (sequence - the star part followed by c)

### Step 1: Build Base Cases for `a` and `b`

First, we build the simplest automata for single characters:

**Automaton for `a`:**

```
    a
  ●────→◎
  s0    s1
```

**Automaton for `b`:**

```
    b
  ●────→◎
  s2    s3
```

Easy! Each is just a start state, a transition on the character, and an accept state.

### Step 2: Combine with Choice - `a|b`

Now we use the **choice construction** to combine these:

1. Create a new start state `s4`
2. Add ε-transitions from `s4` to both `s0` (start of a) and `s2` (start of b)
3. Create a new accept state `s5`
4. Add ε-transitions from `s1` (accept of a) and `s3` (accept of b) to `s5`

**Result:**

```
           ε    ●─a→◎  ε
          ┌────→s0──s1───┐
          │              │
    ●─────┤              ├────→◎
   s4     │              │    s5
  Start   └────→s2──s3───┘  Accept
           ε    ●─b→◎  ε
```

**What this does:** From `s4`, we can take either path (via ε-transitions), read either 'a' or 'b', and end up at `s5`. Perfect!

### Step 3: Apply Kleene Star - `(a|b)*`

Now we use the **Kleene star construction** on our `(a|b)` automaton:

1. Create new start state `s6`
2. ε-transition from `s6` to `s4` (to enter the loop)
3. ε-transition from `s6` directly to new accept `s7` (for zero repetitions - ε)
4. ε-transition from `s5` back to `s4` (to repeat the loop)
5. ε-transition from `s5` to `s7` (to exit the loop)

**Result:**

```
           ┌────────ε────────┐
           │                 ↓
    ●──ε──→●───[a|b automaton]──→◎──ε──→◎
   s6     s4                    s5       s7
    │                                    ↑
    └──────────────ε─────────────────────┘
```

**What this does:**

- Take the ε-transition from `s6` to `s7` → accept empty string (zero repetitions)
- OR enter the (a|b) loop, read a's and b's as many times as we want, then exit to `s7`

### Step 4: Build Base Case for `c`

Simple single character again:

```
    c
  ●────→◎
  s8    s9
```

### Step 5: Sequence - `(a|b)*c`

Finally, we use the **sequence construction** to connect `(a|b)*` with `c`:

1. Merge `s7` (accept state of `(a|b)*`) with `s8` (start state of `c`) using an ε-transition
2. The new accept state is `s9`

**Final NFA for `(a|b)*c`:**

```
           ┌────────ε────────┐
           │                 ↓
    ●──ε──→●───[a|b automaton]──→◎──ε──→●──c──→◎
  Start   s4                    s5      s8      Accept
    │                                            ↑
    └──────────────ε─────────────────────────────┘
```

### Testing Our NFA

Let's verify it works:

**Input: "c"**

- Start → ε-transition directly to 'c' part → read 'c' → Accept ✅

**Input: "ac"**

- Start → ε-transition to (a|b) loop → read 'a' → ε-transition to 'c' part → read 'c' → Accept ✅

**Input: "abc"**

- Start → enter loop → read 'a' → loop back → read 'b' → exit loop → read 'c' → Accept ✅

**Input: "ab"** (no 'c' at end)

- Start → enter loop → read 'a', 'b' → end of input, but we're not in an accept state → Reject ❌

**Input: "d"** (wrong character)

- No transition for 'd' from any state → Reject ❌

Perfect! Our NFA correctly recognizes strings matching `(a|b)*c`!

### Key Takeaway

Notice how we **built up** from simple pieces:

1. Base cases: single characters `a`, `b`, `c`
2. Combined: choice `a|b`
3. Extended: Kleene star `(a|b)*`
4. Finalized: sequence `(a|b)*c`

**This is structural induction in action!** We proved we can convert ANY regex to an NFA by showing:

- Base cases work (single characters, ε)
- Compound operations work (sequence, choice, Kleene star)
- Therefore, ANY combination works!

---

## Important Notes

### Regex vs. Derivation Rules

- **Regex** = The pattern itself (e.g., `(0|1)*.(0|1)*`)
- **Derivation rules** = Mathematical tools for proving what strings match (NOT part of the regex)

**Analogy:**

- Regex is like a math formula `x² + 2x + 1`
- Derivation rules are like algebra rules for manipulating it
- Automaton is like a program that evaluates the formula

### When Converting Regex → Automaton

You're converting **the pattern**, not the derivation rules.

The derivation rules help you **understand** what strings the regex accepts, but the automaton is built from the **pattern structure** itself.

---

## Summary (So Far!)

| Concept                | Purpose                                         |
| ---------------------- | ----------------------------------------------- |
| **Regular Expression** | Compact notation for describing token patterns  |
| **Derivation Rules**   | Tools for generating example strings            |
| **Finite Automaton**   | Executable machine for recognizing tokens       |
| **Language**           | Set of all valid strings (at the lexical level) |
| **Token**              | A single valid string in the language           |

---

**To be continued...** Next up: NFAs vs DFAs, subset construction, and how to actually implement this in code!

_Stay tuned for part 2!_ 🚀

---

For FSAs, there are two main types: **DFA** and **NFA**! Are you confused yet? Fret not! DFA stands for **Deterministic Finite Automata** while NFA is **Non-deterministic Finite Automata**!

Here's a technical breakdown of the differences:

## DFA vs NFA: Key Differences

| Feature                     | DFA                   | NFA                               |
| --------------------------- | --------------------- | --------------------------------- |
| **ε-transitions**           | ❌ No ε-transitions   | ✅ Can have ε-transitions         |
| **Transitions per symbol**  | Exactly one (or zero) | Zero, one, or multiple            |
| **States during execution** | In exactly ONE state  | In MULTIPLE states simultaneously |
| **Implementation**          | Simple (table lookup) | Complex (track sets of states)    |

**Note:** ε-transitions (epsilon transitions) are "free transitions"! They allow you to move from one state to another without consuming any input character.

But the main idea is that in an NFA you can be in **multiple states at once**, whereas in a DFA, you can only be in **one state at a time**.

**What does it mean for an NFA to be "in multiple states at once"?** This is the hardest concept to grasp! Let's break it down.

- **NFA/DFA** = The entire state machine (automaton)
- **State** = A single node/circle in the machine
- **"Being in multiple states"** = The NFA execution can be at multiple nodes simultaneously

Let's use an example to illustrate this!

### Example: NFA with ε-transitions

```
         ε
    ┌─────────→○─ a →◎
    │          2     Accept
    │
    ●  (Start)
    0
    │
    │    ε
    └─────────→○─ a →◎
               3     Accept
```

**Components:**

- State 0 = Start state
- State 2 = Intermediate state (has 'a' transition to accept)
- State 3 = Intermediate state (has 'a' transition to accept)
- Two ε-transitions from state 0

**Processing input "a":**

#### Step 1: Start

```
Current states: {0}
Remaining input: "a"
```

#### Step 2: Follow ALL ε-transitions

```
From state 0 → can ε to state 2
From state 0 → can ε to state 3

Current states: {0, 2, 3}  ← THREE states at once!
Remaining input: "a" (haven't consumed anything yet!)
```

**Key insight:** We take BOTH ε-transitions simultaneously - no choosing! This means that at that point, I am at all three states at once! It also means that any transition out of any of the states in my set becomes available to me!

#### Step 3: Read 'a' from ALL current states

```
From state 0: no 'a' transition → this path dies
From state 2: 'a' → accept state ✅
From state 3: 'a' → accept state ✅

Current states: {accept}
Remaining input: ""
```

#### Step 4: Accept

```
We ended in an accept state → ACCEPTED! ✅
```

---

## Understanding Non-Determinism

Now why is it called "determinism" and "non-determinism," and what does it really mean?

**It does NOT mean:**

- Random choice
- Picking one path
- Guessing

**It DOES mean:**

- Exploring **all paths simultaneously**
- No choosing - take every path at once
- Accept if **ANY** path succeeds, regardless of which path it was!

Here are some good mental models:

#### Model 1: Parallel Universes (Conceptual)

**DFA (Deterministic):**

```
You at state 0 → make a transition → go to state 1
One timeline. One path.
```

**NFA (Non-deterministic):**

```
You at state 0 → the universe splits into TWO copies
- Copy 1 stays at state 0
- Copy 2 goes to state 1

Both versions exist simultaneously!
```

**"Being simultaneously in states {0, 1}" means:**

- There are TWO versions of you
- Version 1 is at state 0
- Version 2 is at state 1
- Both are real and both continue processing

#### Model 2: BFS Graph Search (Computational)

NFAs work just like **breadth-first search**:

```python
# BFS on a graph
current_level = [start_node]

for step in steps:
    next_level = []
    for node in current_level:
        next_level.extend(node.neighbors)
    current_level = next_level

# NFA processing
current_states = {start_state}

for char in input:
    next_states = set()
    for state in current_states:
        next_states.update(transitions[state][char])
    current_states = next_states
```

**Same pattern:**

- Process all nodes/states at current level
- Move to ALL reachable nodes/states
- Continue level by level

Now intuitively, this makes an NFA harder to implement and more complex since at any point you can be in multiple states at once! DFAs are easy to implement since it's just a simple table lookup for the transitions!

**The tradeoff:**

- DFA may be **exponentially larger** than NFA (more states)
- But execution is much faster (deterministic)

---

## NFA to DFA Conversion (Subset Construction)

Now let's run through the algorithm to convert an NFA to a DFA! We create a DFA where **each DFA state represents a set of NFA states**.

Here's an example!

**NFA states:** {0, 1, 2}

**DFA states we build:**

- DFA State A = represents NFA states {0}
- DFA State B = represents NFA states {1, 2}
- DFA State C = represents NFA states {0, 1}
- ... (one DFA state for each possible subset)

**Each DFA state is a "snapshot" of which NFA states could be active.**

Now if you notice, the same NFA state (like state 1) can appear in multiple DFA states! And that is completely fine - we are simply converting the set of states to a single state!

**Why does this work?** Instead of tracking multiple states, we track ONE state that represents the set. For example, instead of me being in states {1, 2}, I am in DFA State B (which represents {1, 2})!

Now how do we do the conversion? Let's run through Thompson's construction! No, not Klay Thompson! It's called the **McNaughton–Yamada–Thompson algorithm**!

---

<details markdown="1">
<summary><strong>📖 Extra Knowledge: Real-World Implementation</strong> (Click to expand)</summary>

## Real-World Implementation: How Regex Engines Actually Work

All this NFA/DFA theory isn't just academic - **this is exactly how regex matching is implemented in practice!**

### The Pipeline

```
Your regex pattern
    ↓
Convert to NFA (Thompson's construction)
    ↓
Convert to DFA (subset construction)
    ↓
Optimize DFA (minimize states)
    ↓
Generate code (state machine implementation)
    ↓
Use it to match strings!
```

### Example: Building a Regex Matcher

Let's implement a simple regex matcher for the pattern `(a|b)*c`:

#### Step 1: The DFA

```
     a,b
    ┌───┐
    ↓   │
  ┌─────┴┐    c    ┌────────┐
  │ Start│────────→│ Accept │
  └──────┘         └────────┘
```

#### Step 2: State Transition Table

```python
# State transition table for (a|b)*c
transitions = {
    'start':  {'a': 'start', 'b': 'start', 'c': 'accept'},
    'accept': {}  # no transitions from accept state
}
```

#### Step 3: Implementation

```python
def match_regex(input_string):
    """Match input against regex (a|b)*c"""
    state = 'start'

    for char in input_string:
        if char in transitions[state]:
            state = transitions[state][char]
        else:
            return False  # No valid transition - reject!

    return state == 'accept'

# Test cases
print(match_regex("aabbc"))   # True - ends in accept
print(match_regex("abc"))     # True
print(match_regex("c"))       # True - zero (a|b)s, then c
print(match_regex("ab"))      # False - no 'c' at end
print(match_regex("abcd"))    # False - 'd' has no transition
```

**That's it!** The regex is now a state machine that processes input character by character.

### How Real Compilers Use This: Lexer Generation

When you build a compiler scanner (lexer), you define tokens using regex:

**Token definitions:**

```
INTEGER:    [0-9]+
IDENTIFIER: [a-zA-Z][a-zA-Z0-9_]*
OPERATOR:   +|-|*|/
LPAREN:     (
RPAREN:     )
```

**What a lexer generator (like Flex/Lex) does:**

```
1. Takes your regex definitions
2. Builds an NFA for each pattern
3. Combines all NFAs into one big NFA
4. Converts the combined NFA to DFA
5. Minimizes the DFA
6. Generates C/Rust code with state transition tables
```

**Generated scanner code (simplified):**

```c
typedef enum {
    TOKEN_INTEGER,
    TOKEN_IDENTIFIER,
    TOKEN_OPERATOR,
    // ...
} TokenType;

TokenType scan_next_token(char *input, int *pos) {
    int state = START_STATE;
    int start_pos = *pos;

    while (input[*pos] != '\0') {
        // Look up next state in transition table
        state = transition_table[state][input[*pos]];

        if (state == ERROR_STATE) {
            break;
        }
        (*pos)++;
    }

    // Check if we're in an accepting state
    return token_type[state];
}
```

### Different Implementation Strategies

Modern regex engines use different approaches:

#### Approach 1: Pure DFA (Fast & Predictable)

**Used by:**

- `grep`, `awk`, `sed`
- Google's RE2 library
- Rust's `regex` crate
- Lexer generators (Flex, Lex)

**How it works:**

```
Regex → NFA → DFA → Optimized lookup table
```

**Pros:**

- **Guaranteed O(n) time** (where n = input length)
- No backtracking, no catastrophic exponential behavior
- Very fast

**Cons:**

- DFA can be exponentially larger than NFA
- Can't support advanced features like backreferences

#### Approach 2: NFA Simulation with Backtracking

**Used by:**

- Python `re`
- JavaScript regex
- Perl, Ruby, Java `Pattern`

**How it works:**

```
Regex → NFA → Simulate NFA + backtracking features
```

**Pros:**

- More features (lookahead, backreferences, captures)
- Smaller memory footprint

**Cons:**

- **Can be exponentially slow** (catastrophic backtracking)
- Worst-case O(2^n) time complexity

**Example of catastrophic backtracking:**

```python
import re

# This can take FOREVER for long strings!
pattern = r'(a+)+b'
text = 'a' * 30 + 'c'  # No 'b' at end

# NFA with backtracking tries all possible ways to split
# the 'a's, leading to exponential time!
re.match(pattern, text)  # Very slow!
```

#### Approach 3: Hybrid

Some modern engines use both:

- Fast DFA path for simple patterns
- Fall back to NFA simulation for complex features

### Practical Example: Identifier Scanner

Let's build a real token scanner for identifiers `[a-zA-Z][a-zA-Z0-9_]*`:

```python
class IdentifierScanner:
    def __init__(self):
        # DFA states
        self.START = 0
        self.LETTER = 1  # Accept state
        self.ERROR = -1

        # Transition table: state × char → next_state
        self.transitions = {}

    def build_dfa(self):
        """Build DFA for identifier pattern"""
        # From START, letter → LETTER
        self.transitions[(self.START, 'letter')] = self.LETTER

        # From LETTER, letter/digit/underscore → LETTER
        self.transitions[(self.LETTER, 'letter')] = self.LETTER
        self.transitions[(self.LETTER, 'digit')] = self.LETTER
        self.transitions[(self.LETTER, 'underscore')] = self.LETTER

    def char_class(self, ch):
        """Classify character"""
        if ch.isalpha():
            return 'letter'
        elif ch.isdigit():
            return 'digit'
        elif ch == '_':
            return 'underscore'
        else:
            return None

    def scan(self, text):
        """Scan text and return identifier (or None)"""
        state = self.START
        pos = 0

        while pos < len(text):
            ch = text[pos]
            char_type = self.char_class(ch)

            if (state, char_type) in self.transitions:
                state = self.transitions[(state, char_type)]
                pos += 1
            else:
                break

        # Check if we ended in accept state
        if state == self.LETTER and pos > 0:
            return text[:pos]
        return None

# Test it
scanner = IdentifierScanner()
scanner.build_dfa()

print(scanner.scan("hello123"))      # "hello123"
print(scanner.scan("_test"))         # None (doesn't start with letter)
print(scanner.scan("var_name_42"))   # "var_name_42"
print(scanner.scan("123abc"))        # None
```

### Why This Matters

**Every time you use regex, a state machine runs under the hood:**

- `grep "pattern" file.txt` → DFA matching
- JavaScript `/\d+/.test(str)` → NFA simulation
- Compiler lexer tokenizing source code → DFA from combined NFAs

**All the theory we learned (NFA, DFA, ε-transitions, subset construction) is directly implemented in these tools!**

</details>

---

## Key Takeaways

1. **NFAs can be in multiple states** - like parallel universe exploration
2. **All transitions from all current states are applicable** - no choosing, take ALL paths
3. **It's like BFS** - level-by-level exploration of all reachable states
4. **DFA conversion creates one state per NFA state-set** - makes implementation simple
5. **Non-deterministic ≠ random** - it means exploring all possibilities simultaneously

**Next:** We'll see the full NFA→DFA conversion algorithm and work through examples!

## Limitations of Regular Languages

### Why Regular Languages Can't Handle Programming Language Syntax

**The Problem:** Regular languages are **suboptimal** for specifying full programming language syntax.

**Why?** Because they cannot handle constructs with **nested syntax**.

**Examples of nesting:**

- Arithmetic: `(a + (b - c)) * (d - (x - (y - z)))`
- Conditionals: `if (x < y) if (y < z) a = 5 else a = 6 else a = 7`
- Balanced parentheses: `()`, `(())`, `((()))`

### The Core Issue: Finite State = Finite Memory

The core issue is essentially that **with finite state comes finite memory!** Regular languages have a finite number of states (remember the "F" in FSAs?). They have no memory beyond "which state I am in right now."

**Now why is nesting an issue for this?**

When nesting, we need to:

- **Count** how deep we are in the nesting
- **Remember** what to match closing parentheses with
- Handle **arbitrarily and infinitely deep** structures!

Let me show you an example below!

### Concrete Example: Balanced Parentheses

**FSA Challenge:** Let's build an FSA that accepts:

- `()` ✓
- `(())` ✓
- `((()))` ✓
- `(((())))` ✓

But rejects:

- `(` ✗ (not closed)
- `())` ✗ (too many closes)
- `(()` ✗ (not enough closes)

**The Problem:**

- To handle `n` levels of nesting, you need `n+1` states
- Nesting can be **arbitrarily deep** (no fixed limit)
- You would need **infinite states** → Not a finite-state automaton!

Let's build an FSA for balanced parentheses and see where it breaks down!

**For 1 level of nesting max:** `()`, `(()` ❌

We need 2 states (1+1):

```
State 0: "I've seen 0 open parens" (start/accept)
State 1: "I've seen 1 open paren"
```

Transitions:

- State 0 --'('--> State 1 (opened one paren)
- State 1 --')'--> State 0 (closed it, back to balanced)

This accepts: `()` ✓
But breaks on: `(())` ❌ (we're in State 1 after the first `(`, read another `(`, but we have no State 2!)

**For 2 levels of nesting max:** `(())`, `((())` ❌

We need 3 states (2+1):

```
State 0: "0 open parens" (start/accept)
State 1: "1 open paren"
State 2: "2 open parens"
```

Transitions:

- State 0 --'('--> State 1
- State 1 --'('--> State 2
- State 2 --')'--> State 1
- State 1 --')'--> State 0

This accepts: `()`, `(())` ✓
But breaks on: `((()))` ❌ (no State 3!)

**For 3 levels:** Need 4 states (3+1)
**For N levels:** Need N+1 states

**The states represent "depth counters":**

- State 0 = depth 0 (balanced)
- State 1 = depth 1 (one unclosed paren)
- State 2 = depth 2 (two unclosed parens)
- ...
- State N = depth N

**The fatal flaw:** In real programs, nesting can be **unbounded**:

```javascript
// Someone could write arbitrarily deep nesting:
if (x) {
  if (y) {
    if (z) {
      if (a) {
        if (b) {
          // ... as deep as they want!
        }
      }
    }
  }
}
```

There's no fixed maximum N! So we'd need **infinite states** - which violates the "finite" in "Finite-State Automaton"!

**This is why we need CFGs:** They have a **stack** (unbounded memory) to track nesting depth, not just a fixed number of states.

**Key Insight:** Finite-state machines **cannot handle infinitely deep nested recursive structures!**

---

## Solution: Context-Free Grammars (CFG)

### What is a Context-Free Grammar?

A CFG consists of **four components:**

1. **Set of Terminals** - Actual tokens (each defined by a regular expression)

   ```
   { Op, Int, Open, Close, IfKeyword, WhileKeyword }
   ```

2. **Set of Nonterminals** - Placeholder symbols (variables to be expanded)

   ```
   { Start, Expr, Stat }
   ```

   **What's a nonterminal?** Just a symbol (letter or word) representing "something to be expanded"

3. **Set of Productions** - Replacement rules

   ```
   Start → Stat
   Expr → Expr Op Int
   Stat → if Expr then Stat else Stat
   ```

   **What's a production?** A rule you can apply to transform nonterminals

4. **Start Symbol** - The nonterminal where generation begins (usually `Start` or `S`)

### Example Grammar for Arithmetic

The example below shows the grammar for arithmetic!

**Terminals** (defined by regex):

- **Op** = operations (`+`, `-`, `*`, `/`)
- **Int** = integers (one or more digits)
- **Open** = opening parenthesis `(`
- **Close** = closing parenthesis `)`

```
Op = + | - | * | /
Int = [0-9][0-9]*
Open = (
Close = )
```

**Nonterminals** are expressions that can be further expanded:

**Nonterminals:**

```
{ Start, Expr }
```

**Productions:**

```
Start → Expr
Expr  → Expr Op Int
Expr  → Int
Expr  → Open Expr Close
```

---

## CFG: Generation or Validation?

Now you might be wondering: **Is CFG for validation or for generation?**

**Answer: Both!** Production rules work in **both directions** depending on what you're doing.

For example, the rule `Expr → Expr Op Int` can be read in two ways:

## The Production Game: Generating Strings

### Algorithm

**How to generate a valid string from a grammar:**

1. **Start with:** The start nonterminal (e.g., `Start`)
2. **Loop until no more nonterminals:**
   - Choose a nonterminal in the current string
   - Choose a production with that nonterminal on LHS (left-hand side)
   - Replace the nonterminal with the RHS (right-hand side)
3. **Substitute:** Any remaining patterns (terminals) with concrete values
4. **Result:** The generated string is in the language!

### Example Generation

**Grammar:**

```
Start → Expr
Expr  → Expr + Int
Expr  → Int
Int   → 1 | 2 | 3
```

**Generating "1 + 2":**

```
Step 1: Start
Step 2: Start → Expr          =  Expr
Step 3: Expr → Expr + Int     =  Expr + Int
Step 4: Expr → Int            =  Int + Int
Step 5: Int → 1               =  1 + Int
Step 6: Int → 2               =  1 + 2

Final: "1 + 2"
```

## Understanding Grammars: Generative vs Recognitive

### The Dual Nature

A production like `Stat → if Expr then Stat else Stat` can be understood **two ways:**

#### Generative (Top-Down)

**Read as:** "A `Stat` can **become** `if Expr then Stat else Stat`"

**Use when:**

- Learning what strings are valid
- Generating test cases
- Understanding "what can I write?"

**Think:** "I start with placeholders and **create** valid strings"

#### Recognitive (Bottom-Up)

**Read as:** "If you see `if Expr then Stat else Stat`, it **is** a valid `Stat`"

**Use when:**

- Parsing code
- Checking if code is syntactically valid
- Building a parser/compiler

**Think:** "I look at code and **verify** it matches the rules"

**Key Point:** Grammars work **both ways!** The arrow `→` can be read as replacement (generative) or recognition (recognitive).

---

## Parse Trees

### Definition

A **parse tree** visualizes the derivation of a string from a grammar.

**Structure:**

- **Internal nodes:** Nonterminals
- **Leaves:** Terminals
- **Edges:** From nonterminal on LHS of production to symbols on RHS

### Example Parse Tree

**Grammar:**

```
Expr → Expr + Int
Expr → Int
```

**String:** `1 + 2`

**Parse Tree:**

```
      Expr
     / | \
  Expr + Int
   |      |
  Int     2
   |
   1
```

---

## Concrete vs Abstract Syntax

### Concrete Syntax

**Definition:** The actual text you write, including all punctuation, keywords, and formatting to make it unambiguous.

**Example:**

```javascript
if (x < 5) {
  y = 10;
} else {
  y = 20;
}
```

**Contains:**

- Keywords: `if`, `else`
- Punctuation: `(`, `)`, `{`, `}`, `;`
- All the "noise" needed for parsing

### Abstract Syntax

**Definition:** The essential structure of the program - just the meaningful parts, without the "noise."

**Same example, abstract view:**

```
IfStatement
├── Condition: (x < 5)
├── ThenBranch: Assignment(y = 10)
└── ElseBranch: Assignment(y = 20)
```

**What's removed:**

- Keywords (`if`, `else`)
- Punctuation (`()`, `{}`, `;`)
- Anything that doesn't affect meaning

### Concrete Parse Tree vs AST

| Type                           | What It Contains                                           |
| ------------------------------ | ---------------------------------------------------------- |
| **Concrete Parse Tree**        | Every element from the grammar (all keywords, punctuation) |
| **Abstract Syntax Tree (AST)** | Just the meaningful parts (operators, operands, structure) |

### Why Concrete Syntax Needs "Noise"

**Parentheses prevent ambiguity:**

```c
// With parens - clear!
if (x > 0) { print(x); }

// Without parens - ambiguous!
if x > 0 print(x);  // Is this "if (x > 0)" or "if (x > 0 print)"?
```

**Braces show scope:**

```c
// Without braces - ambiguous!
if (x > 0)
    print(x);
    print(y);  // Is this in the if, or after it?
```

**In abstract syntax:** The tree structure already shows what belongs where! No ambiguity, no need for extra punctuation.

### Abstract Syntax Can Be Ambiguous

**Example:** `2 + 3 * 4`

**Two valid abstract syntax trees:**

**Option 1:** `(2 + 3) * 4`

```
Multiply
├── Add
│   ├── 2
│   └── 3
└── 4
```

**Option 2:** `2 + (3 * 4)`

```
Add
├── 2
└── Multiply
    ├── 3
    └── 4
```

Both are valid ASTs! The abstract syntax doesn't specify precedence.

**Concrete syntax resolves this:**

- Precedence rules (`*` before `+`)
- Explicit parentheses: `2 + (3 * 4)`

---

## The Parser

### What is a Parser?

**Purpose:** Converts programs into parse trees

**Two approaches:**

1. **Hand-written parser**
   - Written manually by a programmer
   - Full control over structure and error messages

2. **Parser generator**
   - Accepts a **grammar** as input
   - Produces a **parser** as output
   - Examples: Yacc, Bison, ANTLR

### Practical Problem

**Issue:** Parse trees for complex grammars can be very complicated

**Solution:** Start with an intuitive parse tree (AST) that captures just the essential structure

---

## Ambiguous Grammars

### The Dangling Else Problem

**Consider the statement:**

```
if e1 then if e2 then s1 else s2
```

**Question:** Which `if` does the `else` belong to?

**Parse Tree #1:**

```
if e1 then
    (if e2 then s1 else s2)
```

The `else` belongs to the inner `if`

**Parse Tree #2:**

```
(if e1 then
    if e2 then s1)
else s2
```

The `else` belongs to the outer `if`

**Problem:** The grammar is **ambiguous** - one string has multiple valid parse trees!

### Precedence and Associativity Issues

**Example:** `2 - 3 * 4`

**Problem with pure left associativity:**

- Parsing left-to-right gives: `(2 - 3) * 4` = `-4`
- But we want: `2 - (3 * 4)` = `-10`

**Precedence violation:** Multiplication should bind tighter than subtraction!

**Solution:** Structure the grammar to enforce precedence:

```
Expr → Expr AddOp Term   (addition at top level)
Term → Term MulOp Num    (multiplication binds tighter)
```

---

## 🎯 Quiz Questions

Test your understanding of the concepts covered so far!

### Question 1: Regular Expressions

What does the regex `(a|b)*c` describe?

- A) Any string ending with 'c' that contains only a's and b's before it
- B) A string that must have at least one 'a' or 'b' before 'c'
- C) A string with exactly one 'c' at the end
- D) All strings containing a, b, and c

### Question 2: NFA vs DFA

What's the key difference between an NFA and a DFA?

- A) DFA is faster but NFA is easier to construct
- B) NFA can be in multiple states simultaneously; DFA is always in exactly one state
- C) DFA uses more memory than NFA
- D) NFA cannot recognize all regular languages

### Question 3: ε-transitions

An ε-transition (epsilon transition) allows:

- A) Moving to another state without consuming any input character
- B) Skipping invalid characters in the input
- C) Returning to a previous state
- D) Matching any character

### Question 4: Subset Construction

In the subset construction algorithm, when we have a set S of NFA states:

- A) S becomes a DFA state directly
- B) S is discarded if it's too large
- C) We create a DFA state that represents being simultaneously in all NFA states in S
- D) S must contain only one NFA state to be valid

### Question 5: Real-World Implementation

Modern regex engines like PCRE and JavaScript's regex:

- A) Always use pure DFA for maximum speed
- B) Always use pure NFA for maximum flexibility
- C) Use hybrid approaches (DFA-like for simple patterns, backtracking for complex features)
- D) Convert everything to DFA at compile time

<details markdown="1">
<summary><strong>Answer Key</strong></summary>

1. **A** ✓ - `(a|b)*c` matches any string ending with 'c' that has zero or more a's and b's before it (the `*` allows zero occurrences)

2. **B** ✓ - The fundamental difference: NFA can be in multiple states simultaneously, DFA is always in exactly one state

3. **A** ✓ - ε-transitions are "free moves" without consuming input characters

4. **C** ✓ - S is a temporary set of NFA states, and we create a DFA state to represent being simultaneously in those states

5. **C** ✓ - Modern regex engines use hybrid approaches because pure DFA can't handle backreferences, lookaheads, and other complex features

</details>

---

## 🏋️ Practice Problems

### Problem 1: Drawing an NFA

**Task:** Draw an NFA that recognizes the regex `ab*c` (an 'a', followed by zero or more 'b's, followed by a 'c').

Your NFA should show:

- States (circles)
- Transitions (arrows with labels)
- Start state (arrow pointing to it)
- Accept state(s) (double circle)

<details markdown="1">
<summary><strong>Solution</strong></summary>

**NFA for `ab*c`:**

```
         a          b          c
    ●────────→○────────→○────────→◎
   Start     State1    State1   Accept
              │    ↑
              └────┘
              self-loop on 'b'
```

**Explanation:**

- **Start state** → State1 via 'a' (matches the first 'a')
- **State1** → State1 via 'b' (self-loop for zero or more 'b's from `b*`)
- **State1** → Accept via 'c' (matches the final 'c')

**Accepted strings:** "ac", "abc", "abbc", "abbbc", ...

**Rejected strings:** "bc" (no 'a'), "ab" (no 'c'), "abca" (extra char after 'c')

</details>

---

### Problem 2: NFA Execution

**Task:** Given this NFA for `(a|b)*`:

```
Start → State0 ─a→ State0
        State0 ─b→ State0
        State0 is an accept state
```

Trace the execution for input string **"aba"**:

- Show which state(s) you're in after each character
- Does it accept or reject?

<details markdown="1">
<summary><strong>Solution</strong></summary>

**Execution trace for "aba":**

| Step | Input Position | Current Char | Current State(s) | Action                        |
| ---- | -------------- | ------------ | ---------------- | ----------------------------- |
| 0    | -              | -            | State0 (start)   | Initialize                    |
| 1    | 0              | 'a'          | State0           | Read 'a', self-loop to State0 |
| 2    | 1              | 'b'          | State0           | Read 'b', self-loop to State0 |
| 3    | 2              | 'a'          | State0           | Read 'a', self-loop to State0 |
| 4    | End            | -            | State0 (accept)  | ✅ ACCEPT                     |

**Result:** **ACCEPTED** ✓

State0 is an accept state, and we end in State0, so the string is accepted.

**Other accepted strings:** "a", "b", "aaa", "bbb", "ababab", "" (empty string, already in accept state)

</details>

---

### Problem 3: Identify NFA vs DFA

**Task:** For each automaton below, identify if it's an NFA or DFA and explain why:

**Automaton A:**

- State 0 ─a→ State 1
- State 0 ─b→ State 2
- State 1 ─a→ State 3
- State 2 ─b→ State 3

**Automaton B:**

- State 0 ─a→ State 1
- State 0 ─a→ State 2
- State 1 ─b→ State 3
- State 2 ─b→ State 3

<details markdown="1">
<summary><strong>Solution</strong></summary>

**Automaton A: DFA** ✓

**Reasoning:**

- From State 0: exactly one transition for 'a' (to State 1), exactly one transition for 'b' (to State 2)
- From State 1: exactly one transition for 'a' (to State 3)
- From State 2: exactly one transition for 'b' (to State 3)
- **Every state has at most one transition per symbol** → Deterministic!

**Automaton B: NFA** ✓

**Reasoning:**

- From State 0: **TWO transitions on 'a'** (to both State 1 AND State 2)
- This violates the DFA rule: "exactly one transition per symbol"
- When reading 'a' at State 0, the automaton must explore BOTH paths simultaneously
- **Multiple transitions for the same symbol** → Non-deterministic!

**Key insight:** Even one instance of multiple transitions for the same symbol makes it an NFA.

</details>

---

### Problem 4: NFA to DFA Conversion (Subset Construction)

**Task:** Convert this simple NFA to a DFA:

**NFA:**

- Start state: 0
- State 0 ─a→ {1, 2} (transitions to BOTH states 1 and 2)
- State 1 ─b→ 3
- State 2 ─b→ 3
- Accept state: 3

Show:

1. All DFA states (as sets of NFA states)
2. All transitions between DFA states
3. Which DFA state(s) are accept states

<details markdown="1">
<summary><strong>Solution</strong></summary>

**Step 1: Identify all DFA states**

Each DFA state represents a set of NFA states:

| DFA State | NFA States | Is Accept? |
| --------- | ---------- | ---------- |
| **{0}**   | {0}        | No         |
| **{1,2}** | {1, 2}     | No         |
| **{3}**   | {3}        | Yes ✓      |

**Step 2: Build transitions**

Starting from DFA state **{0}**:

- On 'a': NFA state 0 goes to {1, 2} → DFA state **{1,2}**

From DFA state **{1,2}**:

- On 'b': NFA state 1 goes to 3, NFA state 2 goes to 3 → both lead to {3} → DFA state **{3}**

From DFA state **{3}**:

- No outgoing transitions in the NFA → No transitions in DFA

**Step 3: Complete DFA**

```
DFA States: {0}, {1,2}, {3}
Start state: {0}
Accept state: {3}

Transitions:
  {0}   ─a→ {1,2}
  {1,2} ─b→ {3}
  {3}   (no transitions)
```

**Visualization:**

```
        a          b
  ●───────→○───────→◎
 {0}      {1,2}    {3}
Start              Accept
```

**Testing:**

- Input "ab": {0} ─a→ {1,2} ─b→ {3} ✓ Accept
- Input "a": {0} ─a→ {1,2} (not accept state) ✗ Reject
- Input "b": No transition from {0} on 'b' ✗ Reject

</details>

---

### Problem 5: Write a Simple Matcher

**Task:** Write pseudocode or Python code for a DFA that matches strings ending with **"ing"** (like "running", "coding", "ing").

Your matcher should:

- Process the input character by character
- Return `True` if the string ends with "ing"
- Return `False` otherwise

<details markdown="1">
<summary><strong>Solution</strong></summary>

**Python Implementation:**

```python
def matches_ending_ing(input: str):
    """
    DFA that matches strings ending with "ing"

    States:
    - 'start': Initial state
    - '0': Just saw 'i'
    - '1': Just saw 'in'
    - 'accept': Just saw 'ing'
    """
    transitions = {
        'start':  {'i': '0'},
        '0': {'n': '1', 'i': '0'},      # 'i' restarts pattern
        '1': {'g': 'accept', 'i': '0'}, # 'i' restarts pattern
        'accept': {'i': '0'}             # 'i' restarts pattern
    }

    curr_state = 'start'

    for c in input:
        trans = transitions[curr_state]
        if c not in trans:
            curr_state = 'start'  # Reset on any other character
        else:
            curr_state = trans[c]

    return curr_state == 'accept'


# Test cases
print(matches_ending_ing("running"))   # True ✓
print(matches_ending_ing("coding"))    # True ✓
print(matches_ending_ing("ing"))       # True ✓
print(matches_ending_ing("inging"))    # True ✓ (restarts pattern)
print(matches_ending_ing("ining"))     # True ✓ (restarts pattern)
print(matches_ending_ing("run"))       # False ✗
print(matches_ending_ing("in"))        # False ✗
print(matches_ending_ing("inga"))      # False ✗ (doesn't END with "ing")
print(matches_ending_ing(""))          # False ✗ (empty string)
```

**DFA Visualization:**

```
                i                n                g
    ●─────────→○─────────→○─────────→◎
   Start       0          1        Accept
    ↑          │          │          │
    │          └──────────┘          │
    │          i restarts pattern    │
    │                                │
    └────────────────────────────────┘
              i restarts pattern
```

**Key insights:**

1. **Pattern restart:** When we see 'i' at ANY state, go to state '0' (because 'i' starts a new "ing" pattern)
2. **Reset on invalid chars:** Any character other than expected → go back to 'start'
3. **Must end in accept state:** Only return True if we finish in the 'accept' state

**Why the 'i' → '0' transitions matter:**

Without them, "inging" would fail:

- 'i' → '0'
- 'n' → '1'
- 'g' → 'accept'
- 'i' → without the transition, we'd reset to 'start' and LOSE this 'i'!
- With the transition: 'i' → '0' (restart pattern)
- 'n' → '1'
- 'g' → 'accept' ✓

</details>

---

## 💻 Coding Challenges

Put your knowledge into practice with these hands-on coding problems!

### Challenge 1: NFA Simulator

**Difficulty:** Medium

**Task:** Implement a general NFA simulator that can execute any NFA.

**Requirements:**

```python
class NFA:
    def __init__(self, states, alphabet, transitions, start_state, accept_states):
        """
        states: set of state names
        alphabet: set of valid input symbols
        transitions: dict mapping (state, symbol) -> set of next states
                    Use None as symbol for ε-transitions
        start_state: initial state
        accept_states: set of accepting states
        """
        pass

    def accepts(self, input_string):
        """
        Return True if the NFA accepts the input string, False otherwise.
        Must handle ε-transitions correctly!
        """
        pass
```

**Test cases:**

```python
# NFA for (a|b)*c
nfa = NFA(
    states={'s0', 's1'},
    alphabet={'a', 'b', 'c'},
    transitions={
        ('s0', 'a'): {'s0'},
        ('s0', 'b'): {'s0'},
        ('s0', 'c'): {'s1'}
    },
    start_state='s0',
    accept_states={'s1'}
)

assert nfa.accepts("aabbc") == True
assert nfa.accepts("c") == True
assert nfa.accepts("ab") == False
assert nfa.accepts("abcd") == False
```

**Bonus:** Add a method to visualize the NFA execution step-by-step.

---

### Challenge 2: Subset Construction (NFA to DFA)

**Difficulty:** Hard

**Task:** Implement the subset construction algorithm to convert an NFA to a DFA.

**Requirements:**

```python
def nfa_to_dfa(nfa):
    """
    Convert an NFA to an equivalent DFA using subset construction.

    Input: NFA object (from Challenge 1)
    Output: DFA object with:
        - states: set of DFA states (each state is a frozenset of NFA states)
        - alphabet: same as NFA
        - transitions: dict mapping (state, symbol) -> next_state
        - start_state: DFA start state
        - accept_states: set of DFA accept states

    Must handle ε-transitions correctly by computing ε-closure!
    """
    pass
```

**Algorithm steps:**

1. Compute ε-closure for each NFA state
2. Start with ε-closure of NFA start state as DFA start state
3. For each DFA state and each symbol:
   - Find all reachable NFA states
   - Compute their ε-closure
   - Create/find corresponding DFA state
4. Mark DFA states as accepting if they contain any NFA accept state

**Test:**

```python
# Convert the (a|b)*c NFA to DFA
dfa = nfa_to_dfa(nfa)

# DFA should have same behavior
assert dfa.accepts("aabbc") == True
assert dfa.accepts("c") == True
assert dfa.accepts("ab") == False
```

---

### Challenge 3: Regex to NFA (Thompson's Construction)

**Difficulty:** Hard

**Task:** Implement Thompson's construction to convert a simple regex to an NFA.

**Requirements:**

```python
def regex_to_nfa(regex):
    """
    Convert a regex to an NFA using Thompson's construction.

    Supported operators:
    - Character literals: 'a', 'b', etc.
    - Concatenation: 'ab' means 'a' followed by 'b'
    - Choice: 'a|b' means 'a' or 'b'
    - Kleene star: 'a*' means zero or more 'a's
    - Parentheses: '(ab)*'

    Example: "a(b|c)*d" -> NFA

    Return: NFA object
    """
    pass
```

**Implementation hints:**

1. Parse the regex into an AST (expression tree)
2. Recursively build NFAs for each operator:
   - Base case: single character → simple 2-state NFA
   - Concatenation: connect NFAs with ε-transition
   - Choice: add new start/accept with ε-transitions
   - Kleene star: add loop-back and bypass ε-transitions

**Test:**

```python
nfa = regex_to_nfa("(a|b)*c")
assert nfa.accepts("aabbc") == True
assert nfa.accepts("c") == True
assert nfa.accepts("ab") == False

nfa2 = regex_to_nfa("a(b|c)*d")
assert nfa2.accepts("ad") == True
assert nfa2.accepts("abcd") == True
assert nfa2.accepts("abbbcccbd") == True
assert nfa2.accepts("ab") == False
```

---

### Challenge 4: Balanced Parentheses Parser

**Difficulty:** Medium

**Task:** Write a parser for balanced parentheses using a context-free grammar.

**Grammar:**

```
S → ε
S → ( S )
S → S S
```

**Requirements:**

```python
def parse_balanced_parens(input_string):
    """
    Parse a string of parentheses and return a parse tree.

    Return None if the string is not balanced.

    Parse tree representation:
    - Empty: None
    - (S): ('paren', subtree)
    - S S: ('concat', left_tree, right_tree)
    """
    pass

def is_balanced(input_string):
    """
    Return True if parentheses are balanced, False otherwise.
    """
    return parse_balanced_parens(input_string) is not None
```

**Test:**

```python
assert is_balanced("") == True
assert is_balanced("()") == True
assert is_balanced("(())") == True
assert is_balanced("()()") == True
assert is_balanced("(()(()))") == True

assert is_balanced("(") == False
assert is_balanced("())") == False
assert is_balanced(")(") == False

# Parse tree for "(())"
tree = parse_balanced_parens("(())")
assert tree == ('paren', ('paren', None))
```

**Bonus:** Extend to handle multiple bracket types: `()`, `[]`, `{}`

---

### Challenge 5: Expression Parser with Precedence

**Difficulty:** Hard

**Task:** Build a recursive descent parser for arithmetic expressions with correct operator precedence.

**Grammar:**

```
Expr → Term (('+' | '-') Term)*
Term → Factor (('*' | '/') Factor)*
Factor → NUMBER | '(' Expr ')'
```

**Requirements:**

```python
class ASTNode:
    pass

class BinOp(ASTNode):
    def __init__(self, op, left, right):
        self.op = op
        self.left = left
        self.right = right

    def eval(self):
        """Evaluate the expression"""
        pass

class Number(ASTNode):
    def __init__(self, value):
        self.value = value

    def eval(self):
        return self.value

def parse_expression(tokens):
    """
    Parse a list of tokens into an AST.

    tokens: list of strings like ['2', '+', '3', '*', '4']

    Return: ASTNode (root of AST)
    """
    pass
```

**Test:**

```python
# Test precedence: 2 + 3 * 4 = 14 (not 20)
tokens = ['2', '+', '3', '*', '4']
ast = parse_expression(tokens)
assert ast.eval() == 14

# Test associativity: 10 - 3 - 2 = 5 (left-associative)
tokens = ['10', '-', '3', '-', '2']
ast = parse_expression(tokens)
assert ast.eval() == 5

# Test parentheses: (2 + 3) * 4 = 20
tokens = ['(', '2', '+', '3', ')', '*', '4']
ast = parse_expression(tokens)
assert ast.eval() == 20

# Complex: 2 * 3 + 4 * 5 = 26
tokens = ['2', '*', '3', '+', '4', '*', '5']
ast = parse_expression(tokens)
assert ast.eval() == 26
```

**Bonus:** Pretty-print the AST to visualize the parse tree structure.

---

### Challenge 6: Lexer (Tokenizer) Implementation

**Difficulty:** Medium

**Task:** Build a lexer that tokenizes source code based on regex patterns.

**Requirements:**

```python
class Token:
    def __init__(self, type, value, position):
        self.type = type      # Token type (IDENTIFIER, NUMBER, etc.)
        self.value = value    # Actual text
        self.position = position  # Position in source

class Lexer:
    def __init__(self):
        # Define token patterns (regex)
        self.token_patterns = [
            ('NUMBER',     r'\d+'),
            ('IDENTIFIER', r'[a-zA-Z_][a-zA-Z0-9_]*'),
            ('PLUS',       r'\+'),
            ('MINUS',      r'-'),
            ('TIMES',      r'\*'),
            ('DIVIDE',     r'/'),
            ('LPAREN',     r'\('),
            ('RPAREN',     r'\)'),
            ('ASSIGN',     r'='),
            ('SEMICOLON',  r';'),
            ('WHITESPACE', r'\s+'),  # Skip whitespace
        ]

    def tokenize(self, source_code):
        """
        Tokenize source code.

        Return: list of Token objects (skip WHITESPACE tokens)

        Raise exception if invalid character found.
        """
        pass
```

**Test:**

```python
lexer = Lexer()
tokens = lexer.tokenize("x = 42 + y * 3;")

expected = [
    Token('IDENTIFIER', 'x', 0),
    Token('ASSIGN', '=', 2),
    Token('NUMBER', '42', 4),
    Token('PLUS', '+', 7),
    Token('IDENTIFIER', 'y', 9),
    Token('TIMES', '*', 11),
    Token('NUMBER', '3', 13),
    Token('SEMICOLON', ';', 14),
]

assert len(tokens) == len(expected)
for actual, exp in zip(tokens, expected):
    assert actual.type == exp.type
    assert actual.value == exp.value
```

**Bonus:** Add support for keywords (if, while, for) and handle them separately from identifiers.

---

### Challenge 7: Ambiguous Grammar Resolution

**Difficulty:** Medium

**Task:** Given an ambiguous grammar, fix it to remove ambiguity.

**Ambiguous Grammar (dangling else):**

```
Stat → if Expr then Stat
Stat → if Expr then Stat else Stat
Stat → other
```

**Problem:** `if e1 then if e2 then s1 else s2` has two parse trees!

**Your task:**

1. Show both parse trees for the problematic input
2. Rewrite the grammar to be unambiguous (most languages match `else` to nearest `if`)
3. Implement a parser for your unambiguous grammar

```python
def parse_statement(tokens):
    """
    Parse a statement according to your unambiguous grammar.

    tokens: list like ['if', 'e1', 'then', 'if', 'e2', 'then', 's1', 'else', 's2']

    Return: parse tree showing which 'if' the 'else' belongs to
    """
    pass
```

**Test:**

```python
# "if e1 then if e2 then s1 else s2"
# Should parse as: if e1 then (if e2 then s1 else s2)
tokens = ['if', 'e1', 'then', 'if', 'e2', 'then', 's1', 'else', 's2']
tree = parse_statement(tokens)

# Verify the 'else' is matched with the inner 'if'
assert is_else_matched_to_inner_if(tree) == True
```

---

### Bonus Challenge: Mini Language Compiler

**Difficulty:** Very Hard

**Task:** Combine everything to build a tiny compiler for a simple language!

**Language specification:**

```
program:
  x = 5;
  y = x + 3 * 2;
  if (y > 10) {
      z = y - 1;
  }
```

**Your compiler should:**

1. **Lexer:** Tokenize the source code
2. **Parser:** Build an AST from tokens
3. **Semantic analysis:** Type checking, variable tracking
4. **Code generation:** Generate bytecode or target language (Python/JavaScript)

**Requirements:**

```python
def compile_and_run(source_code):
    """
    Compile and execute the mini language.

    Return: dictionary of final variable values
    """
    tokens = lexer.tokenize(source_code)
    ast = parser.parse(tokens)
    analyzer.check(ast)  # Semantic analysis
    result = codegen.execute(ast)
    return result

# Test
source = """
x = 5;
y = x + 3 * 2;
"""
result = compile_and_run(source)
assert result['x'] == 5
assert result['y'] == 11
```

---

## 📦 Helper Code for Challenge 3: Regex Parser

For Challenge 3 (Regex to NFA), you'll need to parse the regex string into an AST before applying Thompson's construction. Here's the complete parser implementation:

### AST Node Classes

```python
class RegexNode:
    """Base class for regex AST nodes"""
    def to_nfa(self, counter):
        """Override in subclasses"""
        raise NotImplementedError

class CharNode(RegexNode):
    """Leaf: single character"""
    def __init__(self, char):
        self.char = char

class ConcatNode(RegexNode):
    """Binary: left THEN right"""
    def __init__(self, left, right):
        self.left = left
        self.right = right

class ChoiceNode(RegexNode):
    """Binary: left OR right"""
    def __init__(self, left, right):
        self.left = left
        self.right = right

class StarNode(RegexNode):
    """Unary: child repeated zero or more times"""
    def __init__(self, child):
        self.child = child
```

### Recursive Descent Parser

```python
class RegexParser:
    """
    Recursive descent parser for regex.

    Operator precedence (highest to lowest):
    1. Parentheses ()
    2. Kleene star *
    3. Concatenation (implicit)
    4. Choice |
    """
    def __init__(self, text):
        self.text = text
        self.pos = 0

    def peek(self):
        """Look at current character without consuming"""
        return self.text[self.pos] if self.pos < len(self.text) else None

    def consume(self):
        """Consume and return current character"""
        char = self.text[self.pos]
        self.pos += 1
        return char

    def parse_choice(self):
        """Parse choice operator (lowest precedence)"""
        left = self.parse_concat()

        while self.peek() == '|':
            self.consume()
            right = self.parse_concat()
            left = ChoiceNode(left, right)

        return left

    def parse_concat(self):
        """Parse concatenation (implicit, higher precedence than |)"""
        nodes = []

        while self.peek() and self.peek() not in '|)':
            nodes.append(self.parse_star())

        if not nodes:
            raise ValueError("Empty expression")

        result = nodes[0]
        for node in nodes[1:]:
            result = ConcatNode(result, node)

        return result

    def parse_star(self):
        """Parse star operator (higher precedence than concatenation)"""
        atom = self.parse_atom()

        if self.peek() == '*':
            self.consume()
            return StarNode(atom)

        return atom

    def parse_atom(self):
        """Parse atoms: characters and parenthesized expressions"""
        char = self.peek()

        if char == '(':
            self.consume()
            expr = self.parse_choice()
            if self.peek() == ')':
                self.consume()
            else:
                raise ValueError("Unmatched opening parenthesis")
            return expr

        elif char and char not in '|*()':
            self.consume()
            return CharNode(char)

        elif char is None:
            raise ValueError("Unexpected end of expression")

        else:
            raise ValueError(f"Unexpected character: {char}")


def parse_regex(regex):
    """Parse a regex string into an AST."""
    if not regex:
        raise ValueError("Empty regex")

    parser = RegexParser(regex)
    ast = parser.parse_choice()

    if parser.pos < len(regex):
        raise ValueError(f"Unexpected character at position {parser.pos}")

    return ast
```

### Usage Example

```python
# Parse "(a|b)*c"
ast = parse_regex("(a|b)*c")

# AST structure:
#     ConcatNode
#     /        \
#  StarNode    CharNode('c')
#     |
#  ChoiceNode
#   /      \
# 'a'      'b'

# Then use Thompson's construction to build NFA from AST
```

---

## 📚 Additional Resources for Coding

- **Testing your implementations:** Write comprehensive unit tests for edge cases
- **Visualization:** Use libraries like Graphviz to visualize NFAs, DFAs, and parse trees
- **Debugging:** Add logging to trace state transitions and parse decisions
- **Optimization:** After getting it working, optimize for performance

**Pro tip:** Start with Challenge 1 (NFA simulator) and work your way up. Each challenge builds on previous concepts!

---
