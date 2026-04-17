from dataclasses import dataclass

# Token types
@dataclass
class Token:
    """Base class for all tokens"""
    pass

@dataclass
class IntToken(Token):
    value: str

@dataclass
class IdentToken(Token):
    value: str

@dataclass
class PlusToken(Token):
    pass

@dataclass
class MinusToken(Token):
    pass

@dataclass
class AsteriskToken(Token):
    pass

@dataclass
class SlashToken(Token):
    pass

@dataclass
class LParenToken(Token):
    pass

@dataclass
class RParenToken(Token):
    pass

@dataclass
class EofToken(Token):
    pass

@dataclass
class IllegalToken(Token):
    char: str


class Lexer:
    """A lexer for tokenizing input source code.

    Mimics the Rust lexer structure with pos, read_pos, and ch fields.
    """

    def __init__(self, input_str: str):
        """Creates a new lexer instance with the given input string."""
        self.input = input_str
        self.pos = 0          # Current position in input (points to current char)
        self.read_pos = 0     # Next reading position in input (after current char)
        self.ch = ''          # Current character being examined
        self.read_char()      # Initialize by reading first character

    def read_char(self):
        """Advances the lexer to the next character in the input."""
        if self.read_pos >= len(self.input):
            self.ch = '\0'  # EOF
        else:
            self.ch = self.input[self.read_pos]
        self.pos = self.read_pos
        self.read_pos += 1

    def peek_char(self) -> str:
        """Peeks at the next character without consuming it."""
        if self.read_pos >= len(self.input):
            return '\0'
        return self.input[self.read_pos]

    def eat_whitespace(self):
        """Skips over all whitespace characters."""
        while self.ch in [' ', '\t', '\n', '\r']:
            self.read_char()

    def read_number(self) -> str:
        """Reads a sequence of digits and returns it as a string."""
        start_pos = self.pos
        while self.is_digit(self.ch):
            self.read_char()
        return self.input[start_pos:self.pos]

    def read_identifier(self) -> str:
        """Reads an identifier (variable name, function name, or keyword)."""
        start_pos = self.pos
        while self.is_letter_or_underscore(self.ch):
            self.read_char()
        return self.input[start_pos:self.pos]

    def next_token(self) -> Token:
        """Reads and returns the next token from the input."""
        self.eat_whitespace()

        # Single character tokens
        if self.ch == '+':
            self.read_char()
            return PlusToken()
        elif self.ch == '-':
            self.read_char()
            return MinusToken()
        elif self.ch == '*':
            self.read_char()
            return AsteriskToken()
        elif self.ch == '/':
            self.read_char()
            return SlashToken()
        elif self.ch == '(':
            self.read_char()
            return LParenToken()
        elif self.ch == ')':
            self.read_char()
            return RParenToken()
        elif self.ch == '\0':
            return EofToken()
        # Numbers
        elif self.is_digit(self.ch):
            num = self.read_number()
            return IntToken(num)
        # Identifiers
        elif self.is_letter(self.ch):
            ident = self.read_identifier()
            return IdentToken(ident)
        else:
            # Illegal character
            illegal_char = self.ch
            self.read_char()
            return IllegalToken(illegal_char)

    @staticmethod
    def is_letter(ch: str) -> bool:
        """Checks if a character is a letter (a-z or A-Z)."""
        return ch.isalpha()

    @staticmethod
    def is_digit(ch: str) -> bool:
        """Checks if a character is a digit (0-9)."""
        return ch.isdigit()

    @staticmethod
    def is_letter_or_underscore(ch: str) -> bool:
        """Checks if a character is valid in an identifier."""
        return ch.isalpha() or ch == '_' or ch.isdigit()
