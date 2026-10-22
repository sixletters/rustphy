"""Challenge 1: PDA Simulator — Recognising a^n b^n

Implement a PDA using an explicit stack.

Requirements:
- Return True if the string is valid a^n b^n (including n = 0)
- Return False otherwise
- Use a stack explicitly — don't solve it with count()
"""


def pda_anbn(s: str) -> bool:
    stack = []
    idx = 0

    while idx < len(s) and s[idx] == 'a':
        stack.append('a')
        idx += 1

    while idx < len(s) and s[idx] == 'b':
        if not stack:
            return False
        stack.pop()
        idx += 1

    return idx == len(s) and len(stack) == 0


if __name__ == "__main__":
    assert pda_anbn("") == True
    assert pda_anbn("ab") == True
    assert pda_anbn("aabb") == True
    assert pda_anbn("aaabbb") == True
    assert pda_anbn("aab") == False
    assert pda_anbn("ba") == False
    assert pda_anbn("abab") == False
    assert pda_anbn("aabbb") == False
    print("All tests passed!")
