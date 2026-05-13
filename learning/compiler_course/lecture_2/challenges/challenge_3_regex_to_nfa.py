"""
Challenge 3: Regex to NFA (Thompson's Construction)

Converts a regex to an NFA using Thompson's construction algorithm.
1. Parse regex string into AST (Abstract Syntax Tree)
2. Recursively build NFA fragments for each AST node
3. Combine fragments using Thompson's patterns (ε-transitions)
4. Convert final fragment to executable NFA

Supported operators (precedence high→low): () > * > concat > |
"""

from challenge_1_nfa_simulator import NFA


# =============================================================================
# AST NODE CLASSES
# =============================================================================

class RegexNode:
    """Base class for regex AST nodes"""
    def to_nfa(self, counter) -> NFAFragment:
        """Override in subclasses"""
        raise NotImplementedError

class CharNode(RegexNode):
    """Leaf: single character"""
    def __init__(self, char):
        self.char = char

    def to_nfa(self, counter) -> NFAFragment:
        return char_nfa(self.char, counter=counter)

class ConcatNode(RegexNode):
    """Binary: left THEN right"""
    def __init__(self, left: RegexNode, right: RegexNode):
        self.left = left
        self.right = right

    def to_nfa(self, counter) -> NFAFragment:
        left_nfa = self.left.to_nfa(counter=counter)
        right_nfa = self.right.to_nfa(counter=counter)
        return sequence_nfa(left_nfa=left_nfa,right_nfa=right_nfa, counter=counter)

class ChoiceNode(RegexNode):
    """Binary: left OR right"""
    def __init__(self, left, right):
        self.left = left
        self.right = right

    def to_nfa(self, counter) -> NFAFragment:
        left_nfa = self.left.to_nfa(counter=counter)
        right_nfa = self.right.to_nfa(counter=counter)
        return choice_nfa(left_nfa=left_nfa,right_nfa=right_nfa, counter=counter)

class StarNode(RegexNode):
    """Unary: child repeated zero or more times"""
    def __init__(self, child):
        self.child = child

    def to_nfa(self, counter) -> NFAFragment:
        child = self.child.to_nfa(counter=counter)
        return star_nfa(child=child, counter=counter)

class NFAFragment:
    """
    A fragment of an NFA with exactly one start and one accept state.
    Used for Thompson's construction.
    """
    def __init__(self, alphabet, start, accept, transitions):
        self.start = start ## single start state
        self.accept = accept ## single accept state
        self.alphabet = alphabet
        self.transitions = transitions ## transitions of {state: {symbol1: set(next_states), ... symbol2: set(next_states)}}

    def to_nfa(self):
        ## collect states
        states = set()
        for state in self.transitions:
            states.add(state)
            for destination in self.transitions[state]:
                states.update(self.transitions[state][destination])

        nfa_transitions = {}
        for state in self.transitions:
            for symbol in self.transitions[state]:
                nfa_transitions[(state, symbol)] = self.transitions[state][symbol]

        return NFA(
            start_state=self.start,
            states=states,
            alphabet=self.alphabet,
            accept_states={self.accept},
            transitions=nfa_transitions
        )

class StateCounter:
    """Generates unique state IDs to avoid collisions when merging NFA fragments"""
    def __init__(self):
        self.count = 0

    def next(self):
        """Get next unique state ID"""
        state = self.count
        self.count += 1
        return state


# =============================================================================
# THOMPSON'S CONSTRUCTION - NFA BUILDING BLOCKS
# =============================================================================

def char_nfa(char: str, counter: StateCounter) -> NFAFragment:
    """Build NFA for single character: start ─char→ accept"""
    start_state = counter.next()
    accept_state = counter.next()
    alphabet = {char}
    transitions = {}
    transitions[start_state] = {char: {accept_state}}
    return NFAFragment(
        start=start_state,
        accept=accept_state,
        alphabet=alphabet,
        transitions=transitions
    )

def sequence_nfa(left_nfa: NFAFragment, right_nfa: NFAFragment, counter: StateCounter) -> NFAFragment:
    """
    Build NFA for concatenation: left THEN right
    Connect left.accept ─ε→ right.start
    """
    transitions = {**left_nfa.transitions, **right_nfa.transitions}

    # Add ε-transition from left.accept to right.start
    if left_nfa.accept not in transitions:
        transitions[left_nfa.accept] = {}
    if None not in transitions[left_nfa.accept]:
        transitions[left_nfa.accept][None] = set()

    transitions[left_nfa.accept][None].add(right_nfa.start)

    return NFAFragment(
        alphabet=left_nfa.alphabet | right_nfa.alphabet,
        start=left_nfa.start,
        accept=right_nfa.accept,
        transitions=transitions
    )

def choice_nfa(left_nfa: NFAFragment, right_nfa: NFAFragment, counter: StateCounter):
    """
    Build NFA for choice: left OR right
            ε → left ─ε→
    new_start         new_accept
            ε → right ─ε→
    """
    start_state = counter.next()
    accept_state = counter.next()

    transitions = {**left_nfa.transitions, **right_nfa.transitions}

    # ε from new start to both branches
    transitions[start_state] = {
        None: {left_nfa.start, right_nfa.start}
    }

    if right_nfa.accept not in transitions:
        transitions[right_nfa.accept] = {}

    if left_nfa.accept not in transitions:
        transitions[left_nfa.accept] = {}

    if None not in transitions[right_nfa.accept]:
        transitions[right_nfa.accept][None] = set()

    if None not in transitions[left_nfa.accept]:
        transitions[left_nfa.accept][None] = set()

    transitions[right_nfa.accept][None].add(accept_state)
    transitions[left_nfa.accept][None].add(accept_state)

    return NFAFragment(
        alphabet=left_nfa.alphabet | right_nfa.alphabet,
        start=start_state,
        accept=accept_state,
        transitions=transitions
    )


def star_nfa(child: NFAFragment, counter: StateCounter):
    """
    Build NFA for Kleene star: child*
          ┌─────ε─────┐ (loop back)
          ↓           │
    new_start → child → new_accept
          └──────ε──────┘ (skip - zero reps)
    """
    start_state = counter.next()
    accept_state = counter.next()

    transitions = {**child.transitions}

    # ε from new start: enter OR skip
    transitions[start_state] = {
        None: {accept_state, child.start}
    }

    if child.accept not in transitions:
        transitions[child.accept] = {}

    if None not in transitions[child.accept]:
        transitions[child.accept][None] = set()

    # ε from child.accept: exit OR loop back
    transitions[child.accept][None].add(accept_state)
    transitions[child.accept][None].add(child.start)

    return NFAFragment(
        alphabet=child.alphabet,
        start=start_state,
        accept=accept_state,
        transitions=transitions
    )


# =============================================================================
# REGEX PARSER (Recursive Descent)
# =============================================================================

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
            self.consume()  # Consume '|'
            right = self.parse_concat()
            left = ChoiceNode(left, right)

        return left

    def parse_concat(self):
        """Parse concatenation (implicit, higher precedence than |)"""
        nodes = []

        # Collect consecutive atoms
        while self.peek() and self.peek() not in '|)':
            nodes.append(self.parse_star())

        if not nodes:
            raise ValueError("Empty expression")

        # Build left-associative concat tree
        result = nodes[0]
        for node in nodes[1:]:
            result = ConcatNode(result, node)

        return result

    def parse_star(self):
        """Parse star operator (higher precedence than concatenation)"""
        atom = self.parse_atom()

        if self.peek() == '*':
            self.consume()  # Consume '*'
            return StarNode(atom)

        return atom

    def parse_atom(self):
        """Parse atoms: characters and parenthesized expressions"""
        char = self.peek()

        if char == '(':
            self.consume()  # Consume '('
            expr = self.parse_choice()  # Recursively parse inside parens
            if self.peek() == ')':
                self.consume()  # Consume ')'
            else:
                raise ValueError("Unmatched opening parenthesis")
            return expr

        elif char and char not in '|*()':
            # Regular character
            self.consume()
            return CharNode(char)

        elif char is None:
            raise ValueError("Unexpected end of expression")

        else:
            raise ValueError(f"Unexpected character: {char}")


def parse_regex(regex: str) -> RegexNode:
    """Parse a regex string into an AST."""
    if not regex:
        raise ValueError("Empty regex")

    parser = RegexParser(regex)
    ast = parser.parse_choice()

    # Check if we consumed the entire regex
    if parser.pos < len(regex):
        raise ValueError(f"Unexpected character at position {parser.pos}: {regex[parser.pos]}")

    return ast
    

def regex_to_nfa(regex: str) -> NFA:
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

    ## The idea is to parse the regex into a simple abstract syntax tree
    ## and then recursively build NFAs for each operator
    
    ast = parse_regex(regex=regex)

    counter = StateCounter()

    ## to nfa fragment then to NFA
    return ast.to_nfa(counter=counter).to_nfa()


# =============================================================================
# TEST CASES
# =============================================================================

if __name__ == "__main__":
    print("="*70)
    print("TESTING THOMPSON'S CONSTRUCTION: REGEX → NFA")
    print("="*70)

    # Test 1: Simple character
    print("\nTest 1: Single character 'a'")
    nfa = regex_to_nfa("a")
    assert nfa.accepts("a") == True
    assert nfa.accepts("") == False
    assert nfa.accepts("aa") == False
    print("✓ Test 1 passed!")

    # Test 2: Concatenation
    print("\nTest 2: Concatenation 'ab'")
    nfa = regex_to_nfa("ab")
    assert nfa.accepts("ab") == True
    assert nfa.accepts("a") == False
    assert nfa.accepts("b") == False
    assert nfa.accepts("") == False
    print("✓ Test 2 passed!")

    # Test 3: Choice
    print("\nTest 3: Choice 'a|b'")
    nfa = regex_to_nfa("a|b")
    assert nfa.accepts("a") == True
    assert nfa.accepts("b") == True
    assert nfa.accepts("ab") == False
    assert nfa.accepts("") == False
    print("✓ Test 3 passed!")

    # Test 4: Kleene star
    print("\nTest 4: Kleene star 'a*'")
    nfa = regex_to_nfa("a*")
    assert nfa.accepts("") == True
    assert nfa.accepts("a") == True
    assert nfa.accepts("aa") == True
    assert nfa.accepts("aaa") == True
    assert nfa.accepts("b") == False
    print("✓ Test 4 passed!")

    # Test 5: Star with concatenation
    print("\nTest 5: Star with concat 'ab*'")
    nfa = regex_to_nfa("ab*")
    assert nfa.accepts("a") == True
    assert nfa.accepts("ab") == True
    assert nfa.accepts("abb") == True
    assert nfa.accepts("abbb") == True
    assert nfa.accepts("") == False
    assert nfa.accepts("b") == False
    print("✓ Test 5 passed!")

    # Test 6: Parentheses with star
    print("\nTest 6: Parentheses '(ab)*'")
    nfa = regex_to_nfa("(ab)*")
    assert nfa.accepts("") == True
    assert nfa.accepts("ab") == True
    assert nfa.accepts("abab") == True
    assert nfa.accepts("ababab") == True
    assert nfa.accepts("a") == False
    assert nfa.accepts("aba") == False
    print("✓ Test 6 passed!")

    # Test 7: Classic (a|b)*c
    print("\nTest 7: Classic '(a|b)*c'")
    nfa = regex_to_nfa("(a|b)*c")
    assert nfa.accepts("c") == True
    assert nfa.accepts("ac") == True
    assert nfa.accepts("bc") == True
    assert nfa.accepts("abc") == True
    assert nfa.accepts("aabbc") == True
    assert nfa.accepts("") == False
    assert nfa.accepts("ab") == False
    print("✓ Test 7 passed!")

    # Test 8: Complex expression
    print("\nTest 8: Complex 'a(b|c)*d'")
    nfa = regex_to_nfa("a(b|c)*d")
    assert nfa.accepts("ad") == True
    assert nfa.accepts("abd") == True
    assert nfa.accepts("acd") == True
    assert nfa.accepts("abcd") == True
    assert nfa.accepts("abbbcccd") == True
    assert nfa.accepts("a") == False
    assert nfa.accepts("d") == False
    assert nfa.accepts("ab") == False
    print("✓ Test 8 passed!")

    # Test 9: Multiple choices
    print("\nTest 9: Multiple choices 'a|b|c'")
    nfa = regex_to_nfa("a|b|c")
    assert nfa.accepts("a") == True
    assert nfa.accepts("b") == True
    assert nfa.accepts("c") == True
    assert nfa.accepts("ab") == False
    assert nfa.accepts("") == False
    print("✓ Test 9 passed!")

    # Test 10: Nested parentheses
    print("\nTest 10: Nested '((a|b)*c)*'")
    nfa = regex_to_nfa("((a|b)*c)*")
    assert nfa.accepts("") == True
    assert nfa.accepts("c") == True
    assert nfa.accepts("ac") == True
    assert nfa.accepts("abc") == True
    assert nfa.accepts("cc") == True
    assert nfa.accepts("acbc") == True
    assert nfa.accepts("ab") == False
    print("✓ Test 10 passed!")

    print("\n" + "="*70)
    print("🎉 ALL TESTS PASSED!")
    print("="*70)
