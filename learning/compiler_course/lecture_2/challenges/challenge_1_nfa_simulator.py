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
        transitions_lookup = {}
        for t in transitions:
            state, char = t
            if state not in transitions_lookup:
                transitions_lookup[state] = {}
            if char not in transitions_lookup[state]:
                transitions_lookup[state][char] = transitions[t]

        self.transitions = transitions_lookup
        self.states = states
        self.start_state = start_state
        self.accept_states = accept_states
        self.alphabet = alphabet

    def epsilon_closure(self, states):
        """
        Returns all states reachable from `states` via ε-transitions.
        """
        closure = set(states)
        to_explore = list(states)
        
        while to_explore:
            state = to_explore.pop()
            
            if state in self.transitions and None in self.transitions[state]:
                for new_state in self.transitions[state][None]:
                    if new_state not in closure:  # Haven't seen it yet!
                        closure.add(new_state)
                        to_explore.append(new_state)  # Explore it!
        
        return closure


    def accepts(self, input_string):
        """
        Return True if the NFA accepts the input string, False otherwise.
        Must handle ε-transitions correctly!
        """
        curr_states = self.epsilon_closure({self.start_state})

        for char in input_string:
            next_states = set()
            for state in curr_states:
                if state in self.transitions and char in self.transitions[state]:
                    next_states_from_this = self.transitions[state][char]
                    next_states.update(self.epsilon_closure(next_states_from_this))
            curr_states = next_states
            if not curr_states:
                return False
            
        return bool(curr_states & self.accept_states)


if __name__ == "__main__":
    print("\n=== Testing ε-transitions ===\n")

    # Test 1: Simple ε-transition
    # Regex: a?b (optional 'a' then 'b')
    # s0 ─ε→ s1 ─b→ s2
    # s0 ─a→ s1
    print("Test 1: Simple ε-transition (a?b)")
    nfa1 = NFA(
        states={'s0', 's1', 's2'},
        alphabet={'a', 'b'},
        transitions={
            ('s0', None): {'s1'},   # ε from s0 to s1
            ('s0', 'a'): {'s1'},     # OR take 'a'
            ('s1', 'b'): {'s2'},     # then 'b'
        },
        start_state='s0',
        accept_states={'s2'}
    )

    assert nfa1.accepts("ab") == True   # Take 'a' path
    assert nfa1.accepts("b") == True    # Take ε path, then 'b'
    assert nfa1.accepts("a") == False   # Missing 'b'
    assert nfa1.accepts("") == False    # Empty
    print("✓ Test 1 passed!")


    # Test 2: Chained ε-transitions
    # s0 ─ε→ s1 ─ε→ s2 ─a→ s3
    print("\nTest 2: Chained ε-transitions")
    nfa2 = NFA(
        states={'s0', 's1', 's2', 's3'},
        alphabet={'a'},
        transitions={
            ('s0', None): {'s1'},   # ε chain
            ('s1', None): {'s2'},   # ε chain
            ('s2', 'a'): {'s3'},    # then 'a'
        },
        start_state='s0',
        accept_states={'s3'}
    )

    assert nfa2.accepts("a") == True    # ε-closure of s0 = {s0,s1,s2}, then 'a'
    assert nfa2.accepts("") == False
    assert nfa2.accepts("aa") == False
    print("✓ Test 2 passed!")


    # Test 3: ε-transition after reading char
    # s0 ─a→ s1 ─ε→ s2 (accept)
    print("\nTest 3: ε-transition after reading")
    nfa3 = NFA(
        states={'s0', 's1', 's2'},
        alphabet={'a'},
        transitions={
            ('s0', 'a'): {'s1'},
            ('s1', None): {'s2'},   # ε after reading 'a'
        },
        start_state='s0',
        accept_states={'s2'}
    )

    assert nfa3.accepts("a") == True    # Read 'a', land in s1, ε to s2
    assert nfa3.accepts("") == False
    assert nfa3.accepts("aa") == False
    print("✓ Test 3 passed!")


    # Test 4: Cycle with ε-transitions
    # s0 ─a→ s1 ─ε→ s2 ─ε→ s1 (cycle!), s2 is accept
    print("\nTest 4: ε-transition cycle")
    nfa4 = NFA(
        states={'s0', 's1', 's2'},
        alphabet={'a'},
        transitions={
            ('s0', 'a'): {'s1'},
            ('s1', None): {'s2'},   # ε to s2
            ('s2', None): {'s1'},   # ε back to s1 (cycle!)
        },
        start_state='s0',
        accept_states={'s2'}
    )

    assert nfa4.accepts("a") == True    # Read 'a', ε-closure includes s2
    assert nfa4.accepts("") == False
    print("✓ Test 4 passed!")


    # Test 5: Multiple ε-paths (NFA for (a|b))
    # s0 ─ε→ s1 ─a→ s3 (accept)
    #   └─ε→ s2 ─b→ s3
    print("\nTest 5: Multiple ε-paths")
    nfa5 = NFA(
        states={'s0', 's1', 's2', 's3'},
        alphabet={'a', 'b'},
        transitions={
            ('s0', None): {'s1', 's2'},  # ε to both s1 and s2
            ('s1', 'a'): {'s3'},
            ('s2', 'b'): {'s3'},
        },
        start_state='s0',
        accept_states={'s3'}
    )

    assert nfa5.accepts("a") == True
    assert nfa5.accepts("b") == True
    assert nfa5.accepts("ab") == False
    assert nfa5.accepts("") == False
    print("✓ Test 5 passed!")

    print("\n🎉 All ε-transition tests passed!")
