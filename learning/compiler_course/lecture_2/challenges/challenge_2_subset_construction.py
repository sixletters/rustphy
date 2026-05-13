from challenge_1_nfa_simulator import NFA

def nfa_to_dfa(nfa: NFA):
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
    def epsilon_closure(states, transitions):
        """
        Returns all states reachable from `states` via ε-transitions.
        """

        ## closure = set of states given
        closure = set(states)
        to_explore = list(states)

        ## exlore all states in set
        while to_explore:
            state = to_explore.pop()

            ## if state in set has en epsilon transition
            ## add the new_states that it can transition to into closure
            ## only add if it was not already in closure
            ## add to to_explore, closure also acts as an is_visited set
            if state in transitions and None in transitions[state]:
                for new_state in transitions[state][None]:
                    if new_state not in closure:
                        closure.add(new_state)
                        to_explore.append(new_state)
        
        return frozenset(closure)
    
    def moves(nfa_states, symbol):
        ## come back and figure out of epsilon needs to be considered here
        result = set()
        for state in nfa_states:
            if state in nfa.transitions and symbol in nfa.transitions[state]:
                result.update(nfa.transitions[state][symbol])

        return frozenset(result)
    
    dfa_start_state = frozenset(epsilon_closure({nfa.start_state}, nfa.transitions))
    to_explore_queue = [dfa_start_state]
    dfa_states = {dfa_start_state}
    dfa_transitions = {}

    while to_explore_queue:
        current_state = to_explore_queue.pop()
        for a in nfa.alphabet:
            ## computes what are the new states that can be moved into
            ## calculate epislon closure to connect states that can
            ## be reached via epsilon transitions
            next_dfa_state = epsilon_closure(moves(current_state, a), nfa.transitions)

            if current_state not in dfa_transitions:
                dfa_transitions[current_state] = {}

            dfa_transitions[current_state][a] = next_dfa_state

            if next_dfa_state and next_dfa_state not in dfa_states:
                dfa_states.add(next_dfa_state)
                to_explore_queue.append(next_dfa_state)

    dfa_accept_states = set()
    for dfa_state in dfa_states:
        ## DFA state is accepting if it contains any NFA accept state
        for nfa_state in nfa.accept_states:
            if nfa_state in dfa_state:
                dfa_accept_states.add(dfa_state)

    return DFA(
        states=dfa_states,
        alphabet=nfa.alphabet,
        transitions=dfa_transitions,
        start_state=dfa_start_state,
        accept_states=dfa_accept_states
    )


class DFA:
    def __init__(self, states, alphabet, transitions, start_state, accept_states):
        self.states = states
        self.alphabet = alphabet
        self.transitions = transitions
        self.start_state = start_state
        self.accept_states = accept_states
    
    def accepts(self, input_string):
        curr_state = self.start_state

        ## Remember that DFAs do not have epislon states! it makes traversing DFAs alot easier!
        for char in input_string:
            if curr_state in self.transitions and char in self.transitions[curr_state]:
                next_state = self.transitions[curr_state][char]
                if not next_state:
                    return False
                curr_state = next_state
            else:
                return False

        ## recall that when we did the transformation from NFA
        ## we just added all DFA states into a set of accept_states
        ## so yes you could have {a, 1} and {a, 2} as two different accept DFA states
        return curr_state in self.accept_states
            
        


# =============================================================================
# TEST NFAs FOR CHALLENGE 2
# =============================================================================

if __name__ == "__main__":
    # Test 1: Simple DFA-like NFA (no epsilons, deterministic)
    # Pattern: "ab" (just 'a' followed by 'b')
    print("Creating Test 1: Simple 'ab' pattern")
    nfa_test1 = NFA(
        states={'s0', 's1', 's2'},
        alphabet={'a', 'b'},
        transitions={
            ('s0', 'a'): {'s1'},
            ('s1', 'b'): {'s2'},
        },
        start_state='s0',
        accept_states={'s2'}
    )
    print("✓ Test 1 created\n")


    # Test 2: Non-deterministic NFA (multiple transitions on same symbol)
    # s0 ─a→ s1 ─b→ s3 (accept)
    # s0 ─a→ s2 ─b→ s3 (accept)
    print("Creating Test 2: Non-deterministic (multiple 'a' transitions)")
    nfa_test2 = NFA(
        states={'s0', 's1', 's2', 's3'},
        alphabet={'a', 'b'},
        transitions={
            ('s0', 'a'): {'s1', 's2'},  # Non-deterministic! Goes to BOTH s1 and s2
            ('s1', 'b'): {'s3'},
            ('s2', 'b'): {'s3'},
        },
        start_state='s0',
        accept_states={'s3'}
    )
    print("✓ Test 2 created\n")
    
    
    # Test 3: NFA with epsilon transitions
    # Pattern: "a?b" (optional 'a' then 'b')
    print("Creating Test 3: Epsilon transitions (a?b)")
    nfa_test3 = NFA(
        states={'s0', 's1', 's2'},
        alphabet={'a', 'b'},
        transitions={
            ('s0', None): {'s1'},   # ε from s0 to s1
            ('s0', 'a'): {'s1'},    # OR take 'a'
            ('s1', 'b'): {'s2'},    # then 'b'
        },
        start_state='s0',
        accept_states={'s2'}
    )
    print("✓ Test 3 created\n")
    
    
    # Test 4: Classic (a|b)*c
    print("Creating Test 4: Classic (a|b)*c")
    nfa_test4 = NFA(
        states={'s0', 's1'},
        alphabet={'a', 'b', 'c'},
        transitions={
            ('s0', 'a'): {'s0'},  # Self-loop on 'a'
            ('s0', 'b'): {'s0'},  # Self-loop on 'b'
            ('s0', 'c'): {'s1'},  # Transition to accept on 'c'
        },
        start_state='s0',
        accept_states={'s1'}
    )
    print("✓ Test 4 created\n")
    
    
    # Test 5: Choice with epsilons (a|b)
    # s0 ─ε→ s1 ─a→ s3 (accept)
    #   └─ε→ s2 ─b→ s3 (accept)
    print("Creating Test 5: Choice (a|b) with epsilons")
    nfa_test5 = NFA(
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
    print("✓ Test 5 created\n")
    
    
    # Test 6: Kleene star (a*)
    # s0 ─ε→ s1 (accept)
    # s0 ─a→ s2 ─ε→ s0 (loop)
    # s2 ─ε→ s1 (to accept)
    print("Creating Test 6: Kleene star (a*)")
    nfa_test6 = NFA(
        states={'s0', 's1', 's2'},
        alphabet={'a'},
        transitions={
            ('s0', None): {'s1'},        # ε to accept (for zero 'a's)
            ('s0', 'a'): {'s2'},         # Read 'a'
            ('s2', None): {'s0', 's1'},  # ε back to s0 (loop) or to accept
        },
        start_state='s0',
        accept_states={'s1'}
    )
    print("✓ Test 6 created\n")
    
    
    # Test 7: Epsilon closure chain
    # s0 ─ε→ s1 ─ε→ s2 ─ε→ s3 ─a→ s4 (accept)
    print("Creating Test 7: Epsilon closure chain")
    nfa_test7 = NFA(
        states={'s0', 's1', 's2', 's3', 's4'},
        alphabet={'a'},
        transitions={
            ('s0', None): {'s1'},   # ε chain
            ('s1', None): {'s2'},   # ε chain
            ('s2', None): {'s3'},   # ε chain
            ('s3', 'a'): {'s4'},    # Finally read 'a'
        },
        start_state='s0',
        accept_states={'s4'}
    )
    print("✓ Test 7 created\n")
    
    
    # =============================================================================
    # RUN TESTS
    # =============================================================================
    print("\n" + "="*70)
    print("TESTING NFA → DFA CONVERSION")
    print("="*70 + "\n")
    
    test_cases = [
        ("Test 1: ab", nfa_test1, [
            ("ab", True), ("", False), ("a", False), ("b", False), ("ba", False)
        ]),
        ("Test 2: Non-deterministic", nfa_test2, [
            ("ab", True), ("a", False), ("b", False), ("", False)
        ]),
        ("Test 3: a?b", nfa_test3, [
            ("b", True), ("ab", True), ("", False), ("a", False), ("bb", False)
        ]),
        ("Test 4: (a|b)*c", nfa_test4, [
            ("c", True), ("ac", True), ("bc", True), ("abc", True), ("aabbc", True),
            ("", False), ("a", False), ("ab", False), ("abcd", False)
        ]),
        ("Test 5: a|b", nfa_test5, [
            ("a", True), ("b", True), ("", False), ("ab", False)
        ]),
        ("Test 6: a*", nfa_test6, [
            ("", True), ("a", True), ("aa", True), ("aaa", True),
            ("b", False), ("ab", False)
        ]),
        ("Test 7: ε-chain", nfa_test7, [
            ("a", True), ("", False), ("aa", False)
        ]),
    ]
    
    for test_name, nfa, cases in test_cases:
        print(f"{test_name}")
        print("-" * 70)
    
        try:
            dfa = nfa_to_dfa(nfa)
    
            for input_str, expected in cases:
                result = dfa.accepts(input_str)
                status = "✓" if result == expected else "✗"
                print(f"  {status} '{input_str}' → {result} (expected {expected})")
    
            print()
        except Exception as e:
            print(f"  ❌ ERROR: {e}\n")
