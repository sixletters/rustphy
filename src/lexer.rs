use crate::token::{Token, lookup_identifier};

/// A lexer for tokenizing input source code.
///
/// The lexer reads through the input string character by character,
/// maintaining both the current position and the next read position
/// to support lookahead operations.
pub struct Lexer {
    /// The input source code as a string
    pub input: String,
    /// Current position in the input (points to current char)
    pub pos: usize,
    /// Next reading position in the input (after current char)
    pub read_pos: usize,
    /// Current character being examined (0 if EOF)
    pub ch: u8,
}

impl Lexer {
    /// Creates a new lexer instance with the given input string.
    ///
    /// The lexer is initialized with the first character of the input
    /// already read and ready for processing.
    ///
    /// # Arguments
    /// * `input` - The source code string to tokenize
    ///
    /// # Examples
    /// ```
    /// use rust_impl::lexer::Lexer;
    /// let lexer = Lexer::new(String::from("let x = 5;"));
    /// ```
    pub fn new(input: String) -> Self {
        let mut res = Lexer {
            input,
            pos: 0,
            read_pos: 0,
            ch: 0,
        };
        res.read_char();
        res
    }

    /// Reads and returns the next token from the input.
    ///
    /// This is the main lexing method that processes the input character by character,
    /// identifying and returning the appropriate token type. It handles:
    /// - Single-character operators and delimiters (=, +, -, *, /, <, >, etc.)
    /// - Two-character operators (==, !=)
    /// - String literals enclosed in double quotes
    /// - Identifiers and keywords (let, func, if, else, etc.)
    /// - Integer literals
    /// - End of file (EOF)
    /// - Illegal characters
    ///
    /// The method automatically skips whitespace before tokenizing and advances
    /// the lexer position after returning most tokens.
    ///
    /// # Returns
    /// The next `Token` from the input stream
    ///
    /// # Examples
    /// ```
    /// use rust_impl::lexer::Lexer;
    /// use rust_impl::token::Token;
    ///
    /// let mut lexer = Lexer::new(String::from("let x = 5;"));
    /// assert_eq!(lexer.next_token(), Token::Let);
    /// assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
    /// assert_eq!(lexer.next_token(), Token::Assign);
    /// assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
    /// assert_eq!(lexer.next_token(), Token::Semicolon);
    /// assert_eq!(lexer.next_token(), Token::Eof);
    /// ```
    pub fn next_token(&mut self) -> Token {
        self.eat_whitespace();
        let token = match self.ch {
            b'=' => {
                if matches!(self.peek_char(), b'=') {
                    self.read_char();
                    self.read_char();
                    return Token::Eq;
                }
                Token::Assign
            }
            b';' => Token::Semicolon,
            b'-' => {
                if matches!(self.peek_char(), b'=') {
                    self.read_char();
                    self.read_char();
                    return Token::MinusAssign;
                }
                Token::Minus
            }
            b'.' => {
                if Lexer::is_digit(self.peek_char()) {
                    todo!("floats not supported yet")
                } else {
                    self.read_char();
                    return Token::Dot;
                }
            }
            b'?' => Token::Conditional,
            b'!' => {
                if matches!(self.peek_char(), b'=') {
                    self.read_char();
                    self.read_char();
                    return Token::NotEq;
                }
                self.read_char();
                return Token::Bang;
            }
            b'&' => {
                if matches!(self.peek_char(), b'&') {
                    self.read_char();
                    self.read_char();
                    return Token::And;
                }
                let illegal_char = self.ch as char;
                self.read_char();
                return Token::Illegal(illegal_char);
            }
            b'|' => {
                if matches!(self.peek_char(), b'|') {
                    self.read_char();
                    self.read_char();
                    return Token::Or;
                }
                let illegal_char = self.ch as char;
                self.read_char();
                return Token::Illegal(illegal_char);
            }
            b'[' => Token::LSquare,
            b']' => Token::RSquare,
            b':' => Token::Colon,
            b'/' => {
                if matches!(self.peek_char(), b'=') {
                    self.read_char();
                    self.read_char();
                    return Token::SlashAssign;
                }
                self.read_char();
                return Token::Slash;
            }
            b'*' => {
                if matches!(self.peek_char(), b'=') {
                    self.read_char();
                    self.read_char();
                    return Token::AsteriskAssign;
                }
                self.read_char();
                return Token::Asterisk;
            }
            b'<' => {
                self.read_char();
                return Token::Lt;
            }
            b'>' => {
                self.read_char();
                return Token::Gt;
            }
            b'(' => {
                self.read_char();
                return Token::LParen;
            }
            b')' => {
                self.read_char();
                return Token::RParen;
            }
            b',' => {
                self.read_char();
                return Token::Comma;
            }
            b'+' => {
                if matches!(self.peek_char(), b'=') {
                    self.read_char();
                    self.read_char();
                    return Token::PlusAssign;
                }
                self.read_char();
                return Token::Plus;
            }
            b'{' => {
                self.read_char();
                return Token::LBrace;
            }
            b'}' => {
                self.read_char();
                return Token::RBrace;
            }
            0 => return Token::Eof,
            b'"' => {
                self.read_char();
                let literal = self.read_string_literal();
                match literal {
                    Ok(val) => return Token::Str(val),
                    Err(e) => panic!("{:?}", e),
                }
            }
            _ => {
                if Lexer::is_letter(self.ch) {
                    let literal = self.read_identifier();
                    return lookup_identifier(&literal);
                } else if Lexer::is_digit(self.ch) {
                    let int = self.read_number();
                    return Token::Int(int);
                }
                let illegal_char = self.ch as char;
                self.read_char();
                return Token::Illegal(illegal_char);
            }
        };
        self.read_char();
        return token;
    }

    /// Advances the lexer to the next character in the input.
    ///
    /// Updates `ch` to the next character, moves `pos` to the current read position,
    /// and increments `read_pos`. Sets `ch` to 0 if the end of input is reached.
    fn read_char(&mut self) {
        if self.read_pos >= self.input.len() {
            self.ch = 0 // Null character ascii code (EOF)
        } else {
            self.ch = self.input.as_bytes()[self.read_pos]
        }
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    /// Skips over all whitespace characters (space, tab, newline, carriage return).
    ///
    /// Advances the lexer position until a non-whitespace character is encountered.
    fn eat_whitespace(&mut self) {
        while matches!(self.ch, b' ' | b'\t' | b'\n' | b'\r') {
            self.read_char();
        }
    }

    /// Reads a sequence of digits and returns it as a string.
    ///
    /// Continues reading characters while they are digits (0-9).
    /// Panics if the slice extraction fails (should not happen in normal operation).
    ///
    /// # Returns
    /// A string containing the numeric literal
    fn read_number(&mut self) -> String {
        let pos = self.pos;
        while Lexer::is_digit(self.ch) {
            self.read_char();
        }
        match self.input.get(pos..self.pos) {
            Some(val) => return val.to_string().clone(),
            None => panic!("Failed to extract number from input"),
        }
    }

    /// Reads an identifier (variable name, function name, or keyword).
    ///
    /// Continues reading while characters are letters, digits, or underscores.
    /// Returns an empty string if extraction fails.
    ///
    /// # Returns
    /// A string containing the identifier
    fn read_identifier(&mut self) -> String {
        let pos = self.pos;
        while Lexer::is_letter_or_underscore(self.ch) {
            self.read_char();
        }

        match self.input.get(pos..self.pos) {
            Some(val) => String::from(val),
            None => String::from(""),
        }
    }

    /// Reads a string literal, excluding the surrounding quotes.
    ///
    /// Note: The opening quote should already be consumed before calling this method.
    /// Reads characters until the closing quote is encountered, then advances past it.
    ///
    /// # Returns
    /// A string containing the literal content (without quotes)
    fn read_string_literal(&mut self) -> Result<String, String> {
        let mut result = String::new();

        // While loop to keep consuming all characters in a string literal
        while self.ch != b'"' && self.ch != 0 {
            if self.ch == b'\\' {
                self.read_char(); // Move to char after backslash

                // Check if we hit EOF after backslash
                if self.ch == 0 {
                    return Err(String::from(
                        "Unterminated string: EOF after escape character",
                    ));
                }

                // Map valid escape sequences
                let escaped_char = match self.ch {
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'"' => '"',
                    b'\\' => '\\',
                    // Invalid escape sequence - return error!
                    _ => {
                        return Err(format!("Invalid escape sequence: \\{}", self.ch as char));
                    }
                };
                result.push(escaped_char);
                self.read_char();
            } else {
                result.push(self.ch as char);
                self.read_char();
            }
        }

        // Check if we found closing quote or hit EOF
        if self.ch == 0 {
            return Err(String::from("Unterminated string: missing closing quote"));
        }

        // Consume the closing quote
        self.read_char();

        Ok(result)
    }

    /// Peeks at the next character without consuming it.
    ///
    /// Useful for lookahead operations, such as distinguishing between
    /// single-character operators (=) and two-character operators (==).
    ///
    /// # Returns
    /// The next character as a byte, or 0 if at end of input
    fn peek_char(&self) -> u8 {
        if self.read_pos >= self.input.len() {
            return 0;
        }
        self.input.as_bytes()[self.read_pos]
    }

    /// Checks if a character is a letter (a-z or A-Z).
    ///
    /// # Arguments
    /// * `ch` - The character to check as a byte
    ///
    /// # Returns
    /// `true` if the character is a letter, `false` otherwise
    fn is_letter(ch: u8) -> bool {
        (b'a'..=b'z').contains(&ch) || (b'A'..=b'Z').contains(&ch)
    }

    /// Checks if a character is a digit (0-9).
    ///
    /// # Arguments
    /// * `ch` - The character to check as a byte
    ///
    /// # Returns
    /// `true` if the character is a digit, `false` otherwise
    fn is_digit(ch: u8) -> bool {
        (b'0'..=b'9').contains(&ch)
    }

    /// Checks if a character is valid in an identifier (letter, digit, or underscore).
    ///
    /// # Arguments
    /// * `ch` - The character to check as a byte
    ///
    /// # Returns
    /// `true` if the character can be part of an identifier, `false` otherwise
    fn is_letter_or_underscore(ch: u8) -> bool {
        Lexer::is_letter(ch) || ch == b'_' || Lexer::is_digit(ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lexer() {
        let input = String::from("hello");
        let lexer = Lexer::new(input.clone());

        assert_eq!(lexer.input, input);
        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.read_pos, 1);
        assert_eq!(lexer.ch, b'h'); // First character should be read
    }

    #[test]
    fn test_new_lexer_empty() {
        let lexer = Lexer::new(String::from(""));

        assert_eq!(lexer.ch, 0); // EOF
        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.read_pos, 1);
    }

    #[test]
    fn test_read_char() {
        let mut lexer = Lexer::new(String::from("abc"));

        assert_eq!(lexer.ch, b'a');
        lexer.read_char();
        assert_eq!(lexer.ch, b'b');
        assert_eq!(lexer.pos, 1);
        assert_eq!(lexer.read_pos, 2);

        lexer.read_char();
        assert_eq!(lexer.ch, b'c');

        lexer.read_char();
        assert_eq!(lexer.ch, 0); // EOF
    }

    #[test]
    fn test_eat_whitespace() {
        let mut lexer = Lexer::new(String::from("\n   a"));
        lexer.eat_whitespace();
        assert_eq!(lexer.ch, b'a');
    }

    #[test]
    fn test_eat_whitespace_mixed() {
        let mut lexer = Lexer::new(String::from(" \t\r\n\t xyz"));
        lexer.eat_whitespace();
        assert_eq!(lexer.ch, b'x');
    }

    #[test]
    fn test_eat_whitespace_no_whitespace() {
        let mut lexer = Lexer::new(String::from("abc"));
        lexer.eat_whitespace();
        assert_eq!(lexer.ch, b'a'); // Should not advance
    }

    #[test]
    fn test_read_number() {
        let mut lexer = Lexer::new(String::from("12345"));
        let num = lexer.read_number();

        assert_eq!(num, "12345");
        assert_eq!(lexer.ch, 0); // Should be at EOF
    }

    #[test]
    fn test_read_number_with_trailing() {
        let mut lexer = Lexer::new(String::from("42 + 3"));
        let num = lexer.read_number();

        assert_eq!(num, "42");
        assert_eq!(lexer.ch, b' '); // Should stop at space
    }

    #[test]
    fn test_read_number_single_digit() {
        let mut lexer = Lexer::new(String::from("7"));
        let num = lexer.read_number();

        assert_eq!(num, "7");
    }

    #[test]
    fn test_read_identifier() {
        let mut lexer = Lexer::new(String::from("variable"));
        let ident = lexer.read_identifier();

        assert_eq!(ident, "variable");
        assert_eq!(lexer.ch, 0); // EOF
    }

    #[test]
    fn test_read_identifier_with_underscore() {
        let mut lexer = Lexer::new(String::from("my_var"));
        let ident = lexer.read_identifier();

        assert_eq!(ident, "my_var");
    }

    #[test]
    fn test_read_identifier_with_numbers() {
        let mut lexer = Lexer::new(String::from("var123"));
        let ident = lexer.read_identifier();

        assert_eq!(ident, "var123");
    }

    #[test]
    fn test_read_identifier_stops_at_operator() {
        let mut lexer = Lexer::new(String::from("foo+bar"));
        let ident = lexer.read_identifier();

        assert_eq!(ident, "foo");
        assert_eq!(lexer.ch, b'+'); // Should stop at operator
    }

    #[test]
    fn test_peek_char() {
        let lexer = Lexer::new(String::from("abc"));

        assert_eq!(lexer.ch, b'a');
        assert_eq!(lexer.peek_char(), b'b'); // Peek should not advance
        assert_eq!(lexer.ch, b'a'); // Current char unchanged
    }

    #[test]
    fn test_peek_char_at_end() {
        let mut lexer = Lexer::new(String::from("a"));
        lexer.read_char(); // Move to EOF

        assert_eq!(lexer.peek_char(), 0); // Should return 0 at EOF
    }

    #[test]
    fn test_is_letter() {
        assert!(Lexer::is_letter(b'a'));
        assert!(Lexer::is_letter(b'z'));
        assert!(Lexer::is_letter(b'A'));
        assert!(Lexer::is_letter(b'Z'));
        assert!(Lexer::is_letter(b'm'));

        assert!(!Lexer::is_letter(b'0'));
        assert!(!Lexer::is_letter(b'_'));
        assert!(!Lexer::is_letter(b' '));
        assert!(!Lexer::is_letter(b'+'));
    }

    #[test]
    fn test_is_digit() {
        assert!(Lexer::is_digit(b'0'));
        assert!(Lexer::is_digit(b'5'));
        assert!(Lexer::is_digit(b'9'));

        assert!(!Lexer::is_digit(b'a'));
        assert!(!Lexer::is_digit(b'_'));
        assert!(!Lexer::is_digit(b' '));
    }

    #[test]
    fn test_is_letter_or_underscore() {
        assert!(Lexer::is_letter_or_underscore(b'a'));
        assert!(Lexer::is_letter_or_underscore(b'Z'));
        assert!(Lexer::is_letter_or_underscore(b'_'));
        assert!(Lexer::is_letter_or_underscore(b'0'));
        assert!(Lexer::is_letter_or_underscore(b'9'));

        assert!(!Lexer::is_letter_or_underscore(b' '));
        assert!(!Lexer::is_letter_or_underscore(b'+'));
        assert!(!Lexer::is_letter_or_underscore(b'-'));
    }

    #[test]
    fn test_read_string_literal() {
        let mut lexer = Lexer::new(String::from("hello\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse valid string");

        assert_eq!(string, "hello");
        assert_eq!(lexer.ch, 0); // Should have consumed closing quote and reached EOF
    }

    #[test]
    fn test_read_string_literal_with_underscore() {
        let mut lexer = Lexer::new(String::from("hello_world\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse valid string");

        assert_eq!(string, "hello_world");
    }

    #[test]
    fn test_read_string_literal_with_spaces() {
        let mut lexer = Lexer::new(String::from("hello world\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with spaces");

        assert_eq!(string, "hello world");
    }

    #[test]
    fn test_read_string_literal_with_special_chars() {
        let mut lexer = Lexer::new(String::from("hello, world!\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with special chars");

        assert_eq!(string, "hello, world!");
    }

    #[test]
    fn test_read_string_literal_escape_newline() {
        let mut lexer = Lexer::new(String::from("hello\\nworld\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with \\n");

        assert_eq!(string, "hello\nworld");
    }

    #[test]
    fn test_read_string_literal_escape_tab() {
        let mut lexer = Lexer::new(String::from("hello\\tworld\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with \\t");

        assert_eq!(string, "hello\tworld");
    }

    #[test]
    fn test_read_string_literal_escape_carriage_return() {
        let mut lexer = Lexer::new(String::from("hello\\rworld\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with \\r");

        assert_eq!(string, "hello\rworld");
    }

    #[test]
    fn test_read_string_literal_escape_quote() {
        let mut lexer = Lexer::new(String::from("say \\\"hello\\\"\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with escaped quotes");

        assert_eq!(string, "say \"hello\"");
    }

    #[test]
    fn test_read_string_literal_escape_backslash() {
        let mut lexer = Lexer::new(String::from("path\\\\to\\\\file\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with escaped backslash");

        assert_eq!(string, "path\\to\\file");
    }

    #[test]
    fn test_read_string_literal_multiple_escapes() {
        let mut lexer = Lexer::new(String::from("line1\\nline2\\tindented\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse string with multiple escapes");

        assert_eq!(string, "line1\nline2\tindented");
    }

    #[test]
    fn test_read_string_literal_unterminated() {
        let mut lexer = Lexer::new(String::from("hello world"));
        let result = lexer.read_string_literal();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unterminated string"));
    }

    #[test]
    fn test_read_string_literal_invalid_escape() {
        let mut lexer = Lexer::new(String::from("hello\\xworld\""));
        let result = lexer.read_string_literal();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid escape sequence"));
    }

    #[test]
    fn test_read_string_literal_escape_at_eof() {
        let mut lexer = Lexer::new(String::from("hello\\"));
        let result = lexer.read_string_literal();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("EOF after escape"));
    }

    #[test]
    fn test_read_string_literal_empty() {
        let mut lexer = Lexer::new(String::from("\""));
        let string = lexer
            .read_string_literal()
            .expect("Should parse empty string");

        assert_eq!(string, "");
    }

    #[test]
    fn test_lexer_positions_update_correctly() {
        let mut lexer = Lexer::new(String::from("abc"));

        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.read_pos, 1);

        lexer.read_char();
        assert_eq!(lexer.pos, 1);
        assert_eq!(lexer.read_pos, 2);

        lexer.read_char();
        assert_eq!(lexer.pos, 2);
        assert_eq!(lexer.read_pos, 3);
    }

    // ===== next_token tests =====

    #[test]
    fn test_next_token_single_char_operators() {
        let mut lexer = Lexer::new(String::from("=+-!/*<>"));

        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Bang);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Asterisk);
        assert_eq!(lexer.next_token(), Token::Lt);
        assert_eq!(lexer.next_token(), Token::Gt);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_two_char_operators() {
        let mut lexer = Lexer::new(String::from("== !="));

        assert_eq!(lexer.next_token(), Token::Eq);
        assert_eq!(lexer.next_token(), Token::NotEq);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_delimiters() {
        let mut lexer = Lexer::new(String::from("(){},;"));

        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_integers() {
        let mut lexer = Lexer::new(String::from("5 10 999"));

        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Int(String::from("10")));
        assert_eq!(lexer.next_token(), Token::Int(String::from("999")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_identifiers() {
        let mut lexer = Lexer::new(String::from("foo bar_baz x123"));

        assert_eq!(lexer.next_token(), Token::Ident(String::from("foo")));
        assert_eq!(lexer.next_token(), Token::Ident(String::from("bar_baz")));
        assert_eq!(lexer.next_token(), Token::Ident(String::from("x123")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_keywords() {
        let mut lexer = Lexer::new(String::from("func let if else return true false for"));

        assert_eq!(lexer.next_token(), Token::Function);
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::Return);
        assert_eq!(lexer.next_token(), Token::True);
        assert_eq!(lexer.next_token(), Token::False);
        assert_eq!(lexer.next_token(), Token::For);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_literal() {
        let mut lexer = Lexer::new(String::from("\"hello\""));

        assert_eq!(lexer.next_token(), Token::Str(String::from("hello")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_with_underscore() {
        let mut lexer = Lexer::new(String::from("\"hello_world\""));

        assert_eq!(lexer.next_token(), Token::Str(String::from("hello_world")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_empty_string() {
        let mut lexer = Lexer::new(String::from("\"\""));

        assert_eq!(lexer.next_token(), Token::Str(String::from("")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_with_spaces() {
        let mut lexer = Lexer::new(String::from("\"hello world\""));

        assert_eq!(lexer.next_token(), Token::Str(String::from("hello world")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_with_special_chars() {
        let mut lexer = Lexer::new(String::from("\"Hello, World!\""));

        assert_eq!(
            lexer.next_token(),
            Token::Str(String::from("Hello, World!"))
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_with_escape_sequences() {
        let mut lexer = Lexer::new(String::from("\"line1\\nline2\\ttab\""));

        assert_eq!(
            lexer.next_token(),
            Token::Str(String::from("line1\nline2\ttab"))
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_with_escaped_quote() {
        let mut lexer = Lexer::new(String::from("\"He said \\\"hi\\\"\""));

        assert_eq!(
            lexer.next_token(),
            Token::Str(String::from("He said \"hi\""))
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_string_with_escaped_backslash() {
        let mut lexer = Lexer::new(String::from("\"path\\\\to\\\\file\""));

        assert_eq!(
            lexer.next_token(),
            Token::Str(String::from("path\\to\\file"))
        );
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    #[should_panic(expected = "Unterminated string")]
    fn test_next_token_unterminated_string() {
        let mut lexer = Lexer::new(String::from("\"hello world"));
        lexer.next_token(); // Should panic
    }

    #[test]
    #[should_panic(expected = "Invalid escape sequence")]
    fn test_next_token_invalid_escape_sequence() {
        let mut lexer = Lexer::new(String::from("\"hello\\xworld\""));
        lexer.next_token(); // Should panic
    }

    #[test]
    fn test_next_token_illegal_character() {
        let mut lexer = Lexer::new(String::from("@"));

        assert_eq!(lexer.next_token(), Token::Illegal('@'));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_whitespace_handling() {
        let mut lexer = Lexer::new(String::from("  \t\n  5  \r\n  +  10  "));

        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Int(String::from("10")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_assignment_vs_equality() {
        let mut lexer = Lexer::new(String::from("x = 5 == 5"));

        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Eq);
        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_bang_vs_not_equal() {
        let mut lexer = Lexer::new(String::from("!true != false"));

        assert_eq!(lexer.next_token(), Token::Bang);
        assert_eq!(lexer.next_token(), Token::True);
        assert_eq!(lexer.next_token(), Token::NotEq);
        assert_eq!(lexer.next_token(), Token::False);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_simple_expression() {
        let mut lexer = Lexer::new(String::from("5 + 10 * 2"));

        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Int(String::from("10")));
        assert_eq!(lexer.next_token(), Token::Asterisk);
        assert_eq!(lexer.next_token(), Token::Int(String::from("2")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_variable_declaration() {
        let mut lexer = Lexer::new(String::from("let x = 5;"));

        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_function_declaration() {
        let mut lexer = Lexer::new(String::from("func add(x, y) { return x + y; }"));

        assert_eq!(lexer.next_token(), Token::Function);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("add")));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("y")));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::Return);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("y")));
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_if_else_statement() {
        let mut lexer = Lexer::new(String::from(
            "if (x < 10) { return true; } else { return false; }",
        ));

        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::Lt);
        assert_eq!(lexer.next_token(), Token::Int(String::from("10")));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::Return);
        assert_eq!(lexer.next_token(), Token::True);
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::Return);
        assert_eq!(lexer.next_token(), Token::False);
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_consecutive_operators() {
        let mut lexer = Lexer::new(String::from("+-*/"));

        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Asterisk);
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_empty_input() {
        let mut lexer = Lexer::new(String::from(""));

        assert_eq!(lexer.next_token(), Token::Eof);
        assert_eq!(lexer.next_token(), Token::Eof); // Should continue returning EOF
    }

    #[test]
    fn test_next_token_only_whitespace() {
        let mut lexer = Lexer::new(String::from("   \t\n\r   "));

        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_mixed_illegal_and_valid() {
        let mut lexer = Lexer::new(String::from("5 @ 10"));

        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Illegal('@'));
        assert_eq!(lexer.next_token(), Token::Int(String::from("10")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_number_followed_by_operator() {
        let mut lexer = Lexer::new(String::from("42+3"));

        assert_eq!(lexer.next_token(), Token::Int(String::from("42")));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Int(String::from("3")));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_identifier_followed_by_paren() {
        let mut lexer = Lexer::new(String::from("foo("));

        assert_eq!(lexer.next_token(), Token::Ident(String::from("foo")));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_next_token_compound_assignment_operators() {
        let mut lexer = Lexer::new(String::from("x += 5; y -= 3; z *= 2; w /= 4;"));

        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::PlusAssign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Ident(String::from("y")));
        assert_eq!(lexer.next_token(), Token::MinusAssign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("3")));
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Ident(String::from("z")));
        assert_eq!(lexer.next_token(), Token::AsteriskAssign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("2")));
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Ident(String::from("w")));
        assert_eq!(lexer.next_token(), Token::SlashAssign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("4")));
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_compound_vs_regular_operators() {
        // Test that += is different from + followed by =
        let mut lexer = Lexer::new(String::from("x+=5"));
        assert_eq!(lexer.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer.next_token(), Token::PlusAssign);
        assert_eq!(lexer.next_token(), Token::Int(String::from("5")));

        // Test regular operators still work
        let mut lexer2 = Lexer::new(String::from("x + y"));
        assert_eq!(lexer2.next_token(), Token::Ident(String::from("x")));
        assert_eq!(lexer2.next_token(), Token::Plus);
        assert_eq!(lexer2.next_token(), Token::Ident(String::from("y")));
    }
}
