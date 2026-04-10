//! Parser implementation for the Goophy language using a Pratt parser.
//!
//! # Parser Implementation
//!
//! This module implements a hand-written Pratt parser for the Goophy language.
//! A lot of modern compilers use parsers that are generated from grammar specifications,
//! but this implementation is done manually for educational purposes.
//!
//! ## Grammar Background
//!
//! A grammar/parsing table is a data structure that tells a parser what action
//! to take based on the current state and the next input token.
//! The basic idea is that instead of writing parsing logic with if/switch statements,
//! you encode all parsing decisions in a table, and the parser looks up what to do.
//!
//! **Current state + next token = what to do next**
//!
//! Let's use a simple grammar for arithmetic:
//! ```text
//! E -> E + T | T
//! T -> T * F | F
//! F -> ( E ) | number
//! ```
//!
//! This is BNF (Backus-Naur Form):
//!
//! ### Basic Symbols:
//! - `→` means "can be replaced by" or "produces"
//! - `|` means "OR" (alternative options)
//! - Capital letters (E, T, F) are non-terminals (abstract concepts)
//! - Lowercase/symbols (+, *, number) are terminals (actual tokens)
//!
//! ### Terminals vs Non-terminals:
//! - **Terminals** = Actual tokens (the atoms of your language that cannot be broken down further)
//! - **Non-terminals** = Things that can be broken down further, e.g., expressions
//!
//! ### Reading the Rules:
//! - `E -> E + T | T` means an Expression can be an Expression followed by `+` followed by a Term, OR just a Term
//! - `T -> T * F | F` means a Term can be a Term followed by `*` followed by a Factor, OR just a Factor
//! - `F -> ( E ) | number` means a Factor can be a parenthesized expression or just a number
//!
//! ### Precedence Hierarchy:
//! ```text
//! E (Expression)  ← Lowest precedence  (+)
//! T (Term)        ← Higher precedence  (*)
//! F (Factor)      ← Highest precedence (numbers, parentheses)
//! ```
//!
//! The key insight is that to get to a `+`, you must first go through T and F. This makes `*` bind tighter than `+`.
//! To evaluate `+`, you must always evaluate T first, and because to evaluate T you have to evaluate F,
//! you always have to do F and T first to get to `+`.
//!
//! ### Example Parse Tree for `3 + 5`:
//! ```text
//!      E
//!     /|\
//!    E + T
//!    |   |
//!    T   F
//!    |   |
//!    F   number(5)
//!    |
//!  number(3)
//! ```
//!
//! Left-recursive rules like `E -> E + T` parse `2 + 3 + 4` as `(2 + 3) + 4`
//!
//! ## Pratt Parser
//!
//! The idea behind the Pratt parser is that you treat an infix operator as a prefix
//! and then pass down a precedence to "group" operations together.
//!
//! ### Examples:
//! - `2 + 3 + 4` becomes `2 + (prefix) (3 + 4)` (expression being parsed).
//!   You will see that `+` is the next operator, which is not of higher precedence,
//!   so you immediately return 3, becoming `(2 + 3) + 4`.
//!
//! - `2 + 3 * 4` becomes `2 + (prefix) (3 * 4)`. However, `*` is of higher precedence,
//!   so it gets evaluated first and grouped together, then returned as an expression.
//!
//! In this way, operators of higher precedence will always be grouped together and evaluated first.

use crate::ast::{ExpressionNode, InfixOp, Node, PrefixOp, StatementNode};
use crate::lexer::Lexer;
use crate::token::Token;

/// Operator precedence levels used in the Pratt parser.
///
/// Lower numeric values indicate lower precedence. Higher precedence operators
/// bind more tightly than lower precedence operators.
///
/// # Precedence Order (lowest to highest):
/// 1. `Lowest` - Default precedence for starting expression parsing
/// 2. `ASSIGN` - Assignment operators (`=`)
/// 3. `LOGICAL` - Logical operators (`&&`, `||`)
/// 4. `EQUALS` - Equality operators (`==`, `!=`)
/// 5. `LESSGREATER` - Comparison operators (`<`, `>`)
/// 6. `SUM` - Addition/subtraction operators (`+`, `-`)
/// 7. `PRODUCT` - Multiplication/division operators (`*`, `/`)
/// 8. `PREFIX` - Prefix operators (`-x`, `!x`)
/// 9. `CALL` - Function call operator (`function()`)
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest = 1,
    ASSIGN,      // =
    LOGICAL,     // && ||
    EQUALS,      // ==
    LESSGREATER, // < >
    SUM,         // + -
    PRODUCT,     // * /
    PREFIX,      // -x !x
    CALL,        // function()
}

/// The main parser structure that maintains parsing state.
///
/// The parser uses a two-token lookahead strategy with `curr_token` and `peek_token`
/// to make parsing decisions.
pub struct Parser {
    /// The lexer that provides tokens
    pub l: Lexer,
    /// The current token being processed
    pub curr_token: Token,
    /// The next token (lookahead)
    pub peek_token: Token,
    /// Counter for assigning unique node IDs
    next_node_id: usize,
}

impl Parser {
    /// Creates a new parser from a lexer.
    ///
    /// The parser initializes by reading the first two tokens from the lexer
    /// to populate both `curr_token` and `peek_token`.
    ///
    /// # Arguments
    ///
    /// * `l` - A lexer instance that will provide tokens
    ///
    /// # Example
    ///
    /// ```ignore
    /// let lexer = Lexer::new("let x = 5;".to_string());
    /// let mut parser = Parser::new(lexer);
    /// ```
    pub fn new(l: Lexer) -> Self {
        let mut p = Parser {
            l,
            curr_token: Token::Let,
            peek_token: Token::Eof,
            next_node_id: 0,
        };

        p.next_token();
        p.next_token();
        p
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    /// Advances to the next token.
    ///
    /// Moves `peek_token` to `curr_token` and reads a new token into `peek_token`.
    fn next_token(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    /// Parses the entire program.
    ///
    /// A program consists of a sequence of statements that are parsed until
    /// the end of file (EOF) token is reached.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Node)` - A Program node containing all parsed statements
    /// - `Err(String)` - An error message if parsing fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut parser = Parser::new(lexer);
    /// match parser.parse_program() {
    ///     Ok(program) => println!("Parsed successfully"),
    ///     Err(e) => println!("Parse error: {}", e),
    /// }
    /// ```
    pub fn parse_program(&mut self) -> Result<Node, String> {
        let mut stmts: Vec<Box<Node>> = vec![];
        let mut last_ends_without_semicolon = false;

        while !matches!(self.curr_token, Token::Eof) {
            let stmt = self.parse_statement(false)?;
            stmts.push(Box::new(stmt));

            // After parse_statement, curr_token is at the next token (might be semicolon, another statement, or EOF)
            // Check if there's a semicolon and consume it
            if matches!(self.curr_token, Token::Semicolon) {
                last_ends_without_semicolon = false;
                self.next_token(); // Move past the semicolon
            } else {
                last_ends_without_semicolon = true;
                // Don't advance - curr_token is already at the next statement
            }
        }

        // If the last statement was an expression without a semicolon, make it the implicit return
        let implicit_return = if last_ends_without_semicolon && !stmts.is_empty() {
            // Check if the last statement is an expression
            let is_expression = matches!(stmts.last().unwrap().as_ref(), Node::ExpressionNode(_));

            if is_expression {
                // Safe to unwrap because we checked is_empty above
                if let Node::ExpressionNode(expr) = stmts.pop().unwrap().as_ref() {
                    Some(expr.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Node::StatementNode(StatementNode::Program {
            statements: stmts,
            implicit_return,
            id: 0,
        }))
    }

    /// Parses a single statement.
    ///
    /// Determines the type of statement based on the current token and
    /// delegates to the appropriate parsing method.
    ///
    /// # Supported Statement Types:
    /// - `let` - Variable declarations
    /// - `return` - Return statements
    /// - `for` - For loop statements
    /// - `func` - Function declarations
    /// - Everything else - Expression statements
    ///
    /// After parsing, this function consumes an optional trailing semicolon.
    fn parse_statement(&mut self, consume_semicolon: bool) -> Result<Node, String> {
        let stmt = match self.curr_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => self.parse_break_statement(),
            Token::Continue => self.parse_continue_statement(),
            Token::For => self.parse_for_statement(),
            Token::Function => self.parse_function_declaration(),
            _ => self.parse_expression_statement(),
        }?;

        // Consume semicolon
        if consume_semicolon && matches!(self.curr_token, Token::Semicolon) {
            self.next_token();
        }
        Ok(stmt)
    }

    /// Parses a return statement.
    ///
    /// Syntax: `return <expression>;`
    ///
    /// The semicolon is handled by the caller (parse_statement).
    fn parse_return_statement(&mut self) -> Result<Node, String> {
        self.next_token();

        let return_value = match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(val) => val,
            _ => return Err(String::from("Expected expression in return statement")),
        };

        Ok(Node::StatementNode(StatementNode::Return {
            token: Token::Return {},
            return_value,
            id: 0,
        }))
    }

    /// Parses a continue statement.
    ///
    /// Syntax: `continue`
    ///
    /// Continue statements are used inside loops to skip the rest of the current
    /// iteration and jump back to the loop condition.
    ///
    /// # Returns
    ///
    /// A Continue statement node.
    ///
    /// # Examples
    ///
    /// ```
    /// // for i < 10 {
    /// //     if i == 5 {
    /// //         continue;
    /// //     }
    /// //     print(i);
    /// // }
    /// ```
    fn parse_continue_statement(&mut self) -> Result<Node, String> {
        self.next_token();

        Ok(Node::StatementNode(StatementNode::Continue {
            token: Token::Continue,
            id: 0,
        }))
    }

    /// Parses a break statement.
    ///
    /// Syntax: `break`
    ///
    /// Break statements are used inside loops to exit the loop immediately,
    /// skipping any remaining iterations.
    ///
    /// # Returns
    ///
    /// A Break statement node.
    ///
    /// # Examples
    ///
    /// ```
    /// // for true {
    /// //     if condition {
    /// //         break;
    /// //     }
    /// // }
    /// ```
    fn parse_break_statement(&mut self) -> Result<Node, String> {
        self.next_token();

        Ok(Node::StatementNode(StatementNode::Break {
            token: Token::Break,
            id: 0,
        }))
    }

    /// Parses a function declaration statement.
    ///
    /// Syntax: `func <identifier>(<parameters>) { <body> }`
    ///
    /// # Returns
    ///
    /// Returns an error if:
    /// - The identifier is missing
    /// - The opening brace is missing
    fn parse_function_declaration(&mut self) -> Result<Node, String> {
        if !matches!(self.peek_token, Token::Ident(_)) {
            return Err(String::from("Expected identifier after 'func' keyword"));
        }

        self.next_token();
        let identifier = self.parse_prefix()?;

        // After parse_prefix, curr_token is already at '('
        let params = self.parse_function_parameters()?;

        // After parse_function_parameters, curr_token is at ')', move to '{'
        self.next_token();
        if !matches!(self.curr_token, Token::LBrace) {
            return Err(String::from("Expected '{' to start function body"));
        }

        let result = Node::StatementNode(StatementNode::FuncDeclr {
            token: Token::Function,
            identifier: match identifier {
                Node::ExpressionNode(val) => val,
                _ => {
                    return Err(String::from(
                        "Expected identifier expression in function declaration",
                    ));
                }
            },
            func: ExpressionNode::Function {
                token: Token::Function,
                parameters: params,
                body: Box::new(self.parse_block_statement()?),
                id: 0,
            },
            id: 0,
        });

        // After parse_block_statement, curr_token is at '}', move past it
        self.next_token();

        Ok(result)
    }

    /// Parses a for loop statement.
    ///
    /// Syntax: `for (<condition>) { <body> }`
    ///
    /// Note: The parentheses around the condition are optional in this implementation.
    ///
    /// # Returns
    ///
    /// Returns an error if:
    /// - Expected left parenthesis is missing
    /// - Expected right parenthesis or left brace is missing
    fn parse_for_statement(&mut self) -> Result<Node, String> {
        if !matches!(self.peek_token, Token::LParen) {
            return Err(String::from("Expected '(' after 'for' keyword"));
        }

        self.next_token();
        let condition = match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(val) => val,
            _ => return Err(String::from("Expected expression in for loop condition")),
        };
        // After parse_expression of (condition), curr_token is at '{'
        if !matches!(self.curr_token, Token::LBrace) {
            return Err(String::from("Expected '{' to start for loop body"));
        }
        let for_block = self.parse_block_statement()?;

        // After parse_block_statement, curr_token is at '}', move past it
        self.next_token();

        Ok(Node::StatementNode(StatementNode::For {
            token: Token::For,
            condition: condition,
            for_block: Box::new(for_block),
            id: 0,
        }))
    }

    /// Parses a let (variable declaration) statement.
    ///
    /// Syntax: `let <identifier> = <expression>;`
    ///
    /// The semicolon is handled by the caller (parse_statement).
    ///
    /// # Returns
    ///
    /// Returns an error if:
    /// - The identifier is not a valid identifier token
    /// - The assignment operator `=` is missing
    fn parse_let_statement(&mut self) -> Result<Node, String> {
        let saved_token = self.curr_token.clone();

        // Move to identifier
        self.next_token();
        let node_id = self.next_id();
        let identifier = ExpressionNode::Identifier {
            token: self.curr_token.clone(),
            value: match self.curr_token.clone() {
                Token::Ident(val) => val,
                _ => return Err(String::from("Expected identifier in let statement")),
            },
            id: node_id as i32,
        };

        // Ensure next token is assignment operator
        if !matches!(self.peek_token, Token::Assign) {
            return Err(String::from(
                "Expected '=' after identifier in let statement",
            ));
        }

        // Move to assignment operator
        self.next_token();
        // Move to first token of expression
        self.next_token();

        // Parse the value expression
        let expression = match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(n) => n,
            _ => return Err(String::from("THIS SHOULD NEVER HAPPEN")),
        };

        Ok(Node::StatementNode(StatementNode::Let {
            token: saved_token,
            value: expression,
            name: identifier,
            id: 0,
        }))
    }

    /// Parses an expression statement.
    ///
    /// An expression statement is an expression followed by an optional semicolon.
    /// The semicolon is handled by the caller (parse_statement).
    fn parse_expression_statement(&mut self) -> Result<Node, String> {
        self.parse_expression(Precedence::Lowest)
    }

    /// Parses a prefix expression.
    ///
    /// Prefix expressions include:
    /// - Identifiers
    /// - Integer literals
    /// - Boolean literals (`true`, `false`)
    /// - Prefix operators (`!`, `-`)
    /// - Parenthesized expressions
    /// - If expressions
    /// - Function literals
    ///
    /// # Returns
    ///
    /// Returns an error if the current token doesn't match any known prefix expression.
    fn parse_prefix(&mut self) -> Result<Node, String> {
        let curr_token = self.curr_token.clone();
        match self.curr_token.clone() {
            Token::Ident(val) => {
                let node_id = self.next_id();
                self.next_token();
                return Ok(Node::ExpressionNode(ExpressionNode::Identifier {
                    token: curr_token,
                    value: val,
                    id: node_id as i32,
                }));
            }
            Token::Int(val) => {
                let int_value: i64 = match val.parse() {
                    Ok(num) => num,
                    Err(e) => return Err(e.to_string()),
                };
                self.next_token();
                return Ok(Node::ExpressionNode(ExpressionNode::Integer {
                    token: curr_token.clone(),
                    value: int_value,
                    id: 0,
                }));
            }
            Token::Str(val) => {
                self.next_token();
                return Ok(Node::ExpressionNode(ExpressionNode::String {
                    token: curr_token.clone(),
                    value: val.clone(),
                    id: 0,
                }));
            }
            Token::True => {
                self.next_token();
                return Ok(Node::ExpressionNode(ExpressionNode::Boolean {
                    token: curr_token.clone(),
                    value: true,
                    id: 0,
                }));
            }
            Token::False => {
                self.next_token();
                return Ok(Node::ExpressionNode(ExpressionNode::Boolean {
                    token: curr_token.clone(),
                    value: false,
                    id: 0,
                }));
            }
            Token::Bang | Token::Minus => {
                let token = self.curr_token.clone();
                let operator = match token {
                    Token::Bang => PrefixOp::Not,
                    Token::Minus => PrefixOp::Negative,
                    _ => unreachable!(),
                };
                // Move to the next token (the operand)
                self.next_token();
                // Parse the right side with PREFIX precedence
                let right = match self.parse_expression(Precedence::PREFIX)? {
                    Node::ExpressionNode(expr) => expr,
                    _ => return Err(String::from("Expected expression after prefix operator")),
                };
                return Ok(Node::ExpressionNode(ExpressionNode::Prefix {
                    token,
                    operator,
                    right: Box::new(right),
                    id: 0,
                }));
            }
            Token::LParen => {
                // Move past the '('
                self.next_token();
                // Parse the expression inside parentheses with lowest precedence
                // to allow any expression to be parenthesized
                let expr = self.parse_expression(Precedence::Lowest)?;
                // Expect a closing ')'
                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after expression"));
                }
                self.next_token();
                return Ok(expr);
            }
            Token::LSquare => self.parse_array_literal(),
            Token::If => self.parse_if_expression(),
            Token::LBrace => self.parse_hash_literal(),
            Token::Function => self.parse_function_literal(),
            _ => Err(String::from(format!(
                "Unexpected token {:?} in prefix position",
                self.curr_token.clone()
            ))),
        }
    }

    /// Parses an expression using Pratt parsing with the given precedence.
    ///
    /// This is the core of the Pratt parser algorithm. It parses a left expression,
    /// then continues parsing infix expressions as long as they have higher precedence
    /// than the current precedence level.
    ///
    /// # Arguments
    ///
    /// * `p` - The current precedence level
    ///
    /// # Returns
    ///
    /// The parsed expression node
    fn parse_expression(&mut self, p: Precedence) -> Result<Node, String> {
        let mut left_exp = self.parse_prefix()?;
        // Handle comma-separated expressions (used in function parameters/arguments)
        if matches!(self.curr_token, Token::Comma) {
            return Ok(left_exp);
        }

        while !matches!(self.curr_token, Token::Semicolon)
            && p < self.get_precedence(&self.curr_token)
        {
            left_exp = self.parse_infix(&left_exp)?;
        }
        Ok(left_exp)
    }

    /// Gets the precedence level for a given token.
    ///
    /// # Arguments
    ///
    /// * `t` - The token to get precedence for
    ///
    /// # Returns
    ///
    /// The precedence level for the token, or `Precedence::Lowest` if the token
    /// is not an operator.
    fn get_precedence(&self, t: &Token) -> Precedence {
        match t {
            Token::Assign
            | Token::PlusAssign
            | Token::MinusAssign
            | Token::AsteriskAssign
            | Token::SlashAssign => Precedence::ASSIGN,
            Token::And | Token::Or => Precedence::LOGICAL,
            Token::Eq | Token::NotEq => Precedence::EQUALS,
            Token::Lt | Token::Gt => Precedence::LESSGREATER,
            Token::Plus | Token::Minus => Precedence::SUM,
            Token::Asterisk | Token::Slash => Precedence::PRODUCT,
            Token::Dot => Precedence::CALL,
            Token::LParen => Precedence::CALL,
            Token::LSquare => Precedence::CALL,
            _ => Precedence::Lowest,
        }
    }

    /// Parses an if expression.
    ///
    /// Syntax: `if (<condition>) { <if_block> } else { <else_block> }`
    ///
    /// The `else` block is optional.
    ///
    /// # Returns
    ///
    /// Returns an error if:
    /// - The opening parenthesis is missing
    /// - The closing parenthesis is missing
    /// - The opening brace for the if block is missing
    /// - The opening brace for the else block is missing (when else is present)
    fn parse_if_expression(&mut self) -> Result<Node, String> {
        if !matches!(self.peek_token, Token::LParen) {
            return Err(String::from("Expected '(' after 'if' keyword"));
        }

        self.next_token();

        // Parse the condition expression
        let condition_exp = match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(val) => val,
            _ => return Err(String::from("Expected expression in if condition")),
        };

        if !matches!(self.curr_token, Token::LBrace) {
            return Err(String::from("Expected '{' to start if block"));
        }
        let if_block = self.parse_block_statement()?;
        let mut else_block = None;

        self.next_token();
        if matches!(self.curr_token, Token::Else) {
            self.next_token();
            if !matches!(self.curr_token, Token::LBrace) {
                return Err(String::from("Expected '{' to start else block"));
            }
            else_block = Some(Box::new(self.parse_block_statement()?));
            // After else block, curr_token is at '}', move past it
            self.next_token();
        }
        Ok(Node::ExpressionNode(ExpressionNode::If {
            token: Token::If,
            condition: Box::new(condition_exp),
            if_block: Box::new(if_block),
            else_block: else_block,
            id: 0,
        }))
    }

    /// Parses a function literal expression.
    ///
    /// Syntax: `func(<parameters>) { <body> }`
    ///
    /// Function literals can be assigned to variables or passed as arguments.
    /// Example: `let x = func(y, z) { return y + z };`
    ///
    /// # Returns
    ///
    /// Returns an error if:
    /// - The opening parenthesis is missing
    /// - The opening brace is missing
    fn parse_function_literal(&mut self) -> Result<Node, String> {
        if !matches!(self.peek_token, Token::LParen) {
            return Err(String::from("Expected '(' after 'func' keyword"));
        }
        // Move into left parenthesis
        self.next_token();
        let params = self.parse_function_parameters()?;

        // Move into left brace
        self.next_token();

        if !matches!(self.curr_token, Token::LBrace) {
            return Err(String::from("Expected '{' to start function body"));
        }

        let result = Node::ExpressionNode(ExpressionNode::Function {
            token: Token::Function,
            parameters: params,
            body: Box::new(self.parse_block_statement()?),
            id: 0,
        });

        // After parse_block_statement, curr_token is at '}', move past it
        self.next_token();

        Ok(result)
    }

    /// Parses a function call expression.
    ///
    /// Syntax: `<function>(<arguments>)`
    ///
    /// # Arguments
    ///
    /// * `func` - The function expression being called
    ///
    /// # Returns
    ///
    /// A Call expression node containing the function and its arguments
    fn parse_call_expression(&mut self, func: ExpressionNode) -> Result<Node, String> {
        Ok(Node::ExpressionNode(ExpressionNode::Call {
            token: self.curr_token.clone(),
            function: Box::new(func),
            arguments: self.parse_call_arguments()?,
            id: 0,
        }))
    }

    /// Parses the arguments of a function call.
    ///
    /// Syntax: `(<arg1>, <arg2>, ...)`
    ///
    /// # Returns
    ///
    /// A vector of expression nodes representing the arguments, or an error if parsing fails.
    fn parse_call_arguments(&mut self) -> Result<Vec<Box<ExpressionNode>>, String> {
        let mut args = vec![];
        if matches!(self.peek_token, Token::RParen) {
            self.next_token();
            self.next_token();
            return Ok(args);
        }

        // Go to first argument
        self.next_token();
        args.push(match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(val) => Box::new(val),
            Node::StatementNode(_) => {
                return Err(String::from("Expected expression in function arguments"));
            }
        });

        while matches!(self.curr_token, Token::Comma) {
            self.next_token();
            args.push(match self.parse_expression(Precedence::Lowest)? {
                Node::ExpressionNode(val) => Box::new(val),
                Node::StatementNode(_) => {
                    return Err(String::from("Expected expression in function arguments"));
                }
            });
        }

        if !matches!(self.curr_token, Token::RParen) {
            return Err(String::from("Expected ')' after function arguments"));
        }

        self.next_token();
        Ok(args)
    }

    fn parse_hash_literal(&mut self) -> Result<Node, String> {
        // Currently curr token is at {
        // move to first token of expression
        self.next_token();
        let mut pairs = vec![];

        while !matches!(self.curr_token, Token::RBrace) {
            let key = match self.parse_expression(Precedence::Lowest)? {
                Node::ExpressionNode(val) => Box::new(val),
                Node::StatementNode(_) => {
                    return Err(String::from("Expected expression in key of hashmap"));
                }
            };
            if !matches!(self.curr_token, Token::Colon) {
                return Err(String::from(
                    "colon not found for key value assignment in hashmap",
                ));
            }
            self.next_token();

            let value = match self.parse_expression(Precedence::Lowest)? {
                Node::ExpressionNode(val) => Box::new(val),
                Node::StatementNode(_) => {
                    return Err(String::from("Expected expression in value of hashmap"));
                }
            };

            pairs.push((key, value));

            // If current token isnt brace
            if !matches!(self.curr_token, Token::RBrace) {
                // if current token is comma, then next
                if matches!(self.curr_token, Token::Comma) {
                    self.next_token();
                } else {
                    return Err(String::from("No comma in hashmap"));
                }
            }
        }

        if !matches!(self.curr_token, Token::RBrace) {
            return Err(String::from("Expected '}' after hash literal"));
        }

        self.next_token();

        Ok(Node::ExpressionNode(ExpressionNode::HashMap {
            token: Token::Colon,
            pairs,
            id: 0,
        }))
    }

    /// Parses function parameters.
    ///
    /// Syntax: `(<param1>, <param2>, ...)`
    ///
    /// Parameters are comma-separated identifiers. Empty parameter lists are allowed.
    ///
    /// # Returns
    ///
    /// A vector of identifier expression nodes representing the parameters.
    fn parse_function_parameters(&mut self) -> Result<Vec<Box<ExpressionNode>>, String> {
        let mut identifiers = vec![];
        if matches!(self.peek_token, Token::RParen) {
            // Move to right parenthesis, currently at left parenthesis
            self.next_token();
            return Ok(identifiers);
        }
        // Move to first identifier
        self.next_token();

        identifiers.push(Box::new(ExpressionNode::Identifier {
            token: self.curr_token.clone(),
            value: self.curr_token.to_string(),
            id: self.next_id() as i32,
        }));

        self.next_token();

        // Continue while comma-separated
        while matches!(self.curr_token, Token::Comma) {
            self.next_token();
            identifiers.push(Box::new(ExpressionNode::Identifier {
                token: self.curr_token.clone(),
                value: self.curr_token.to_string(),
                id: self.next_id() as i32,
            }));
            self.next_token();
        }

        if !matches!(self.curr_token, Token::RParen) {
            return Err(String::from("Expected ')' after function parameters"));
        }

        Ok(identifiers)
    }

    /// Parses an array literal expression.
    ///
    /// Syntax: `[<element1>, <element2>, ...]`
    ///
    /// Array literals contain comma-separated expressions. Empty arrays are allowed.
    ///
    /// # Returns
    ///
    /// An Array expression node containing all the elements.
    ///
    /// # Example
    ///
    /// ```ignore
    /// [1, 2, 3]
    /// ["hello", "world"]
    /// []
    /// [x + 1, y * 2]
    /// ```
    fn parse_array_literal(&mut self) -> Result<Node, String> {
        let token = self.curr_token.clone();
        let mut elements = vec![];

        // Check for empty array
        if matches!(self.peek_token, Token::RSquare) {
            // Move to ']'
            self.next_token();
            // Move past ']'
            self.next_token();
            return Ok(Node::ExpressionNode(ExpressionNode::Array {
                token,
                elements,
                id: 0,
            }));
        }

        // Move to first element
        self.next_token();

        // Parse first element
        elements.push(match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(val) => Box::new(val),
            Node::StatementNode(_) => {
                return Err(String::from("Expected expression in array literal"));
            }
        });

        // Parse remaining elements
        while matches!(self.curr_token, Token::Comma) {
            self.next_token();
            elements.push(match self.parse_expression(Precedence::Lowest)? {
                Node::ExpressionNode(val) => Box::new(val),
                Node::StatementNode(_) => {
                    return Err(String::from("Expected expression in array literal"));
                }
            });
        }

        // Expect closing bracket
        if !matches!(self.curr_token, Token::RSquare) {
            return Err(String::from("Expected ']' after array elements"));
        }

        // Move past ']'
        self.next_token();

        Ok(Node::ExpressionNode(ExpressionNode::Array {
            token,
            elements,
            id: 0,
        }))
    }

    /// Parses a block statement.
    ///
    /// Syntax: `{ <statement1> <statement2> ... }`
    ///
    /// A block contains zero or more statements enclosed in braces.
    ///
    /// # Implicit Returns
    ///
    /// If the last item in a block is an expression without a semicolon,
    /// it becomes the implicit return value of the block. This allows blocks
    /// to produce values.
    ///
    /// # Examples
    ///
    /// ```
    /// // { let x = 5; x + 1 }  // implicit return: x + 1
    /// // { print("hello"); }   // no implicit return (statement)
    /// // { 42; }               // no implicit return (semicolon)
    /// ```
    ///
    /// # Returns
    ///
    /// A Block statement node containing all the statements in the block
    /// and an optional implicit return expression.
    fn parse_block_statement(&mut self) -> Result<StatementNode, String> {
        let token = self.curr_token.clone();
        let mut statements = vec![];
        let mut implicit_return = None;
        self.next_token();

        while !matches!(self.curr_token, Token::RBrace) && !matches!(self.curr_token, Token::Eof) {
            let stmt = self.parse_statement(false)?;
            match stmt {
                Node::ExpressionNode(e) => {
                    // Expression nodes can either be:
                    // 1. Expression statements (with semicolon)
                    // 2. Implicit return candidates (without semicolon)
                    if matches!(self.curr_token, Token::Semicolon) {
                        // Expression with semicolon - convert to Expression statement
                        statements.push(Box::new(StatementNode::Expression {
                            token: self.curr_token.clone(),
                            expression: e,
                            id: 0,
                        }));
                    } else {
                        // Expression without semicolon - potential implicit return
                        // We store it temporarily; if another statement follows,
                        // this will be converted to a regular statement
                        implicit_return = Some(e);
                    }
                }
                Node::StatementNode(s) => {
                    // If we encounter a statement, any previously seen expression
                    // without a semicolon was NOT the last expression, so we need
                    // to convert it to a regular Expression statement
                    if let Some(ref prev_expr) = implicit_return {
                        statements.push(Box::new(StatementNode::Expression {
                            token: self.curr_token.clone(),
                            expression: prev_expr.clone(),
                            id: 0,
                        }));
                        implicit_return = None;
                    }

                    statements.push(Box::new(s));

                    // Consume optional semicolon after statements (if not at closing brace)
                    if !matches!(self.curr_token, Token::RBrace)
                        && matches!(self.peek_token, Token::Semicolon)
                    {
                        self.next_token();
                    }
                }
            }

            // Advance to next token unless we're at the closing brace
            if !matches!(self.curr_token, Token::RBrace) {
                self.next_token();
            }
        }

        Ok(StatementNode::Block {
            token,
            statements: statements,
            implicit_return: implicit_return,
            id: 0,
        })
    }

    /// Parses an index expression.
    ///
    /// Syntax: `<array>[<index>]`
    ///
    /// # Arguments
    ///
    /// * `array` - The array expression being indexed
    ///
    /// # Returns
    ///
    /// An Index expression node containing the array and index
    fn parse_index_expression(&mut self, array: ExpressionNode) -> Result<Node, String> {
        let token = self.curr_token.clone();

        self.next_token();

        let index_expr = match self.parse_expression(Precedence::Lowest)? {
            Node::ExpressionNode(expr) => expr,
            Node::StatementNode(_) => {
                return Err(String::from("Expected expression as array index"));
            }
        };

        // Expect closing ']'
        if !matches!(self.curr_token, Token::RSquare) {
            return Err(String::from("Expected ']' after array index"));
        }

        // Move past the ']'
        self.next_token();

        Ok(Node::ExpressionNode(ExpressionNode::Index {
            token,
            object: Box::new(array),
            index: Box::new(index_expr),
            id: 0,
        }))
    }

    /// Parses an infix expression.
    ///
    /// Infix expressions include:
    /// - Binary operators (`+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `=`)
    /// - Function calls (when `(` follows an expression)
    ///
    /// # Arguments
    ///
    /// * `left_exp` - The left-hand side expression
    ///
    /// # Returns
    ///
    /// The parsed infix expression node
    fn parse_infix(&mut self, left_exp: &Node) -> Result<Node, String> {
        if matches!(self.curr_token, Token::LSquare) {
            return self.parse_index_expression(match left_exp {
                Node::ExpressionNode(val) => val.clone(),
                Node::StatementNode(_) => {
                    return Err(String::from("Expected expression in array/hashmap index"));
                }
            });
        }
        if matches!(self.curr_token, Token::LParen) {
            return self.parse_call_expression(match left_exp {
                Node::ExpressionNode(val) => val.clone(),
                Node::StatementNode(_) => {
                    return Err(String::from("Expected expression before function call"));
                }
            });
        }

        if matches!(self.curr_token, Token::Dot) {
            let token = self.curr_token.clone();
            self.next_token();

            let field_name = match &self.curr_token {
                Token::Ident(name) => name.clone(),
                _ => {
                    return Err(format!(
                        "Expected identifier after dot, got {:?}",
                        self.curr_token
                    ));
                }
            };
            // move past identifier
            self.next_token();

            //Desugar
            return Ok(Node::ExpressionNode(ExpressionNode::Index {
                token,
                object: match left_exp {
                    Node::ExpressionNode(val) => Box::new(val.clone()),
                    Node::StatementNode(_) => {
                        return Err(String::from("Expected expression before function call"));
                    }
                },
                index: Box::new(ExpressionNode::String {
                    token: Token::Str(field_name.clone()),
                    value: field_name,
                    id: 0,
                }),
                id: 0,
            }));
        }
        let precedence = self.get_precedence(&self.curr_token);
        let curr_token = self.curr_token.clone();

        // Handle compound assignment operators by desugaring them
        // e.g., x += 5 becomes x = x + 5
        let (operator, desugar_op) = match &curr_token {
            Token::Plus => (InfixOp::Add, None),
            Token::Minus => (InfixOp::Subtract, None),
            Token::Asterisk => (InfixOp::Multiply, None),
            Token::Slash => (InfixOp::Divide, None),
            Token::Lt => (InfixOp::Lt, None),
            Token::Gt => (InfixOp::Gt, None),
            Token::Eq => (InfixOp::Eq, None),
            Token::NotEq => (InfixOp::NotEq, None),
            Token::And => (InfixOp::And, None),
            Token::Or => (InfixOp::Or, None),
            Token::Assign => (InfixOp::Assign, None),
            Token::PlusAssign => (InfixOp::Assign, Some(InfixOp::Add)),
            Token::MinusAssign => (InfixOp::Assign, Some(InfixOp::Subtract)),
            Token::AsteriskAssign => (InfixOp::Assign, Some(InfixOp::Multiply)),
            Token::SlashAssign => (InfixOp::Assign, Some(InfixOp::Divide)),
            _ => return Err(format!("Unexpected infix operator: {:?}", curr_token)),
        };
        self.next_token();
        let right_exp = self.parse_expression(precedence)?;
        match left_exp {
            Node::ExpressionNode(val) => {
                // If this is a compound assignment (+=, -=, etc.), desugar it
                // e.g., x += 5 becomes x = x + 5
                let right_node = match right_exp {
                    Node::ExpressionNode(r) => {
                        if let Some(desugar_op) = desugar_op {
                            // Create the operation: left desugar_op right
                            // e.g., x + 5
                            Box::new(ExpressionNode::Infix {
                                token: curr_token.clone(),
                                left: Box::new(val.clone()),
                                operator: desugar_op,
                                right: Box::new(r),
                                id: 0,
                            })
                        } else {
                            Box::new(r)
                        }
                    }
                    _ => {
                        return Err(String::from(
                            "Expected expression on right side of infix operator",
                        ));
                    }
                };

                return Ok(Node::ExpressionNode(ExpressionNode::Infix {
                    token: curr_token.clone(),
                    left: Box::new(val.clone()),
                    operator,
                    right: right_node,
                    id: 0,
                }));
            }
            _ => {
                return Err(String::from(
                    "Expected expression on left side of infix operator",
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNode;

    fn parse_program(input: &str) -> Result<Node, String> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    #[test]
    fn test_let_statements() {
        let input = "let x = 5; let y = 10; let foobar = 838383;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 3);

                let expected_identifiers = vec!["x", "y", "foobar"];
                for (i, stmt) in statements.iter().enumerate() {
                    match stmt.as_ref() {
                        Node::StatementNode(StatementNode::Let { name, .. }) => match name {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, expected_identifiers[i]);
                            }
                            _ => panic!("Expected identifier"),
                        },
                        _ => panic!("Expected Let statement"),
                    }
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_return_statements() {
        // Return statements need to be inside a function
        let input = "func test() { return 5; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::FuncDeclr { func, .. }) => match func {
                        ExpressionNode::Function { body, .. } => match body.as_ref() {
                            StatementNode::Block { statements, .. } => {
                                assert!(statements.len() > 0);
                                match statements[0].as_ref() {
                                    StatementNode::Return { .. } => {}
                                    _ => panic!("Expected Return statement"),
                                }
                            }
                            _ => panic!("Expected Block"),
                        },
                        _ => panic!("Expected Function"),
                    },
                    _ => panic!("Expected FuncDeclr statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                assert_eq!(statements[0].as_ref().to_string(), "foobar");
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                assert_eq!(statements[0].as_ref().to_string(), "5");
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_boolean_expressions() {
        let input = "true; false;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 2);
                assert_eq!(statements[0].as_ref().to_string(), "true");
                assert_eq!(statements[1].as_ref().to_string(), "false");
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_prefix_expressions() {
        let tests = vec![
            ("!5;", "(!5)"),
            ("-15;", "(-15)"),
            ("!true;", "(!true)"),
            ("!false;", "(!false)"),
        ];

        for (input, expected) in tests {
            let program = parse_program(input).expect("parse_program failed");
            match program {
                Node::StatementNode(StatementNode::Program { statements, .. }) => {
                    assert_eq!(statements.len(), 1);
                    assert_eq!(statements[0].as_ref().to_string(), expected);
                }
                _ => panic!("Expected Program node"),
            }
        }
    }

    #[test]
    fn test_infix_expressions() {
        let tests = vec![
            ("5 + 5;", "(5 + 5)"),
            ("5 - 5;", "(5 - 5)"),
            ("5 * 5;", "(5 * 5)"),
            ("5 / 5;", "(5 / 5)"),
            ("5 > 5;", "(5 > 5)"),
            ("5 < 5;", "(5 < 5)"),
            ("5 == 5;", "(5 == 5)"),
            ("5 != 5;", "(5 != 5)"),
        ];

        for (input, expected) in tests {
            let program = parse_program(input).expect("parse_program failed");
            match program {
                Node::StatementNode(StatementNode::Program { statements, .. }) => {
                    assert_eq!(statements.len(), 1);
                    assert_eq!(statements[0].as_ref().to_string(), expected);
                }
                _ => panic!("Expected Program node"),
            }
        }
    }

    #[test]
    fn test_operator_precedence() {
        let tests = vec![
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            // Note: "3 + 4; -5 * 5" parses as two separate statements
            // The second statement "-5 * 5" is parsed correctly as "((-5) * 5)"
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
        ];

        for (input, expected) in tests {
            let program =
                parse_program(input).expect(&format!("parse_program failed for: {}", input));
            assert_eq!(
                AstNode::to_string(&program),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 0, "Should have no statements");
                assert!(implicit_return.is_some(), "Should have implicit return");
                assert_eq!(
                    Node::ExpressionNode(implicit_return.unwrap()).to_string(),
                    "if (x < y) { x }"
                );
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 0, "Should have no statements");
                assert!(implicit_return.is_some(), "Should have implicit return");
                // Note: The current implementation doesn't include else in to_string
                // This test validates the structure parses correctly
                match implicit_return.unwrap() {
                    ExpressionNode::If { else_block, .. } => {
                        assert!(else_block.is_some());
                    }
                    _ => panic!("Expected If expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_function_literal() {
        // Function literals must be in an expression context (e.g., assignment)
        let input = "let f = func(x, y) { x + y; };";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::Let { value, .. }) => match value {
                        ExpressionNode::Function { parameters, .. } => {
                            assert_eq!(parameters.len(), 2);
                        }
                        _ => panic!("Expected Function expression in let value"),
                    },
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_function_parameters() {
        let tests = vec![
            ("let f = func() {};", 0),
            ("let f = func(x) {};", 1),
            ("let f = func(x, y, z) {};", 3),
        ];

        for (input, expected_params) in tests {
            let program = parse_program(input).expect("parse_program failed");
            match program {
                Node::StatementNode(StatementNode::Program { statements, .. }) => {
                    match statements[0].as_ref() {
                        Node::StatementNode(StatementNode::Let { value, .. }) => match value {
                            ExpressionNode::Function { parameters, .. } => {
                                assert_eq!(parameters.len(), expected_params);
                            }
                            _ => panic!("Expected Function expression"),
                        },
                        _ => panic!("Expected Let statement"),
                    }
                }
                _ => panic!("Expected Program node"),
            }
        }
    }

    #[test]
    fn test_call_expression() {
        let input = "add(1, 2 * 3, 4 + 5);";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 1);
                assert!(
                    implicit_return.is_none(),
                    "Should have no implicit return with semicolon"
                );
                match statements[0].as_ref() {
                    Node::ExpressionNode(expr) => {
                        assert_eq!(
                            Node::ExpressionNode(expr.clone()).to_string(),
                            "add(1, (2 * 3), (4 + 5))"
                        );
                    }
                    _ => panic!("Expected Expression node"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_function_declaration() {
        let input = "func add(x, y) { return x + y; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::FuncDeclr { identifier, .. }) => {
                        match identifier {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, "add");
                            }
                            _ => panic!("Expected identifier"),
                        }
                    }
                    _ => panic!("Expected FuncDeclr statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_for_statement() {
        let input = "for (true) { x; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::For { condition, .. }) => match condition {
                        ExpressionNode::Boolean { value, .. } => {
                            assert_eq!(*value, true);
                        }
                        _ => panic!("Expected boolean condition"),
                    },
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_complex_program() {
        let input = "func add(x,y){return x + y}; let y = add(1,2); let z = add(2,y);";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 3);
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_let_statement_error_no_identifier() {
        let input = "let = 5;";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected identifier"));
    }

    #[test]
    fn test_let_statement_error_no_assign() {
        let input = "let x 5;";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected '='"));
    }

    #[test]
    fn test_parenthesized_expression() {
        let tests = vec![
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
        ];

        for (input, expected) in tests {
            let program = parse_program(input).expect("parse_program failed");
            assert_eq!(AstNode::to_string(&program), expected);
        }
    }

    #[test]
    fn test_assignment_as_expression() {
        // Test that x = 1 is parsed as an infix expression (not a let statement)
        let input = "x = 1;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 1);
                assert!(
                    implicit_return.is_none(),
                    "Should have no implicit return with semicolon"
                );
                // Should be an expression node with infix operator
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Infix { operator, .. }) => {
                        assert_eq!(*operator, InfixOp::Assign);
                    }
                    _ => panic!("Expected Infix expression with Assign operator"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_assignment_precedence() {
        // Test that assignment has lower precedence than arithmetic
        // y = x + 1 should parse as y = (x + 1), not (y = x) + 1
        let input = "y = x + 1;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 1);
                assert!(
                    implicit_return.is_none(),
                    "Should have no implicit return with semicolon"
                );
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Infix {
                        operator,
                        left,
                        right,
                        ..
                    }) => {
                        // Outer operation should be assignment
                        assert_eq!(*operator, InfixOp::Assign);
                        // Left should be identifier y
                        match left.as_ref() {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, "y");
                            }
                            _ => panic!("Expected identifier on left"),
                        }
                        // Right should be x + 1 (an infix add expression)
                        match right.as_ref() {
                            ExpressionNode::Infix {
                                operator: InfixOp::Add,
                                ..
                            } => {}
                            _ => panic!("Expected Add expression on right"),
                        }
                    }
                    _ => panic!("Expected Infix expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_chained_assignment_as_expression() {
        // Test that y = x = 1 is parsed as left-associative: (y = x) = 1
        // Note: Typically assignment should be right-associative, but the current
        // implementation parses it left-associative
        let input = "y = x = 1;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Infix { operator, left, .. }) => {
                        assert_eq!(*operator, InfixOp::Assign);
                        // Left side should be an assignment expression (y = x)
                        match left.as_ref() {
                            ExpressionNode::Infix {
                                operator: InfixOp::Assign,
                                ..
                            } => {}
                            _ => panic!("Expected nested assignment on left (left-associative)"),
                        }
                    }
                    _ => panic!("Expected Infix expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_all_operators_are_left_associative() {
        // All operators are left-associative because parse_infix passes
        // the same precedence when parsing the right side
        let tests = vec![
            ("1 + 2 + 3", "((1 + 2) + 3)"),     // Addition
            ("8 - 3 - 2", "((8 - 3) - 2)"),     // Subtraction
            ("2 * 3 * 4", "((2 * 3) * 4)"),     // Multiplication
            ("24 / 4 / 2", "((24 / 4) / 2)"),   // Division
            ("1 == 2 == 3", "((1 == 2) == 3)"), // Equality
        ];

        for (input, expected) in tests {
            let program = parse_program(input).expect("parse_program failed");
            assert_eq!(
                AstNode::to_string(&program),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_if_expression_error_no_lparen() {
        let input = "if x < y { x }";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected '(' after 'if'"));
    }

    #[test]
    fn test_if_expression_error_no_rparen() {
        let input = "if (x < y { x }";
        let result = parse_program(input);
        assert!(result.is_err());
        // The parenthesized expression parser catches the missing rparen
        assert!(result.unwrap_err().contains("Expected ')' after"));
    }

    #[test]
    fn test_if_expression_error_no_lbrace() {
        let input = "if (x < y) x }";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected '{' to start if block")
        );
    }

    #[test]
    fn test_for_statement_error_no_lparen() {
        let input = "for true { x; }";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected '(' after 'for'"));
    }

    #[test]
    fn test_for_statement_error_missing_rparen_in_condition() {
        let input = "for (true { x; }";
        let result = parse_program(input);
        assert!(result.is_err());
        // The parenthesized expression parser catches this
        assert!(
            result
                .unwrap_err()
                .contains("Expected ')' after expression")
        );
    }

    #[test]
    fn test_for_statement_error_no_lbrace() {
        let input = "for (true) x; }";
        let result = parse_program(input);
        assert!(result.is_err());
        // Should get error about missing block
        assert!(
            result
                .unwrap_err()
                .contains("Expected '{' to start for loop body")
        );
    }

    #[test]
    fn test_function_declaration_error_no_identifier() {
        let input = "func (x, y) { return x + y; }";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected identifier after 'func'")
        );
    }

    #[test]
    fn test_function_declaration_error_no_lbrace() {
        let input = "func add(x, y) return x + y; }";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected '{' to start function body")
        );
    }

    #[test]
    fn test_function_literal_error_no_lparen() {
        let input = "let f = func x, y) { x + y; };";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected '(' after 'func'"));
    }

    #[test]
    fn test_function_literal_error_no_lbrace() {
        let input = "let f = func(x, y) return x + y; };";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected '{' to start function body")
        );
    }

    #[test]
    fn test_function_parameters_error_no_rparen() {
        let input = "let f = func(x, y { x + y; };";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected ')' after function parameters")
        );
    }

    #[test]
    fn test_call_arguments_error_no_rparen() {
        let input = "add(1, 2;";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected ')' after function arguments")
        );
    }

    #[test]
    fn test_parenthesized_expression_error_no_rparen() {
        let input = "(5 + 5;";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected ')' after expression")
        );
    }

    #[test]
    fn test_prefix_expression_error_unexpected_token() {
        let input = "; 5";
        let result = parse_program(input);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        eprintln!("Actual error: '{}'", error_msg);
        eprintln!("Contains check: {}", error_msg.contains("Unexpected token"));
        eprintln!(
            "Contains check 2: {}",
            error_msg.contains("prefix position")
        );
        eprintln!(
            "Contains full: {}",
            error_msg.contains("Unexpected token in prefix position")
        );
        assert!(error_msg.contains("Unexpected token"));
        assert!(error_msg.contains("prefix position"));
    }

    #[test]
    fn test_else_block_error_no_lbrace() {
        let input = "if (x < y) { x } else y }";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected '{' to start else block")
        );
    }

    #[test]
    fn test_program_implicit_return_single_expression() {
        let input = "5";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 0, "Should have no statements");
                assert!(implicit_return.is_some(), "Should have implicit return");
                match implicit_return.unwrap() {
                    ExpressionNode::Integer { value, .. } => {
                        assert_eq!(value, 5);
                    }
                    _ => panic!("Expected integer expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_program_no_implicit_return_with_semicolon() {
        let input = "5;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 1, "Should have one statement");
                assert!(implicit_return.is_none(), "Should have no implicit return");
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_program_implicit_return_after_let() {
        let input = "let x = 5; x";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 1, "Should have one let statement");
                assert!(implicit_return.is_some(), "Should have implicit return");
                match implicit_return.unwrap() {
                    ExpressionNode::Identifier { value, .. } => {
                        assert_eq!(value, "x");
                    }
                    _ => panic!("Expected identifier expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_program_no_implicit_return_after_let_with_semicolon() {
        let input = "let x = 5; x;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 2, "Should have two statements");
                assert!(implicit_return.is_none(), "Should have no implicit return");
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_program_implicit_return_complex_expression() {
        let input = "let x = 5; let y = 10; x + y";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 2, "Should have two let statements");
                assert!(implicit_return.is_some(), "Should have implicit return");
                match implicit_return.unwrap() {
                    ExpressionNode::Infix { operator, .. } => {
                        assert_eq!(operator, InfixOp::Add);
                    }
                    _ => panic!("Expected infix expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_program_empty() {
        let input = "";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 0, "Should have no statements");
                assert!(implicit_return.is_none(), "Should have no implicit return");
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_program_multiple_expressions_without_semicolon() {
        // Only the last expression without semicolon should be implicit return
        let input = "5\n10";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(
                    statements.len(),
                    1,
                    "First expression should become statement"
                );
                assert!(
                    implicit_return.is_some(),
                    "Last expression should be implicit return"
                );
                match implicit_return.unwrap() {
                    ExpressionNode::Integer { value, .. } => {
                        assert_eq!(value, 10, "Should return the last expression");
                    }
                    _ => panic!("Expected integer expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_empty() {
        let input = "[];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Array { elements, .. }) => {
                        assert_eq!(elements.len(), 0, "Empty array should have no elements");
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_single_element() {
        let input = "[1];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Array { elements, .. }) => {
                        assert_eq!(elements.len(), 1, "Array should have one element");
                        match elements[0].as_ref() {
                            ExpressionNode::Integer { value, .. } => {
                                assert_eq!(*value, 1);
                            }
                            _ => panic!("Expected integer element"),
                        }
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_multiple_elements() {
        let input = "[1, 2, 3];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Array { elements, .. }) => {
                        assert_eq!(elements.len(), 3, "Array should have three elements");
                        let expected_values = vec![1, 2, 3];
                        for (i, element) in elements.iter().enumerate() {
                            match element.as_ref() {
                                ExpressionNode::Integer { value, .. } => {
                                    assert_eq!(*value, expected_values[i]);
                                }
                                _ => panic!("Expected integer element at index {}", i),
                            }
                        }
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_mixed_types() {
        let input = "[1, true, \"hello\"];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Array { elements, .. }) => {
                        assert_eq!(elements.len(), 3, "Array should have three elements");

                        // Check first element (integer)
                        match elements[0].as_ref() {
                            ExpressionNode::Integer { value, .. } => {
                                assert_eq!(*value, 1);
                            }
                            _ => panic!("Expected integer at index 0"),
                        }

                        // Check second element (boolean)
                        match elements[1].as_ref() {
                            ExpressionNode::Boolean { value, .. } => {
                                assert_eq!(*value, true);
                            }
                            _ => panic!("Expected boolean at index 1"),
                        }

                        // Check third element (string)
                        match elements[2].as_ref() {
                            ExpressionNode::String { value, .. } => {
                                assert_eq!(value, "hello");
                            }
                            _ => panic!("Expected string at index 2"),
                        }
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_with_expressions() {
        let input = "[1 + 1, 2 * 3, x];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Array { elements, .. }) => {
                        assert_eq!(elements.len(), 3, "Array should have three elements");

                        // First element should be an infix expression (1 + 1)
                        match elements[0].as_ref() {
                            ExpressionNode::Infix {
                                operator: InfixOp::Add,
                                ..
                            } => {}
                            _ => panic!("Expected Add infix expression at index 0"),
                        }

                        // Second element should be an infix expression (2 * 3)
                        match elements[1].as_ref() {
                            ExpressionNode::Infix {
                                operator: InfixOp::Multiply,
                                ..
                            } => {}
                            _ => panic!("Expected Multiply infix expression at index 1"),
                        }

                        // Third element should be an identifier
                        match elements[2].as_ref() {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, "x");
                            }
                            _ => panic!("Expected identifier at index 2"),
                        }
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_nested() {
        let input = "[[1, 2], [3, 4]];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Array { elements, .. }) => {
                        assert_eq!(elements.len(), 2, "Outer array should have two elements");

                        // Check both nested arrays
                        for (i, element) in elements.iter().enumerate() {
                            match element.as_ref() {
                                ExpressionNode::Array {
                                    elements: inner_elements,
                                    ..
                                } => {
                                    assert_eq!(
                                        inner_elements.len(),
                                        2,
                                        "Inner array {} should have two elements",
                                        i
                                    );
                                }
                                _ => panic!("Expected Array at index {}", i),
                            }
                        }
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_in_let_statement() {
        let input = "let arr = [1, 2, 3];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::Let { name, value, .. }) => {
                        // Check the variable name
                        match name {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, "arr");
                            }
                            _ => panic!("Expected identifier"),
                        }

                        // Check the value is an array
                        match value {
                            ExpressionNode::Array { elements, .. } => {
                                assert_eq!(elements.len(), 3);
                            }
                            _ => panic!("Expected Array expression in let value"),
                        }
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_array_literal_to_string() {
        let tests = vec![
            ("[]", "[]"),
            ("[1]", "[1]"),
            ("[1, 2, 3]", "[1, 2, 3]"),
            ("[1, true, \"hello\"]", "[1, true, \"hello\"]"),
            ("[[1, 2], [3, 4]]", "[[1, 2], [3, 4]]"),
        ];

        for (input, expected) in tests {
            let program =
                parse_program(input).expect(&format!("parse_program failed for: {}", input));
            assert_eq!(
                AstNode::to_string(&program),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_array_literal_error_missing_closing_bracket() {
        let input = "[1, 2, 3;";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected ']' after array elements")
        );
    }

    #[test]
    fn test_array_literal_error_statement_in_array() {
        // This should fail because you can't have statements inside arrays
        let input = "[let x = 5];";
        let result = parse_program(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_array_as_implicit_return() {
        let input = "[1, 2, 3]";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program {
                statements,
                implicit_return,
                ..
            }) => {
                assert_eq!(statements.len(), 0, "Should have no statements");
                assert!(implicit_return.is_some(), "Should have implicit return");
                match implicit_return.unwrap() {
                    ExpressionNode::Array { elements, .. } => {
                        assert_eq!(elements.len(), 3);
                    }
                    _ => panic!("Expected Array expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    // ===== Array Indexing Tests =====
    //
    // These tests verify that the parser correctly handles array indexing expressions.
    // Array indexing follows the syntax: `array[index]`
    //
    // Test Coverage:
    // - Simple indexing with literals: arr[0]
    // - Indexing with expressions: arr[i + 1]
    // - Nested/chained indexing: arr[0][1]
    // - Index assignment: arr[0] = 42
    // - Index expressions in other contexts (let statements, arithmetic, etc.)
    // - Error cases: missing brackets, invalid syntax

    #[test]
    fn test_parse_array_index_simple() {
        let input = "arr[0];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Index { object, index, .. }) => {
                        // Check object is identifier 'arr'
                        match object.as_ref() {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, "arr");
                            }
                            _ => panic!("Expected identifier for array object"),
                        }
                        // Check index is integer 0
                        match index.as_ref() {
                            ExpressionNode::Integer { value, .. } => {
                                assert_eq!(*value, 0);
                            }
                            _ => panic!("Expected integer for array index"),
                        }
                    }
                    _ => panic!("Expected Index expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_array_index_with_expression() {
        let input = "arr[i + 1];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Index { object, index, .. }) => {
                        // Check object is identifier 'arr'
                        match object.as_ref() {
                            ExpressionNode::Identifier { value, .. } => {
                                assert_eq!(value, "arr");
                            }
                            _ => panic!("Expected identifier"),
                        }
                        // Check index is an infix expression (i + 1)
                        match index.as_ref() {
                            ExpressionNode::Infix {
                                operator: InfixOp::Add,
                                ..
                            } => {}
                            _ => panic!("Expected Add infix expression for index"),
                        }
                    }
                    _ => panic!("Expected Index expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_nested_array_index() {
        let input = "arr[0][1];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Index {
                        object: outer_object,
                        index: outer_index,
                        ..
                    }) => {
                        // Outer object should be an Index expression (arr[0])
                        match outer_object.as_ref() {
                            ExpressionNode::Index { object, index, .. } => {
                                // Inner object should be identifier 'arr'
                                match object.as_ref() {
                                    ExpressionNode::Identifier { value, .. } => {
                                        assert_eq!(value, "arr");
                                    }
                                    _ => panic!("Expected identifier in nested index"),
                                }
                                // Inner index should be 0
                                match index.as_ref() {
                                    ExpressionNode::Integer { value, .. } => {
                                        assert_eq!(*value, 0);
                                    }
                                    _ => panic!("Expected integer for first index"),
                                }
                            }
                            _ => panic!("Expected Index expression for outer object"),
                        }
                        // Outer index should be 1
                        match outer_index.as_ref() {
                            ExpressionNode::Integer { value, .. } => {
                                assert_eq!(*value, 1);
                            }
                            _ => panic!("Expected integer for second index"),
                        }
                    }
                    _ => panic!("Expected Index expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_array_index_assignment() {
        let input = "arr[0] = 42;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Infix {
                        left,
                        operator,
                        right,
                        ..
                    }) => {
                        // Should be assignment operator
                        assert_eq!(*operator, InfixOp::Assign);

                        // Left side should be Index expression
                        match left.as_ref() {
                            ExpressionNode::Index { object, index, .. } => {
                                match object.as_ref() {
                                    ExpressionNode::Identifier { value, .. } => {
                                        assert_eq!(value, "arr");
                                    }
                                    _ => panic!("Expected identifier"),
                                }
                                match index.as_ref() {
                                    ExpressionNode::Integer { value, .. } => {
                                        assert_eq!(*value, 0);
                                    }
                                    _ => panic!("Expected integer index"),
                                }
                            }
                            _ => panic!("Expected Index expression on left of assignment"),
                        }

                        // Right side should be integer 42
                        match right.as_ref() {
                            ExpressionNode::Integer { value, .. } => {
                                assert_eq!(*value, 42);
                            }
                            _ => panic!("Expected integer on right of assignment"),
                        }
                    }
                    _ => panic!("Expected Infix expression for assignment"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_array_index_in_expression() {
        let input = "arr[0] + 10;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Infix {
                        left,
                        operator,
                        right,
                        ..
                    }) => {
                        assert_eq!(*operator, InfixOp::Add);

                        // Left should be Index expression
                        match left.as_ref() {
                            ExpressionNode::Index { .. } => {}
                            _ => panic!("Expected Index expression"),
                        }

                        // Right should be integer
                        match right.as_ref() {
                            ExpressionNode::Integer { value, .. } => {
                                assert_eq!(*value, 10);
                            }
                            _ => panic!("Expected integer"),
                        }
                    }
                    _ => panic!("Expected Infix expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_array_index_to_string() {
        let tests = vec![
            ("arr[0]", "arr[0]"),
            ("arr[i + 1]", "arr[(i + 1)]"),
            ("arr[0][1]", "arr[0][1]"),
        ];

        for (input, expected) in tests {
            let program =
                parse_program(input).expect(&format!("parse_program failed for: {}", input));
            assert_eq!(
                AstNode::to_string(&program),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_array_index_error_missing_closing_bracket() {
        let input = "arr[0;";
        let result = parse_program(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Expected ']' after array index")
        );
    }

    #[test]
    fn test_parse_array_index_in_let_statement() {
        let input = "let x = arr[0];";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::Let { value, .. }) => {
                        // Value should be an Index expression
                        match value {
                            ExpressionNode::Index { object, index, .. } => {
                                match object.as_ref() {
                                    ExpressionNode::Identifier { value, .. } => {
                                        assert_eq!(value, "arr");
                                    }
                                    _ => panic!("Expected identifier"),
                                }
                                match index.as_ref() {
                                    ExpressionNode::Integer { value, .. } => {
                                        assert_eq!(*value, 0);
                                    }
                                    _ => panic!("Expected integer"),
                                }
                            }
                            _ => panic!("Expected Index expression in let value"),
                        }
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_array_index_chained_assignment() {
        let input = "arr[0] = arr[1] + 1;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(ExpressionNode::Infix {
                        left,
                        operator,
                        right,
                        ..
                    }) => {
                        assert_eq!(*operator, InfixOp::Assign);

                        // Left should be Index expression
                        match left.as_ref() {
                            ExpressionNode::Index { .. } => {}
                            _ => panic!("Expected Index on left"),
                        }

                        // Right should be Infix (arr[1] + 1)
                        match right.as_ref() {
                            ExpressionNode::Infix {
                                left: inner_left,
                                operator: InfixOp::Add,
                                ..
                            } => {
                                // Inner left should be Index expression
                                match inner_left.as_ref() {
                                    ExpressionNode::Index { .. } => {}
                                    _ => panic!("Expected Index in right side expression"),
                                }
                            }
                            _ => panic!("Expected Infix on right"),
                        }
                    }
                    _ => panic!("Expected Infix expression"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_break_statement() {
        let input = "for (true) { break; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);

                // Check that we have a For statement
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::For { for_block, .. }) => {
                        // Check the for block contains a break statement
                        match for_block.as_ref() {
                            StatementNode::Block { statements, .. } => {
                                assert_eq!(statements.len(), 1);
                                match statements[0].as_ref() {
                                    StatementNode::Break { .. } => {
                                        // Success - found the break statement
                                    }
                                    _ => panic!("Expected Break statement"),
                                }
                            }
                            _ => panic!("Expected Block statement"),
                        }
                    }
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_continue_statement() {
        let input = "for (true) { continue; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);

                // Check that we have a For statement
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::For { for_block, .. }) => {
                        // Check the for block contains a continue statement
                        match for_block.as_ref() {
                            StatementNode::Block { statements, .. } => {
                                assert_eq!(statements.len(), 1);
                                match statements[0].as_ref() {
                                    StatementNode::Continue { .. } => {
                                        // Success - found the continue statement
                                    }
                                    _ => panic!("Expected Continue statement"),
                                }
                            }
                            _ => panic!("Expected Block statement"),
                        }
                    }
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_break_with_conditional() {
        let input = "for (true) { if (x > 5) { break; }; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);

                // Check that we have a For statement with a nested if containing break
                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::For { for_block, .. }) => {
                        match for_block.as_ref() {
                            StatementNode::Block { statements, .. } => {
                                assert_eq!(statements.len(), 1);
                                // First statement should be an Expression wrapping an If
                                match statements[0].as_ref() {
                                    StatementNode::Expression { expression, .. } => {
                                        match expression {
                                            ExpressionNode::If { if_block, .. } => {
                                                match if_block.as_ref() {
                                                    StatementNode::Block { statements, .. } => {
                                                        assert_eq!(statements.len(), 1);
                                                        match statements[0].as_ref() {
                                                            StatementNode::Break { .. } => {
                                                                // Success
                                                            }
                                                            _ => {
                                                                panic!("Expected Break in if block")
                                                            }
                                                        }
                                                    }
                                                    _ => panic!("Expected Block in if"),
                                                }
                                            }
                                            _ => panic!("Expected If expression"),
                                        }
                                    }
                                    _ => panic!("Expected Expression statement"),
                                }
                            }
                            _ => panic!("Expected Block statement"),
                        }
                    }
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_continue_with_conditional() {
        let input = "for (i < 10) { if (i == 5) { continue; }; print(i); }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);

                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::For { for_block, .. }) => {
                        match for_block.as_ref() {
                            StatementNode::Block { statements, .. } => {
                                // Should have 2 statements: if and print
                                assert_eq!(statements.len(), 2);

                                // First statement: if with continue
                                match statements[0].as_ref() {
                                    StatementNode::Expression { expression, .. } => {
                                        match expression {
                                            ExpressionNode::If { if_block, .. } => {
                                                match if_block.as_ref() {
                                                    StatementNode::Block { statements, .. } => {
                                                        assert_eq!(statements.len(), 1);
                                                        match statements[0].as_ref() {
                                                            StatementNode::Continue { .. } => {
                                                                // Success
                                                            }
                                                            _ => {
                                                                panic!(
                                                                    "Expected Continue in if block"
                                                                )
                                                            }
                                                        }
                                                    }
                                                    _ => panic!("Expected Block in if"),
                                                }
                                            }
                                            _ => panic!("Expected If expression"),
                                        }
                                    }
                                    _ => panic!("Expected Expression statement"),
                                }

                                // Second statement: print call
                                match statements[1].as_ref() {
                                    StatementNode::Expression { expression, .. } => {
                                        match expression {
                                            ExpressionNode::Call { .. } => {
                                                // Success
                                            }
                                            _ => panic!("Expected Call expression"),
                                        }
                                    }
                                    _ => panic!("Expected Expression statement"),
                                }
                            }
                            _ => panic!("Expected Block statement"),
                        }
                    }
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_multiple_breaks_and_continues() {
        let input = "for (true) { if (a) { break; }; if (b) { continue; }; }";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);

                match statements[0].as_ref() {
                    Node::StatementNode(StatementNode::For { for_block, .. }) => {
                        match for_block.as_ref() {
                            StatementNode::Block { statements, .. } => {
                                // Should have 2 if statements
                                assert_eq!(statements.len(), 2);

                                // First if with break
                                match statements[0].as_ref() {
                                    StatementNode::Expression { expression, .. } => {
                                        match expression {
                                            ExpressionNode::If { if_block, .. } => {
                                                match if_block.as_ref() {
                                                    StatementNode::Block { statements, .. } => {
                                                        match statements[0].as_ref() {
                                                            StatementNode::Break { .. } => {}
                                                            _ => panic!("Expected Break"),
                                                        }
                                                    }
                                                    _ => panic!("Expected Block"),
                                                }
                                            }
                                            _ => panic!("Expected If"),
                                        }
                                    }
                                    _ => panic!("Expected Expression"),
                                }

                                // Second if with continue
                                match statements[1].as_ref() {
                                    StatementNode::Expression { expression, .. } => {
                                        match expression {
                                            ExpressionNode::If { if_block, .. } => {
                                                match if_block.as_ref() {
                                                    StatementNode::Block { statements, .. } => {
                                                        match statements[0].as_ref() {
                                                            StatementNode::Continue { .. } => {}
                                                            _ => panic!("Expected Continue"),
                                                        }
                                                    }
                                                    _ => panic!("Expected Block"),
                                                }
                                            }
                                            _ => panic!("Expected If"),
                                        }
                                    }
                                    _ => panic!("Expected Expression"),
                                }
                            }
                            _ => panic!("Expected Block"),
                        }
                    }
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_compound_assignment_desugaring() {
        // Test that x += 5 is desugared to x = (x + 5)
        let input = "x += 5;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(expression) => {
                        // Should be: x = (x + 5)
                        match expression {
                            ExpressionNode::Infix {
                                operator,
                                left,
                                right,
                                ..
                            } => {
                                // Outer operator should be Assign
                                assert_eq!(*operator, InfixOp::Assign);

                                // Left should be identifier x
                                match left.as_ref() {
                                    ExpressionNode::Identifier { value, .. } => {
                                        assert_eq!(value, "x");
                                    }
                                    _ => panic!("Expected identifier on left side"),
                                }

                                // Right should be (x + 5)
                                match right.as_ref() {
                                    ExpressionNode::Infix {
                                        operator,
                                        left,
                                        right,
                                        ..
                                    } => {
                                        assert_eq!(*operator, InfixOp::Add);

                                        // Left of inner should be x
                                        match left.as_ref() {
                                            ExpressionNode::Identifier { value, .. } => {
                                                assert_eq!(value, "x");
                                            }
                                            _ => panic!("Expected identifier"),
                                        }

                                        // Right of inner should be 5
                                        match right.as_ref() {
                                            ExpressionNode::Integer { value, .. } => {
                                                assert_eq!(*value, 5);
                                            }
                                            _ => panic!("Expected integer"),
                                        }
                                    }
                                    _ => panic!("Expected infix expression on right side"),
                                }
                            }
                            _ => panic!("Expected infix expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_all_compound_assignment_operators() {
        // Test all four compound assignment operators
        let test_cases = vec![
            ("x += 5;", InfixOp::Add),
            ("x -= 3;", InfixOp::Subtract),
            ("x *= 2;", InfixOp::Multiply),
            ("x /= 4;", InfixOp::Divide),
        ];

        for (input, expected_op) in test_cases {
            let program = parse_program(input).expect("parse_program failed");

            match program {
                Node::StatementNode(StatementNode::Program { statements, .. }) => {
                    match statements[0].as_ref() {
                        Node::ExpressionNode(expression) => {
                            match expression {
                                ExpressionNode::Infix {
                                    operator, right, ..
                                } => {
                                    assert_eq!(*operator, InfixOp::Assign);

                                    // Verify inner operation is correct
                                    match right.as_ref() {
                                        ExpressionNode::Infix { operator, .. } => {
                                            assert_eq!(*operator, expected_op);
                                        }
                                        _ => panic!("Expected infix in desugared right side"),
                                    }
                                }
                                _ => panic!("Expected infix"),
                            }
                        }
                        _ => panic!("Expected expression statement"),
                    }
                }
                _ => panic!("Expected Program node"),
            }
        }
    }

    #[test]
    fn test_dot_notation_desugaring() {
        // Test that person.name is desugared to person["name"]
        let input = "person.name;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(expression) => {
                        // Should be: person["name"] (Index expression)
                        match expression {
                            ExpressionNode::Index { object, index, .. } => {
                                // Object should be identifier "person"
                                match object.as_ref() {
                                    ExpressionNode::Identifier { value, .. } => {
                                        assert_eq!(value, "person");
                                    }
                                    _ => panic!("Expected identifier as object"),
                                }

                                // Index should be string literal "name"
                                match index.as_ref() {
                                    ExpressionNode::String { value, .. } => {
                                        assert_eq!(value, "name");
                                    }
                                    _ => panic!("Expected string literal as index"),
                                }
                            }
                            _ => panic!("Expected index expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_chained_dot_notation() {
        // Test that person.address.city is desugared correctly
        // Should become: (person["address"])["city"]
        let input = "person.address.city;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(expression) => {
                        // Outer index: ...["city"]
                        match expression {
                            ExpressionNode::Index { object, index, .. } => {
                                // Index should be "city"
                                match index.as_ref() {
                                    ExpressionNode::String { value, .. } => {
                                        assert_eq!(value, "city");
                                    }
                                    _ => panic!("Expected string literal 'city'"),
                                }

                                // Object should be person["address"]
                                match object.as_ref() {
                                    ExpressionNode::Index {
                                        object: inner_object,
                                        index: inner_index,
                                        ..
                                    } => {
                                        // Inner object should be "person"
                                        match inner_object.as_ref() {
                                            ExpressionNode::Identifier { value, .. } => {
                                                assert_eq!(value, "person");
                                            }
                                            _ => panic!("Expected identifier 'person'"),
                                        }

                                        // Inner index should be "address"
                                        match inner_index.as_ref() {
                                            ExpressionNode::String { value, .. } => {
                                                assert_eq!(value, "address");
                                            }
                                            _ => panic!("Expected string literal 'address'"),
                                        }
                                    }
                                    _ => panic!("Expected index expression for person.address"),
                                }
                            }
                            _ => panic!("Expected index expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_dot_notation_with_method_call() {
        // Test that person.getName() works
        // Should become: person["getName"]()
        let input = "person.getName();";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(expression) => {
                        // Outer should be a Call expression
                        match expression {
                            ExpressionNode::Call {
                                function,
                                arguments,
                                ..
                            } => {
                                // Args should be empty
                                assert_eq!(arguments.len(), 0);

                                // Function should be person["getName"]
                                match function.as_ref() {
                                    ExpressionNode::Index { object, index, .. } => {
                                        match object.as_ref() {
                                            ExpressionNode::Identifier { value, .. } => {
                                                assert_eq!(value, "person");
                                            }
                                            _ => panic!("Expected identifier 'person'"),
                                        }

                                        match index.as_ref() {
                                            ExpressionNode::String { value, .. } => {
                                                assert_eq!(value, "getName");
                                            }
                                            _ => panic!("Expected string literal 'getName'"),
                                        }
                                    }
                                    _ => panic!("Expected index expression"),
                                }
                            }
                            _ => panic!("Expected call expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_dot_notation_with_array_index() {
        // Test that users[0].name works
        // Should become: (users[0])["name"]
        let input = "users[0].name;";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(expression) => {
                        // Outer should be Index for .name
                        match expression {
                            ExpressionNode::Index { object, index, .. } => {
                                // Index should be "name"
                                match index.as_ref() {
                                    ExpressionNode::String { value, .. } => {
                                        assert_eq!(value, "name");
                                    }
                                    _ => panic!("Expected string literal 'name'"),
                                }

                                // Object should be users[0]
                                match object.as_ref() {
                                    ExpressionNode::Index {
                                        object: inner_object,
                                        index: inner_index,
                                        ..
                                    } => {
                                        match inner_object.as_ref() {
                                            ExpressionNode::Identifier { value, .. } => {
                                                assert_eq!(value, "users");
                                            }
                                            _ => panic!("Expected identifier 'users'"),
                                        }

                                        match inner_index.as_ref() {
                                            ExpressionNode::Integer { value, .. } => {
                                                assert_eq!(*value, 0);
                                            }
                                            _ => panic!("Expected integer 0"),
                                        }
                                    }
                                    _ => panic!("Expected index expression for users[0]"),
                                }
                            }
                            _ => panic!("Expected index expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_dot_notation_assignment() {
        // Test that person.name = "John" works
        // Should become: person["name"] = "John"
        let input = "person.name = \"John\";";
        let program = parse_program(input).expect("parse_program failed");

        match program {
            Node::StatementNode(StatementNode::Program { statements, .. }) => {
                assert_eq!(statements.len(), 1);
                match statements[0].as_ref() {
                    Node::ExpressionNode(expression) => {
                        // Should be an assignment
                        match expression {
                            ExpressionNode::Infix {
                                operator,
                                left,
                                right,
                                ..
                            } => {
                                assert_eq!(*operator, InfixOp::Assign);

                                // Left should be person["name"]
                                match left.as_ref() {
                                    ExpressionNode::Index { object, index, .. } => {
                                        match object.as_ref() {
                                            ExpressionNode::Identifier { value, .. } => {
                                                assert_eq!(value, "person");
                                            }
                                            _ => panic!("Expected identifier 'person'"),
                                        }

                                        match index.as_ref() {
                                            ExpressionNode::String { value, .. } => {
                                                assert_eq!(value, "name");
                                            }
                                            _ => panic!("Expected string literal 'name'"),
                                        }
                                    }
                                    _ => panic!("Expected index expression on left"),
                                }

                                // Right should be "John"
                                match right.as_ref() {
                                    ExpressionNode::String { value, .. } => {
                                        assert_eq!(value, "John");
                                    }
                                    _ => panic!("Expected string literal 'John'"),
                                }
                            }
                            _ => panic!("Expected infix expression"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }
}
