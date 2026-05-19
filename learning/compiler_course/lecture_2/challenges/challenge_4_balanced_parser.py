"""
Balanced Parentheses Parser using Context-Free Grammar

This module implements a recursive descent parser for balanced parentheses
based on the following context-free grammar (CFG):

    S → ε           (empty string)
    S → (S)         (parentheses wrapped around a balanced expression)
    S → SS          (concatenation of two balanced expressions)

The parser builds an explicit parse tree representation:
    - Empty string     → None
    - (S)             → ('paren', subtree)
    - S S             → ('concat', left_tree, right_tree)

Algorithm:
    Uses recursive descent parsing to build a right-associative parse tree.
    For input "()()()":
        - Parses first "()"
        - Sees more input, so creates concat with remaining "()()"
        - Recursively builds: ('concat', ('paren', None), ('concat', ...))

    The parser tracks position through the string and returns (tree, position)
    tuples. Position indicates where the parser stopped, allowing the caller
    to continue parsing or verify the entire string was consumed.

Examples:
    "()"        → ('paren', None)
    "(())"      → ('paren', ('paren', None))
    "()()"      → ('concat', ('paren', None), ('paren', None))
    "()()()    → ('concat', ('paren', None), ('concat', ('paren', None), ('paren', None)))
    "(()())"    → ('paren', ('concat', ('paren', None), ('paren', None)))
    "()("       → None (unbalanced)
"""


def parse_balanced_parens(input_string):
    """
    Parse a string of parentheses and return a parse tree.

    Args:
        input_string: String containing only '(' and ')' characters

    Returns:
        Parse tree as nested tuples, or None if unbalanced
        - None for empty/unbalanced strings
        - ('paren', subtree) for (S)
        - ('concat', left, right) for SS

    Examples:
        >>> parse_balanced_parens("()")
        ('paren', None)
        >>> parse_balanced_parens("()()")
        ('concat', ('paren', None), ('paren', None))
        >>> parse_balanced_parens("())")
        None
    """
    def recursive_parser(input_string: str, pos: int):
        """
        Recursively parse from position and return (tree, next_position).

        Args:
            input_string: The full input string
            pos: Current position in the string

        Returns:
            Tuple of (parse_tree, position_after_parsing)
            - parse_tree: The parsed tree structure or None if invalid
            - position: Index where this parse ended (pointing AT an unconsumed char)

        Algorithm:
            1. If at '(': recursively parse inside, verify closing ')',
               check for concat with following expressions
            2. If at ')': return (None, pos) - signals end of this level to caller
            3. If past end: return (None, pos) - no more input to parse
        """
        # Base case: reached end of string
        if pos >= len(input_string):
            return (None, pos)

        # Case 1: Opening parenthesis - parse a 'paren' node
        if input_string[pos] == "(":
            # Recursively parse the content inside the parentheses
            # This advances pos+1 to skip the opening '('
            left, end_pos = recursive_parser(input_string, pos + 1)

            # Verify there's a matching closing parenthesis
            # end_pos points AT the position where we expect ')'
            if end_pos >= len(input_string) or input_string[end_pos] != ")":
                return (None, end_pos)  # Unbalanced: missing ')'

            # Check if there's another expression to concat after this paren
            # Look at position after the ')', if it's '(' then concat
            if end_pos + 1 < len(input_string) and input_string[end_pos + 1] == "(":
                # Parse the rest of the expression after consuming the ')'
                right_tree, final_pos = recursive_parser(input_string, end_pos + 1)

                # If the right side failed to parse, propagate the error
                if right_tree is None:
                    return (None, final_pos)

                # Build right-associative concat: current paren + rest
                return (('concat', ('paren', left), right_tree), final_pos)
            else:
                # No concat needed, just return the paren node
                # Consume the closing ')' by returning end_pos + 1
                return (('paren', left), end_pos + 1)

        # Case 2: Closing parenthesis - return to caller to match it
        if input_string[pos] == ")":
            # Don't consume the ')' - let the caller verify and consume it
            # This signals "I've reached the end of my level"
            return (None, pos)

    # Start parsing from position 0
    result, final_pos = recursive_parser(input_string, 0)

    # Verify entire string was consumed (no leftover characters)
    if final_pos != len(input_string):
        return None  # Error: unbalanced or trailing characters

    return result


def is_balanced(input_string):
    """
    Return True if parentheses are balanced, False otherwise.
    """
    return parse_balanced_parens(input_string) is not None

if __name__ == "__main__":
    print("\n=== Testing Balanced Parentheses Parser ===\n")

    # Test 1: Empty string
    print("Test 1: Empty string")
    result = parse_balanced_parens("")
    assert result is None, f"Expected None, got {result}"
    assert is_balanced("") == False
    print("✓ Test 1 passed!")

    # Test 2: Single pair
    print("\nTest 2: Single pair '()'")
    result = parse_balanced_parens("()")
    expected = ('paren', None)
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("()") == True
    print("✓ Test 2 passed!")

    # Test 3: Nested pairs
    print("\nTest 3: Nested pairs '(())'")
    result = parse_balanced_parens("(())")
    expected = ('paren', ('paren', None))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("(())") == True
    print("✓ Test 3 passed!")

    # Test 4: Double nested
    print("\nTest 4: Double nested '((()))'")
    result = parse_balanced_parens("((()))")
    expected = ('paren', ('paren', ('paren', None)))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("((()))") == True
    print("✓ Test 4 passed!")

    # Test 5: Simple concatenation
    print("\nTest 5: Simple concatenation '()()'")
    result = parse_balanced_parens("()()")
    expected = ('concat', ('paren', None), ('paren', None))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("()()") == True
    print("✓ Test 5 passed!")

    # Test 6: Triple concatenation (right-associative)
    print("\nTest 6: Triple concatenation '()()()'")
    result = parse_balanced_parens("()()()")
    expected = ('concat', ('paren', None), ('concat', ('paren', None), ('paren', None)))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("()()()") == True
    print("✓ Test 6 passed!")

    # Test 7: Nested with concat inside
    print("\nTest 7: Nested with concat inside '(()())'")
    result = parse_balanced_parens("(()())")
    expected = ('paren', ('concat', ('paren', None), ('paren', None)))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("(()())") == True
    print("✓ Test 7 passed!")

    # Test 8: Complex nested and concat (right-associative)
    print("\nTest 8: Complex '((()))()'")
    result = parse_balanced_parens("((()))()")
    # Parser creates right-associative tree
    expected = ('concat', ('paren', ('paren', ('paren', None))), ('paren', None))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("((()))()") == True
    print("✓ Test 8 passed!")

    # Test 9: Unbalanced - missing closing
    print("\nTest 9: Unbalanced - missing closing '()('")
    result = parse_balanced_parens("()(")
    assert result is None, f"Expected None for unbalanced, got {result}"
    assert is_balanced("()(") == False
    print("✓ Test 9 passed!")

    # Test 10: Unbalanced - missing opening
    print("\nTest 10: Unbalanced - missing opening '())'")
    result = parse_balanced_parens("())")
    assert result is None, f"Expected None for unbalanced, got {result}"
    assert is_balanced("())") == False
    print("✓ Test 10 passed!")

    # Test 11: Unbalanced - only opening
    print("\nTest 11: Unbalanced - only opening '((('")
    result = parse_balanced_parens("(((")
    assert result is None, f"Expected None for unbalanced, got {result}"
    assert is_balanced("(((") == False
    print("✓ Test 11 passed!")

    # Test 12: Unbalanced - only closing
    print("\nTest 12: Unbalanced - only closing ')))'")
    result = parse_balanced_parens(")))")
    assert result is None, f"Expected None for unbalanced, got {result}"
    assert is_balanced(")))") == False
    print("✓ Test 12 passed!")

    # Test 13: Complex valid expression (right-associative)
    print("\nTest 13: Complex valid '()(())(()())'")
    result = parse_balanced_parens("()(())(()())")
    # Right-associative: () then (()) then (()())
    expected = ('concat',
                 ('paren', None),
                 ('concat',
                   ('paren', ('paren', None)),
                   ('paren', ('concat', ('paren', None), ('paren', None)))))
    assert result == expected, f"Expected {expected}, got {result}"
    assert is_balanced("()(())(()())") == True
    print("✓ Test 13 passed!")

    print("\n🎉 All balanced parentheses parser tests passed!")
