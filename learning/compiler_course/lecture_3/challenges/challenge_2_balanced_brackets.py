"""Challenge 2: Extend the PDA — Balanced Brackets

Generalise the PDA to recognise strings with balanced nested brackets:
(), [], {}

Requirements:
- Accept any string where all brackets are correctly matched and nested
- Reject mismatched types: ([)] is invalid
- Reject unclosed brackets: ((( is invalid
"""


def balanced_brackets(s: str) -> bool:
    stack = []
    matching = {
        ")": "(",
        "]": "[",
        "}": "{",
    }
    opening = set(matching.values())

    for c in s:
        if c in opening:
            stack.append(c)
        elif c in matching:
            if not stack or stack.pop() != matching[c]:
                return False

    return not stack


if __name__ == "__main__":
    assert balanced_brackets("") == True
    assert balanced_brackets("()") == True
    assert balanced_brackets("[]") == True
    assert balanced_brackets("([{}])") == True
    assert balanced_brackets("(())") == True
    assert balanced_brackets("([)]") == False
    assert balanced_brackets("(((") == False
    assert balanced_brackets(")") == False
    print("All tests passed!")
