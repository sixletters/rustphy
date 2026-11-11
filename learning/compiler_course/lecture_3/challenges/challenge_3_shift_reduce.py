"""Challenge 3: Table-Driven Shift-Reduce Engine

The lecture showed the mechanics of shift and reduce but left the key question
open: how does the parser decide when to shift vs. reduce? The answer is the
LR automaton — next lecture. Here you'll implement the engine that uses a
pre-built SLR(1) table for the arithmetic grammar.

You do not need to understand how the table was constructed — focus on the
loop that drives the parse using it.
"""

from dataclasses import dataclass, field


# Grammar (for reference):
#   E → E + T | T
#   T → T * F | F
#   F → ( E ) | id


# Parse tables (provided — do not modify):
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

GOTO = {
    (0, 'E'): 1,  (0, 'T'): 2,  (0, 'F'): 3,
    (4, 'E'): 8,  (4, 'T'): 2,  (4, 'F'): 3,
    (6, 'T'): 9,  (6, 'F'): 3,
    (7, 'F'): 10,
}


@dataclass
class Node:
    symbol: str
    children: list = field(default_factory=list)

    def pretty_print(self, indent: int = 0) -> None:
        print(' ' * indent + self.symbol)
        for child in self.children:
            child.pretty_print(indent + 2)



def shift_reduce(tokens: list) -> bool:
    """Same engine as in the lecture.

    tokens: list of terminals ending with '$'
    e.g. ['id', '+', 'id', '*', 'id', '$']

    Print each step as: stack=[...] input=[...] action=...
    Return True on accept, False on error.
    """
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



def shift_reduce_tree(tokens: list):
    """Same engine as Challenge 3 but returns the root Node on accept."""
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


if __name__ == "__main__":
    shift_reduce(['id', '$'])
    shift_reduce(['id', '+', 'id', '$'])
    shift_reduce(['id', '+', 'id', '*', 'id', '$'])
    shift_reduce(['(', 'id', '+', 'id', ')', '*', 'id', '$'])

    tree = shift_reduce_tree(['id', '+', 'id', '*', 'id', '$'])
    if tree:
        tree.pretty_print()
