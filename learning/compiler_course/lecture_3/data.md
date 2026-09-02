# Lecture 3: Shift-Reduce Parsing

---

## Context-Free Grammar (CFG)

A **CFG** is a 4-tuple `G = (V, Σ, R, S)`:

- `V` — finite set of **non-terminals** (abstract categories)
- `Σ` — finite set of **terminals** (actual tokens), disjoint from `V`
- `R` — finite set of **production rules** of the form `A → β`
- `S ∈ V` — the **start symbol**

### Terminals vs Non-Terminals

| | Definition | Example |
|---|---|---|
| **Non-terminal** | Has a production rule. Can be expanded into something else. Never appears in source code. | `E`, `T`, `F` |
| **Terminal** | Has NO production rule. Cannot be expanded. Is a final, atomic token. | `+`, `*`, `id`, `(` |

**The mechanical test**: does it appear on the left side of any rule? Yes → non-terminal. No → terminal.

### Example Grammar (Arithmetic)

```
E → E + T  |  T
T → T * F  |  F
F → ( E )  |  id
```

A well-formed grammar guarantees that every derivation eventually reaches **only terminals**. That's the "terminal" in the name — it's where derivation terminates.

### What "Context-Free" Means

Every production rule has a **single non-terminal** on the left side: `A → β`.

`A` can always be replaced by `β` regardless of what surrounds it. The surrounding context doesn't matter — hence "context-free."

---

## Key Concepts

### Sentential Form

Derivation starts from `S` and keeps expanding non-terminals until you have all terminals. Any **intermediate snapshot** during that process is called a **sentential form**.

```
E
E + T         ← sentential form
E + T * F     ← sentential form
id + T * F    ← sentential form
id + id * id  ← sentential form (all terminals = final string)
```

### Rightmost Derivation

At each derivation step you pick a non-terminal to expand. A **rightmost derivation** always expands the **rightmost** non-terminal first.

A **right-sentential form** is any snapshot from a rightmost derivation.

### Handle

A shift-reduce parser works **backwards** — it starts from `id + id * id` and wants to get back to `E`. So it reverses the derivation, asking at each step:

> "What was the last thing expanded in the rightmost derivation? I'll reduce that first."

That last-expanded piece is the **handle**.

**Key theorem**: In an unambiguous grammar, every right-sentential form has exactly one handle. This is what makes shift-reduce parsing tractable — we just need to find it.

---

## Pushdown Automata (PDA)

### Why DFAs Aren't Enough

A regular finite automaton has:
1. A set of states
2. A transition function: `(state, input_char) → next_state`
3. Accept/reject based on final state

But a DFA has **no memory**. It can't recognize `aⁿbⁿ` (n a's followed by n b's) because it can't count. There's a formal proof of this — the DFA literally cannot remember how many a's it saw.

### The Fix: Add a Stack

A **Pushdown Automaton** is a DFA with one addition — an **infinite stack**.

Formally, a PDA is a 7-tuple `M = (Q, Σ, Γ, δ, q₀, Z₀, F)`:

| Symbol | Meaning |
|---|---|
| `Q` | Finite set of states |
| `Σ` | Input alphabet (terminals / tokens) |
| `Γ` | Stack alphabet (can differ from Σ) |
| `δ` | Transition function: `Q × (Σ ∪ {ε}) × Γ → P(Q × Γ*)` |
| `q₀` | Start state |
| `Z₀` | Initial stack symbol (bottom sentinel) |
| `F` | Accepting states |

The transition `δ(q, a, X) = {(p, γ)}` means:

> "In state `q`, reading input `a`, with `X` on top of the stack: move to state `p` and replace `X` with string `γ`."

- If `γ = ε` → you **popped** X
- If `γ = XY` → you **pushed** Y on top of X

### Concrete Example: Recognizing `aⁿbⁿ`

Grammar: `S → a S b | ε`

Strategy:
1. For every `a` read, **push** `a` onto the stack
2. For every `b` read, **pop** an `a` off the stack
3. If input ends and stack is empty → **accept**

Trace on `aaabbb`:

```
State   Input Remaining   Stack (top → bottom)
q₀      aaabbb            Z₀
q₀      aabbb             a Z₀           (push a)
q₀      abbb              a a Z₀         (push a)
q₀      bbb               a a a Z₀       (push a)
q₁      bb                a a Z₀         (pop a on seeing b)
q₁      b                 a Z₀           (pop a)
q₁      ε                 Z₀             (pop a)
ACCEPT
```

On `aab`: stack still has `a` when input ends → **reject**.

**Key insight**: the stack gives the PDA unbounded memory. It can count, match brackets, and track nesting — anything a CFG can generate, a PDA can recognize.

### The Fundamental Theorem

> A language is context-free **if and only if** some PDA accepts it.

This means: by writing a CFG for Python syntax, you're guaranteeing that a stack machine capable of parsing it exists.

---

## Shift-Reduce Parser

A shift-reduce parser is a **deterministic PDA** where the stack and transitions are driven by your grammar's structure.

### The Two Operations

**SHIFT** — read one token from input, push it onto the stack.

```
($ E +  ,  T * id $)
    -- SHIFT -->
($ E + T *  ,  id $)
```

**REDUCE** — when the top of the stack matches the RHS of some production `A → β`, pop `|β|` symbols and push `A`.

```
($ E + T * F  ,  $ )
    -- REDUCE T → T * F -->
($ E + T  ,  $)
```

No input is consumed during a reduce — you're just reorganizing the stack.

**ACCEPT** — when the stack is `$ S` and input is `$`.

### Why Does This Work?

Every time you reduce, you're saying:

> "The symbols on top of my stack form a complete grammatical phrase. I'm collapsing them into their parent non-terminal."

Keep shifting tokens in and reducing completed phrases until the entire input collapses down to the start symbol. That's a successful parse.

The hard part: how does the parser know **when to shift vs. when to reduce**? The answer is LR automaton and parsing tables — that's the machinery that makes the PDA deterministic. We'll cover that next lecture.

---

## Full Worked Example: Parsing `id + id * id`

Grammar:
```
E → E + T  |  T
T → T * F  |  F
F → id
```

We'll trace the parser step by step. The format is `(stack | remaining input)`.

---

**Step 1** `($ | id + id * id $)`

Stack is empty. Nothing to reduce. **Shift** `id`.

---

**Step 2** `($ id | + id * id $)`

Top of stack is `id`. This matches the RHS of `F → id`. **Reduce**.
Pop `id`, push `F`.

---

**Step 3** `($ F | + id * id $)`

Top is `F`. Matches `T → F`. **Reduce**.
Pop `F`, push `T`.

---

**Step 4** `($ T | + id * id $)`

Top is `T`. Matches `E → T`. **Reduce**.
Pop `T`, push `E`.

> But wait — why didn't we reduce `T` to `E` back in step 3? Because we hadn't looked at the next token yet. The `+` tells us we're building a larger expression, so we need `E` on the stack first.

---

**Step 5** `($ E | + id * id $)`

Can't reduce `E` alone into anything useful. **Shift** `+`.

---

**Step 6** `($ E + | id * id $)`

Can't reduce `E +` — it's an incomplete handle. **Shift** `id`.

---

**Step 7** `($ E + id | * id $)`

Top is `id`. Matches `F → id`. **Reduce**.
Pop `id`, push `F`.

---

**Step 8** `($ E + F | * id $)`

Top is `F`. Matches `T → F`. **Reduce**.
Pop `F`, push `T`.

---

**Step 9** `($ E + T | * id $)`

Next token is `*`. The rule `T → T * F` means we need a `T` on the left of `*` — which we now have. **Shift** `*`.

---

**Step 10** `($ E + T * | id $)`

Can't reduce an incomplete sequence. **Shift** `id`.

---

**Step 11** `($ E + T * id | $)`

Top is `id`. Matches `F → id`. **Reduce**.
Pop `id`, push `F`.

---

**Step 12** `($ E + T * F | $)`

Top three symbols are `T * F`. Matches `T → T * F`. **Reduce**.
Pop `T * F`, push `T`.

---

**Step 13** `($ E + T | $)`

Top three symbols are `E + T`. Matches `E → E + T`. **Reduce**.
Pop `E + T`, push `E`.

---

**Step 14** `($ E | $)`

Stack is `$ E`, input is `$`. **ACCEPT.**

---

### What the Parse Tree Looks Like

```
        E
      / | \
     E  +  T
     |    / | \
     T   T  *  F
     |   |     |
     F   F     id
     |   |
     id  id
```

This correctly encodes that `*` binds tighter than `+`. The parser arrived at this structure purely by finding handles in the right order.

---

## Quiz & Practice Problems

---

### Section 1: Conceptual Checks

**Q1.** Given this grammar:

```
S → S ; S
S → id := E
E → E + id
E → id
```

List every terminal and every non-terminal. How do you know which is which?

<details markdown="1">
<summary>Answer</summary>

**Non-terminals:** `S`, `E` — they appear on the left-hand side of at least one production rule.

**Terminals:** `; `, `:=`, `+`, `id` — they never appear on the left-hand side of any rule. They are the final, atomic tokens that can't be expanded further.

The mechanical test: scan every left-hand side. Whatever appears there is a non-terminal. Everything else that shows up in the grammar is a terminal.

</details>

---

**Q2.** A classmate says: "non-terminals are just variables — placeholders for patterns we haven't seen yet." Is this a good mental model? What does it get right and what does it miss?

<details markdown="1">
<summary>Answer</summary>

It gets the right intuition — non-terminals are indeed placeholders that stand in for something to be expanded. The "variable" analogy captures that they're abstract and not yet concrete.

What it misses: non-terminals aren't about things "we haven't seen yet." They exist purely to describe grammatical structure, not knowledge or time. A better framing is that a non-terminal is a named grammatical category — it represents any string that can be derived from it. The key property is that it appears on the left-hand side of at least one rule, which is what makes it expandable.

</details>

---

**Q3.** True or False — explain your reasoning for each:

a) A terminal can appear on the right-hand side of a production rule.

b) A non-terminal must appear on the right-hand side of at least one production rule.

c) The start symbol `S` can never appear on the right-hand side of a production rule.

d) A string with only terminals is always a valid sentence in the language.

<details markdown="1">
<summary>Answer</summary>

a) **True.** Terminals appear on the right-hand side all the time. Example: in `E → E + T`, the `+` is a terminal on the RHS.

b) **False.** A non-terminal doesn't have to appear on the RHS of any rule — it just has to appear on the LHS of at least one rule. (Though a non-terminal that never appears on any RHS would be unreachable from `S` and useless.)

c) **False.** `S` can appear on the RHS. Example: `S → S ; S` is a perfectly valid production. This is how you get recursive grammars.

d) **False.** A string of all terminals is only valid if it can be derived from `S`. `* + id` is all terminals but cannot be derived from our grammar.

</details>

---

**Q4.** What is the difference between a sentential form and a right-sentential form? Can a sentential form be a right-sentential form? Can every right-sentential form be a sentential form?

<details markdown="1">
<summary>Answer</summary>

A **sentential form** is any intermediate snapshot during a derivation starting from `S` — you can expand any non-terminal at each step.

A **right-sentential form** is a snapshot from a derivation where you always expand the **rightmost** non-terminal first.

Can a sentential form be a right-sentential form? **Yes** — any right-sentential form is also a sentential form, since a rightmost derivation is still a valid derivation.

Can every right-sentential form be a sentential form? **Yes**, for the same reason — the set of right-sentential forms is a subset of sentential forms.

The distinction matters because shift-reduce parsers work backwards through rightmost derivations specifically. Not every sentential form is a right-sentential form — only those reachable by always expanding rightmost.

</details>

---

**Q5.** In your own words: what is a handle and why does finding it matter to the parser?

<details markdown="1">
<summary>Answer</summary>

A handle is the substring in the current right-sentential form that was expanded last in the rightmost derivation — so it's the piece the parser should reduce next when working bottom-up.

It matters because the shift-reduce parser's entire job is to reverse a rightmost derivation. At each step it needs to answer: "what should I collapse right now?" The handle is the answer. In an unambiguous grammar every right-sentential form has exactly one handle, which is what makes the parser's decision tractable — there's always a unique correct reduction to make.

</details>

---

### Section 2: Rightmost Derivations by Hand

Use this grammar for problems 6–8:

```
E → E + T  |  T
T → T * F  |  F
F → ( E )  |  id
```

**Q6.** Write out the full rightmost derivation of `id * id + id`. Show every intermediate step. Label each step with which production you used.

---

**Q7.** Write out the full rightmost derivation of `( id + id ) * id`. At each step, circle the non-terminal you are expanding.

---

**Q8.** In your derivation from Q6, identify the handle at each step. Remember: the handle is the substring that was expanded last in the rightmost derivation — so reading your derivation bottom-up, what substring gets reduced at each step?

---

### Section 3: Trace the Parser by Hand

For each problem below, fill in the table. Format: `(stack | remaining input)` → action.

**Q9.** Parse `id * id` using the grammar above. Fill in this table:

```
Step   Stack            Remaining Input    Action
1      $                id * id $          ?
2      ?                ?                  ?
3      ?                ?                  ?
...
```

Hint: it should take about 7 steps to reach ACCEPT.

---

**Q10.** Parse `( id + id )` using the same grammar. This one has parentheses — trace carefully. How does the parser know when to reduce `( E )` to `F`?

---

**Q11 (harder).** Parse `id + id + id`. This input is left-associative — meaning the correct parse tree groups it as `(id + id) + id`, not `id + (id + id)`.

Trace the parser and verify your final parse tree reflects left-associativity. Why does left-associativity fall out naturally from this grammar?

---

### Section 4: Draw the Parse Tree

**Q12.** Draw the parse tree for `id * id + id` using the grammar above. Do it on paper — label every internal node with its non-terminal and every leaf with its terminal.

---

**Q13.** Draw the parse tree for `( id + id ) * id`. Compare it to your tree from Q12. What is structurally different and why?

---

**Q14.** Two different parse trees for the same string means the grammar is **ambiguous**. Consider this grammar:

```
E → E + E
E → E * E
E → id
```

Draw **two different** parse trees for `id + id * id`. Why is this grammar dangerous for a shift-reduce parser?

---

### Section 5: PDA Tracing

**Q15.** Trace the PDA for `aⁿbⁿ` on the input `aabb`. Show every state, input remaining, and stack contents at each step. Does it accept?

---

**Q16.** What happens if you run the same PDA on `abba`? Trace it and show exactly where and why it rejects.

---

**Q17 (think deeply).** The fundamental theorem says every CFL has a PDA. But the PDA we described for `aⁿbⁿ` is non-deterministic — it guesses. How is a shift-reduce parser different from this? What property does a shift-reduce parser have that the naive PDA doesn't?

---

### Answers (don't peek until you've tried!)

<details>
<summary>Q3 answers</summary>

a) **True.** Terminals appear on the right-hand side all the time. Example: in `E → E + T`, the `+` is a terminal on the RHS.

b) **False.** A non-terminal doesn't have to appear on the RHS of any rule — it just has to appear on the LHS of at least one rule. (Though a non-terminal that never appears on any RHS would be unreachable from `S` and useless.)

c) **False.** `S` can appear on the RHS. Example: `S → S ; S` is a perfectly valid production. This is how you get recursive grammars.

d) **False.** A string of all terminals is only valid if it can be derived from `S`. `* + id` is all terminals but cannot be derived from our grammar.

</details>

<details>
<summary>Q6 rightmost derivation of id * id + id</summary>

```
E
E + T                    (E → E + T, expand rightmost E)
E + F                    (T → F, expand rightmost T)
E + id                   (F → id, expand rightmost F)
T + id                   (E → T, expand rightmost E)
T * F + id               (T → T * F, expand rightmost T)
T * id + id              (F → id, expand rightmost F)
F * id + id              (T → F, expand rightmost T)
id * id + id             (F → id, expand rightmost F)
```

</details>

<details>
<summary>Q14 hint — ambiguous grammar</summary>

For `id + id * id` you can get:
- Tree 1: `(id + id) * id` — reduce the `+` first
- Tree 2: `id + (id * id)` — reduce the `*` first

Both are valid derivations in `E → E + E | E * E | id`. This is exactly a shift-reduce conflict — when the parser has `E + E` on the stack and sees `*`, should it reduce `E + E` or shift `*`? The grammar gives no answer.

</details>

---

## Coding Challenges

---

### Challenge 1: PDA Simulator — Recognising aⁿbⁿ

Implement the PDA from the lecture that accepts strings of the form aⁿbⁿ.

**Requirements:**
- Return `True` if the string is valid aⁿbⁿ (including n=0)
- Return `False` otherwise
- Use a stack explicitly — don't solve it with `count()`

**Starter code:**

```python
def pda_anbn(s: str) -> bool:
    stack = []
    # TODO: push for each 'a', pop for each 'b'
    # Accept if all input consumed and stack is empty
    pass

assert pda_anbn("") == True
assert pda_anbn("ab") == True
assert pda_anbn("aabb") == True
assert pda_anbn("aaabbb") == True
assert pda_anbn("aab") == False
assert pda_anbn("ba") == False
assert pda_anbn("abab") == False
assert pda_anbn("aabbb") == False
print("All tests passed!")
```

<details markdown="1">
<summary>Solution</summary>

```python
def pda_anbn(s: str) -> bool:
    stack = []
    i = 0
    while i < len(s) and s[i] == 'a':
        stack.append('a')
        i += 1
    while i < len(s) and s[i] == 'b':
        if not stack:
            return False
        stack.pop()
        i += 1
    return i == len(s) and len(stack) == 0
```

</details>

---

### Challenge 2: Extend the PDA — Balanced Brackets

Generalise the PDA to recognise strings with balanced nested brackets: `()`, `[]`, `{}`.

**Requirements:**
- Accept any string where all brackets are correctly matched and nested
- Reject mismatched types: `([)]` is invalid
- Reject unclosed brackets: `(((` is invalid

**Starter code:**

```python
def balanced_brackets(s: str) -> bool:
    stack = []
    # TODO
    pass

assert balanced_brackets("") == True
assert balanced_brackets("()") == True
assert balanced_brackets("[]") == True
assert balanced_brackets("([{}])") == True
assert balanced_brackets("(())") == True
assert balanced_brackets("([)]") == False
assert balanced_brackets("(((") == False
assert balanced_brackets(")") == False
print("All tests passed!")
```

<details markdown="1">
<summary>Solution</summary>

```python
def balanced_brackets(s: str) -> bool:
    stack = []
    match = {')': '(', ']': '[', '}': '{'}
    for ch in s:
        if ch in '([{':
            stack.append(ch)
        elif ch in ')]}':
            if not stack or stack[-1] != match[ch]:
                return False
            stack.pop()
    return len(stack) == 0
```

</details>

---

### Challenge 3: Table-Driven Shift-Reduce Engine

The lecture showed the *mechanics* of shift and reduce but left the key question open: how does the parser decide *when* to shift vs. reduce? The answer is the LR automaton — next lecture. Here you'll implement the engine that uses a pre-built SLR(1) table for the arithmetic grammar, so you can see exactly how the decision is made in practice.

You don't need to understand how the table was constructed — focus on implementing the loop that *drives* the parse using it.

**Grammar (for reference):**
```
E → E + T | T
T → T * F | F
F → ( E ) | id
```

**The parse tables (provided — do not modify):**

```python
# ACTION[(state, token)] → ('shift', next_state) | ('reduce', lhs, rhs) | ('accept',)
ACTION = {
    (0,  'id'): ('shift', 5),     (0,  '('): ('shift', 4),
    (1,  '+'):  ('shift', 6),     (1,  '$'):  ('accept',),
    (2,  '+'):  ('reduce', 'E', ['T']),
    (2,  '*'):  ('shift', 7),
    (2,  ')'):  ('reduce', 'E', ['T']),
    (2,  '$'):  ('reduce', 'E', ['T']),
    (3,  '+'):  ('reduce', 'T', ['F']),
    (3,  '*'):  ('reduce', 'T', ['F']),
    (3,  ')'):  ('reduce', 'T', ['F']),
    (3,  '$'):  ('reduce', 'T', ['F']),
    (4,  'id'): ('shift', 5),     (4,  '('): ('shift', 4),
    (5,  '+'):  ('reduce', 'F', ['id']),
    (5,  '*'):  ('reduce', 'F', ['id']),
    (5,  ')'):  ('reduce', 'F', ['id']),
    (5,  '$'):  ('reduce', 'F', ['id']),
    (6,  'id'): ('shift', 5),     (6,  '('): ('shift', 4),
    (7,  'id'): ('shift', 5),     (7,  '('): ('shift', 4),
    (8,  '+'):  ('shift', 6),     (8,  ')'):  ('shift', 11),
    (9,  '+'):  ('reduce', 'E', ['E', '+', 'T']),
    (9,  '*'):  ('shift', 7),
    (9,  ')'):  ('reduce', 'E', ['E', '+', 'T']),
    (9,  '$'):  ('reduce', 'E', ['E', '+', 'T']),
    (10, '+'):  ('reduce', 'T', ['T', '*', 'F']),
    (10, '*'):  ('reduce', 'T', ['T', '*', 'F']),
    (10, ')'):  ('reduce', 'T', ['T', '*', 'F']),
    (10, '$'):  ('reduce', 'T', ['T', '*', 'F']),
    (11, '+'):  ('reduce', 'F', ['(', 'E', ')']),
    (11, '*'):  ('reduce', 'F', ['(', 'E', ')']),
    (11, ')'):  ('reduce', 'F', ['(', 'E', ')']),
    (11, '$'):  ('reduce', 'F', ['(', 'E', ')']),
}

# GOTO[(state, nonterminal)] → next_state
GOTO = {
    (0, 'E'): 1,  (0, 'T'): 2,  (0, 'F'): 3,
    (4, 'E'): 8,  (4, 'T'): 2,  (4, 'F'): 3,
    (6, 'T'): 9,  (6, 'F'): 3,
    (7, 'F'): 10,
}
```

**The algorithm:**

1. `state_stack = [0]`, `pos = 0`
2. `state = state_stack[-1]`, `lookahead = tokens[pos]`
3. Look up `ACTION[(state, lookahead)]`:
   - **`('shift', s)`** — push `lookahead` onto symbol stack, push `s` onto state stack, advance `pos`
   - **`('reduce', lhs, rhs)`** — pop `len(rhs)` from both stacks; let `s = state_stack[-1]`; push `lhs` onto symbol stack; push `GOTO[(s, lhs)]` onto state stack
   - **`('accept',)`** — done!
   - **missing entry** — parse error
4. Repeat from step 2

**Starter code:**

```python
def shift_reduce(tokens: list) -> bool:
    """
    tokens: list of terminals ending with '$'
    e.g. ['id', '+', 'id', '*', 'id', '$']

    Print each step as: stack=[...] input=[...] action=...
    Return True on accept, False on error.
    """
    state_stack = [0]
    symbol_stack = ['$']
    pos = 0
    # TODO: implement the loop above
    pass

shift_reduce(['id', '$'])
shift_reduce(['id', '+', 'id', '$'])
shift_reduce(['id', '+', 'id', '*', 'id', '$'])
shift_reduce(['(', 'id', '+', 'id', ')', '*', 'id', '$'])
```

<details markdown="1">
<summary>Solution</summary>

```python
def shift_reduce(tokens: list) -> bool:
    state_stack = [0]
    symbol_stack = ['$']
    pos = 0

    while True:
        state = state_stack[-1]
        lookahead = tokens[pos]
        action = ACTION.get((state, lookahead))

        print(f"  stack={symbol_stack}  input={tokens[pos:]}  action={action}")

        if action is None:
            print(f"Parse error: no action for state {state}, token '{lookahead}'")
            return False

        if action[0] == 'shift':
            _, next_state = action
            symbol_stack.append(lookahead)
            state_stack.append(next_state)
            pos += 1

        elif action[0] == 'reduce':
            _, lhs, rhs = action
            for _ in rhs:
                symbol_stack.pop()
                state_stack.pop()
            symbol_stack.append(lhs)
            state_stack.append(GOTO[(state_stack[-1], lhs)])

        elif action[0] == 'accept':
            print("ACCEPT")
            return True
```

</details>

---

### Bonus Challenge: Parse Tree Builder

Extend your `shift_reduce` engine from Challenge 3 to build and return a parse tree.

**Hint:** Maintain a third stack — a tree node stack that mirrors the symbol stack. On shift, push a leaf node. On reduce, pop `len(rhs)` nodes, create an internal node with those as children (in order), and push it.

**Starter code:**

```python
from dataclasses import dataclass, field

@dataclass
class Node:
    symbol: str
    children: list = field(default_factory=list)

    def pretty_print(self, indent: int = 0) -> None:
        print(' ' * indent + self.symbol)
        for child in self.children:
            child.pretty_print(indent + 2)

def shift_reduce_tree(tokens: list):
    """Same engine as Challenge 3 but returns the root Node on accept."""
    state_stack = [0]
    symbol_stack = ['$']
    tree_stack = [Node('$')]
    pos = 0
    # TODO: extend your shift_reduce solution with tree_stack
    pass

tree = shift_reduce_tree(['id', '+', 'id', '*', 'id', '$'])
if tree:
    tree.pretty_print()
```

Expected output for `id + id * id`:

```
E
  E
    T
      F
        id
  +
  T
    T
      F
        id
    *
    F
      id
```

<details markdown="1">
<summary>Solution</summary>

```python
def shift_reduce_tree(tokens: list):
    state_stack = [0]
    symbol_stack = ['$']
    tree_stack = [Node('$')]
    pos = 0

    while True:
        state = state_stack[-1]
        lookahead = tokens[pos]
        action = ACTION.get((state, lookahead))

        if action is None:
            return None

        if action[0] == 'shift':
            _, next_state = action
            symbol_stack.append(lookahead)
            state_stack.append(next_state)
            tree_stack.append(Node(lookahead))
            pos += 1

        elif action[0] == 'reduce':
            _, lhs, rhs = action
            children = []
            for _ in rhs:
                symbol_stack.pop()
                state_stack.pop()
                children.append(tree_stack.pop())
            children.reverse()
            symbol_stack.append(lhs)
            state_stack.append(GOTO[(state_stack[-1], lhs)])
            tree_stack.append(Node(lhs, children))

        elif action[0] == 'accept':
            return tree_stack[-1]
```

</details>
