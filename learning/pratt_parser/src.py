from enum import IntEnum
from lexer import Lexer, EofToken, IntToken, IdentToken, PlusToken, MinusToken, AsteriskToken, SlashToken, LParenToken, RParenToken


class Precedence(IntEnum):
    LOWEST = 1
    SUM = 2
    PRODUCT = 3
    PREFIX = 4
    CALL = 5

INPUT_STRING = " 2 + 3 * 4"
INPUT_STRING_2 = " 2 + 3 * (4 + 5)"

class PrattParser:
    def __init__(self, lexer: Lexer):
        self.lexer = lexer
        self.current_token = None
        self.peek_token = None
        self.consume_token()
        self.consume_token()

    def consume_token(self):
        self.current_token = self.peek_token
        self.peek_token = self.lexer.next_token()

    def parse_expression(self, precedence: Precedence):
        ## 1. Parse the prefix expression based on the current token
        left_exp = self.parse_prefix(precedence)
        ## While the precedence of the curr token is greater than precedence passed in
        ## you should keep "taking", one example is for example if u have a + b * c
        ## when you parse infix of a + b, its going to be such that left is a
        ## operator is + and then u call parse expression on b * c with the 
        ## precedence of +, that means that b + c will be grouped together
        ## and returned as the right becoming, a + (b * c) instead of (a + b) * c
        ## what this logic essentially ensure its, everything that is of higher
        ## precedence than current is grouped together, done recursively.
        while self.get_precedence(self.current_token) > precedence and self.current_token is not EofToken:
            left_exp = self.parse_infix(left_exp)
        return left_exp

    def parse_infix(self, left) -> dict:
        if isinstance(self.current_token, PlusToken):
            self.consume_token()
            right = self.parse_expression(Precedence.SUM)
            return f"({left} + {right})"
        if isinstance(self.current_token, AsteriskToken):
            self.consume_token()
            right = self.parse_expression(Precedence.PRODUCT)
            return f"({left} * {right})"

    
    def parse_prefix(self, precedence: Precedence):
        if isinstance(self.current_token, IntToken):
            value = self.current_token.value
            self.consume_token()
            return value
        elif isinstance(self.current_token, IdentToken):
            value = self.current_token.value
            self.consume_token()
            return value
        elif isinstance(self.current_token, LParenToken):
            self.consume_token()
            exp = self.parse_expression(Precedence.LOWEST)
            if not isinstance(self.current_token, RParenToken):
                raise Exception("Expected closing parenthesis")
            self.consume_token()
            return exp
        else:
            raise Exception(f"Unexpected token: {self.current_token}")
    
    def get_precedence(self, token):
        if isinstance(token, PlusToken) or isinstance(token, MinusToken):
            return Precedence.SUM
        elif isinstance(token, AsteriskToken) or isinstance(token, SlashToken):
            return Precedence.PRODUCT
        else:
            return Precedence.LOWEST

if __name__ == "__main__":
    # Test the lexer
    lexer = Lexer(INPUT_STRING)
    parser = PrattParser(lexer)
    result = parser.parse_expression(Precedence.LOWEST)
    print(result)

    # Test the lexer
    lexer = Lexer(INPUT_STRING_2)
    parser = PrattParser(lexer)
    result = parser.parse_expression(Precedence.LOWEST)
    print(result)
