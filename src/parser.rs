// This is the reimplementation of the parser from before, but with cleaner code
// reason being that my use of next token is random and unclean and the abilityh
// to support infix, prefix and postfix is not great
// Base the parser on https://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Precedence {
    Lowest = 1,
    TERNARY,
    ASSIGN,      // =
    LOGICAL,     // && ||
    EQUALS,      // ==
    LESSGREATER, // < >
    SUM,         // + -
    PRODUCT,     // * /
    PREFIX,      // -x !x
    POSTFIX,
    CALL, // function()
}

pub struct Parser {
    pub l: Lexer,
    pub curr_token: Token,
    pub peek_token: Token,
    /// Counter for assigning unique node IDs
    next_node_id: i32,
}

impl Parser {
    pub fn new(l: Lexer) -> Self {
        let mut p = Parser {
            l,
            curr_token: Token::Eof,
            peek_token: Token::Eof,
            next_node_id: 0,
        };
        p.consume_token();
        p.consume_token();
        p
    }
    // Consumes current token and advances to the next token
    pub fn consume_token(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn consume_id(&mut self) -> i32 {
        let curr_id = self.next_node_id;
        self.next_node_id += 1;
        curr_id
    }

    pub fn parse_program(&mut self) -> Result<Node, String> {
        // parsing a program is just like parsing a block but without the enclosing "{"
        let mut statements = vec![];
        while !matches!(self.curr_token, Token::Eof) {
            // todo: figure out if its better to move past the semi colon here?
            // to do it in the outer function
            statements.push(Box::new(Node::StatementNode(self.parse_statement(true)?)));
        }
        // todo: figure out the implicit return
        Ok(Node::StatementNode(StatementNode::Program {
            statements: statements,
            implicit_return: None,
            id: self.consume_id(),
        }))
    }

    pub fn parse_expression(&mut self, p: Precedence) -> Result<ExpressionNode, String> {
        // a * b + c -> this should be (a * b) + c
        // a + b * c  -> this has precednece on the right
        // basically the idea behind the pratt parser is this
        // given a predecence, the parse_expression takes in how far it should
        // continue grouping to the right
        // so for example a + b * c, when parseInfix is called, it calls
        // parse expression on b * c with precedence of +, because
        // of this the pratt parser groups b and c together
        // whereas if its a * b + c, when observing that b + c has a lower
        // precedence than *, it
        // paranthesis has higher precedence than * than +
        // parse the prefix
        // prefix for "(" handles groupings like a * (b + c)
        // while the infix handles function calls like a(b)
        let mut left_exp = self.parse_prefix()?;
        // todo: Handle comma-separated expressions (used in function parameters/arguments)
        while !matches!(self.curr_token, Token::Semicolon)
            && p < self.get_precedence(&self.curr_token)
        {
            if matches!(self.curr_token, Token::Conditional) {
                left_exp = self.parse_mixfix(&left_exp)?;
            } else {
                left_exp = self.parse_infix(&left_exp)?;
            }
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
            Token::Conditional => Precedence::TERNARY,
            _ => Precedence::Lowest,
        }
    }

    pub fn parse_statement(&mut self, consume_semicolon: bool) -> Result<StatementNode, String> {
        let stmt = match self.curr_token {
            Token::Let => {
                let let_token = self.curr_token.clone();

                // move to identifier
                self.consume_token();
                let identifier = if let Token::Ident(val) = self.curr_token.clone() {
                    ExpressionNode::Identifier {
                        token: self.curr_token.clone(),
                        value: val,
                        id: self.consume_id(),
                    }
                } else {
                    return Err(String::from(
                        "let statement does not have identifier after let",
                    ));
                };
                self.consume_token();
                // Ensure next token is assignment operator
                if !matches!(self.curr_token, Token::Assign) {
                    return Err(String::from(
                        "Expected '=' after identifier in let statement",
                    ));
                }
                self.consume_token();
                StatementNode::Let {
                    token: let_token,
                    value: self.parse_expression(Precedence::Lowest)?,
                    name: identifier,
                    id: self.consume_id(),
                }
            }
            Token::Return => {
                // Todo: Verify that returns are only
                // in function body, does not make sense
                // for this to be in the main scope
                self.consume_token();
                StatementNode::Return {
                    token: Token::Return,
                    return_value: self.parse_expression(Precedence::Lowest)?,
                    id: self.consume_id(),
                }
            }
            Token::Break => {
                self.consume_token();
                StatementNode::Break {
                    token: Token::Break,
                    id: self.consume_id(),
                }
            }
            Token::Continue => {
                self.consume_token();
                StatementNode::Continue {
                    token: Token::Continue,
                    id: self.consume_id(),
                }
            }
            Token::For => {
                self.consume_token();
                if !matches!(self.curr_token, Token::LParen) {
                    return Err(String::from("Expected '(' after 'for' keyword"));
                }
                self.consume_token();
                let condition = self.parse_expression(Precedence::Lowest)?;
                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after 'for' keyword"));
                }
                self.consume_token();
                StatementNode::For {
                    token: Token::For,
                    condition: condition,
                    for_block: Box::new(self.parse_statement(false)?),
                    id: self.consume_id(),
                }
            }
            Token::Function => {
                self.consume_token();
                let identifier = if let Token::Ident(identifier) = self.curr_token.clone() {
                    self.consume_token();
                    ExpressionNode::Identifier {
                        token: Token::Ident(identifier.clone()),
                        value: identifier,
                        id: self.consume_id(),
                    }
                } else {
                    return Err(String::from("Expected identifier after 'func' keyword"));
                };

                if !matches!(self.curr_token, Token::LParen) {
                    return Err(String::from("Expected '(' after 'func' keyword"));
                }

                // consume the (
                self.consume_token();
                let mut param_identifiers = vec![];
                while !matches!(self.curr_token, Token::RParen) {
                    let identifier = Box::new(self.parse_expression(Precedence::Lowest)?);
                    param_identifiers.push(identifier);
                    // End of attributes
                    if matches!(self.curr_token, Token::RParen) {
                        break;
                    }
                    // if its not comma sperated, return error
                    if !matches!(self.curr_token, Token::Comma) {
                        return Err(String::from("Expected comma in object literal"));
                    }
                    self.consume_token();
                }

                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after function param expression"));
                }
                self.consume_token();

                if !matches!(self.curr_token, Token::LBrace) {
                    return Err(String::from("Expected '{' before 'func' body"));
                }
                let func = ExpressionNode::Function {
                    token: Token::Function,
                    parameters: param_identifiers,
                    // todo: finish this once blocks are done
                    body: Box::new(self.parse_statement(false)?),
                    id: self.consume_id(),
                };
                StatementNode::FuncDeclr {
                    token: Token::Function,
                    identifier: identifier,
                    func,
                    id: self.consume_id(),
                }
            }
            Token::LBrace => {
                // Parsing of blocks using the {}
                self.consume_token();

                let mut statements = vec![];
                while !(matches!(self.curr_token, Token::RBrace)
                    || matches!(self.curr_token, Token::Eof))
                {
                    // todo: figure out if its better to move past the semi colon here?
                    // to do it in the outer function
                    statements.push(Box::new(self.parse_statement(true)?));
                }
                self.consume_token();
                // todo: figure out the implicit return
                StatementNode::Block {
                    token: Token::LBrace,
                    statements: statements,
                    implicit_return: None,
                    id: self.consume_id(),
                }
            }
            _ => StatementNode::Expression {
                token: self.curr_token.clone(),
                expression: self.parse_expression(Precedence::Lowest)?,
                id: self.consume_id(),
            },
        };
        if consume_semicolon {
            if !matches!(self.curr_token, Token::Semicolon) {
                return Err(String::from("; expected at the end of a statement"));
            }
            self.consume_token();
        }
        Ok(stmt)
    }

    pub fn parse_prefix(&mut self) -> Result<ExpressionNode, String> {
        let curr_token = self.curr_token.clone();
        match curr_token.clone() {
            Token::Ident(val) => {
                self.consume_token();
                return Ok(ExpressionNode::Identifier {
                    token: curr_token,
                    value: val,
                    id: self.consume_id(),
                });
            }
            Token::Int(val) => {
                let int_value: i64 = match val.parse() {
                    Ok(num) => num,
                    Err(e) => return Err(e.to_string()),
                };
                self.consume_token();
                return Ok(ExpressionNode::Integer {
                    token: curr_token.clone(),
                    value: int_value,
                    id: self.consume_id(),
                });
            }
            Token::Str(val) => {
                self.consume_token();
                return Ok(ExpressionNode::String {
                    token: curr_token.clone(),
                    value: val.clone(),
                    id: self.consume_id(),
                });
            }
            Token::True => {
                self.consume_token();
                return Ok(ExpressionNode::Boolean {
                    token: curr_token.clone(),
                    value: true,
                    id: self.consume_id(),
                });
            }
            Token::False => {
                self.consume_token();
                return Ok(ExpressionNode::Boolean {
                    token: curr_token.clone(),
                    value: false,
                    id: self.consume_id(),
                });
            }
            Token::Bang | Token::Minus => {
                let token = self.curr_token.clone();
                let operator = match token {
                    Token::Bang => PrefixOp::Not,
                    Token::Minus => PrefixOp::Negative,
                    _ => unreachable!(),
                };
                // Move to the next token (the operand)
                self.consume_token();
                // Parse the right side with PREFIX precedence
                let right = self.parse_expression(Precedence::PREFIX)?;
                return Ok(ExpressionNode::Prefix {
                    token,
                    operator,
                    right: Box::new(right),
                    id: self.consume_id(),
                });
            }
            Token::LParen => {
                // LParen here handles the a + (b + c)
                self.consume_token();
                // Parse the expression inside parentheses with lowest precedence
                // to allow any expression to be parenthesized
                let expr = self.parse_expression(Precedence::Lowest)?;
                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after expression"));
                }
                self.consume_token();
                return Ok(expr);
            }
            Token::LSquare => {
                // same logic here as call expression
                // todo: this can be refactored I guess
                self.consume_token();
                let mut elements = vec![];
                while !matches!(self.curr_token, Token::RSquare)
                    && !matches!(self.curr_token, Token::Eof)
                {
                    elements.push(Box::new(self.parse_expression(Precedence::Lowest)?));

                    // End of elements
                    if matches!(self.curr_token, Token::RSquare) {
                        break;
                    }
                    // if its not comma sperated, return error
                    if !matches!(self.curr_token, Token::Comma) {
                        return Err(String::from("Expected comma"));
                    }
                    self.consume_token();
                }

                // Expect closing ']'
                if !matches!(self.curr_token, Token::RSquare) {
                    return Err(String::from("Expected ']' for array litera"));
                }
                self.consume_token();
                Ok(ExpressionNode::Array {
                    token: curr_token,
                    elements,
                    id: self.consume_id(),
                })
            }
            Token::LBrace => {
                self.consume_token();
                // empty object
                // Empty array
                let mut pairs = vec![];
                while !matches!(self.curr_token, Token::RBrace) {
                    let key = Box::new(self.parse_expression(Precedence::Lowest)?);
                    if !matches!(self.curr_token, Token::Colon) {
                        return Err(String::from(
                            "colon not found for key value assignment in hashmap",
                        ));
                    }
                    self.consume_token();
                    let value = Box::new(self.parse_expression(Precedence::Lowest)?);
                    pairs.push((key, value));

                    // End of attributes
                    if matches!(self.curr_token, Token::RBrace) {
                        break;
                    }
                    // if its not comma sperated, return error
                    if !matches!(self.curr_token, Token::Comma) {
                        return Err(String::from("Expected comma in object literal"));
                    }
                    self.consume_token();
                }
                if !matches!(self.curr_token, Token::RBrace) {
                    return Err(String::from("Expected '}' after hash literal"));
                }
                self.consume_token();
                Ok(ExpressionNode::HashMap {
                    token: Token::Colon,
                    pairs,
                    id: 0,
                })
            }
            Token::If => {
                // not the same as a ternary opreator, the if else block
                // if condition is not true and there is no else block, it is set undefined
                // last value on the block without a ; is the value returned
                self.consume_token();

                if !matches!(self.curr_token, Token::LParen) {
                    return Err(String::from("Expected '(' after 'if' keyword"));
                }
                self.consume_token();

                // Parse the condition expression
                let condition_exp = self.parse_expression(Precedence::Lowest)?;

                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after 'if' conditional"));
                }
                self.consume_token();

                if !matches!(self.curr_token, Token::LBrace) {
                    return Err(String::from("Expected '{' to start if block"));
                }
                let if_block = self.parse_statement(false)?;
                let mut else_block = None;

                if matches!(self.curr_token, Token::Else) {
                    self.consume_token();
                    else_block = Some(Box::new(self.parse_statement(false)?));
                }

                Ok(ExpressionNode::If {
                    token: curr_token.clone(),
                    condition: Box::new(condition_exp),
                    if_block: Box::new(if_block),
                    else_block: else_block,
                    id: self.consume_id(),
                })
            }
            Token::Function => {
                // consume func keyword
                self.consume_token();
                if !matches!(self.curr_token, Token::LParen) {
                    return Err(String::from("Expected '(' after 'func' keyword"));
                }
                // consume the (
                self.consume_token();
                let mut param_identifiers = vec![];
                while !matches!(self.curr_token, Token::RParen) {
                    let identifier = Box::new(self.parse_expression(Precedence::Lowest)?);
                    param_identifiers.push(identifier);
                    // End of attributes
                    if matches!(self.curr_token, Token::RParen) {
                        break;
                    }
                    // if its not comma sperated, return error
                    if !matches!(self.curr_token, Token::Comma) {
                        return Err(String::from("Expected comma in object literal"));
                    }
                    self.consume_token();
                }

                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after function param expression"));
                }
                self.consume_token();

                if !matches!(self.curr_token, Token::LBrace) {
                    return Err(String::from("Expected '{' before 'func' body"));
                }
                let result = ExpressionNode::Function {
                    token: curr_token.clone(),
                    parameters: param_identifiers,
                    // todo: finish this once blocks are done
                    body: Box::new(self.parse_statement(false)?),
                    id: self.consume_id(),
                };
                Ok(result)
            }
            // todo: handle the rest in abit
            _ => Err(String::from(format!(
                "Unexpected token {:?} in prefix position",
                self.curr_token.clone()
            ))),
        }
    }

    pub fn parse_infix(&mut self, left_exp: &ExpressionNode) -> Result<ExpressionNode, String> {
        let curr_token = self.curr_token.clone();
        self.consume_token();
        // current token at infix operator
        match curr_token.clone() {
            Token::LSquare => {
                let index_expr = self.parse_expression(Precedence::Lowest)?;
                // Expect closing ']'
                if !matches!(self.curr_token, Token::RSquare) {
                    return Err(String::from("Expected ']' after square indexing"));
                }
                self.consume_token();
                Ok(ExpressionNode::Index {
                    token: curr_token,
                    object: Box::new(left_exp.clone()),
                    index: Box::new(index_expr),
                    id: self.consume_id(),
                })
            }
            Token::LParen => {
                // Function call with no arguments
                let mut args = vec![];
                while !matches!(self.curr_token, Token::RParen)
                    && !matches!(self.curr_token, Token::Eof)
                {
                    args.push(Box::new(self.parse_expression(Precedence::Lowest)?));

                    // End of call
                    if matches!(self.curr_token, Token::RParen) {
                        break;
                    }
                    // if its not comma sperated, return error
                    if !matches!(self.curr_token, Token::Comma) {
                        return Err(String::from("Expected comma in function call"));
                    }
                    self.consume_token();
                }
                // Expect closing ')'
                if !matches!(self.curr_token, Token::RParen) {
                    return Err(String::from("Expected ')' after function call"));
                }
                self.consume_token();
                Ok(ExpressionNode::Call {
                    token: curr_token,
                    function: Box::new(left_exp.clone()),
                    arguments: args,
                    id: self.consume_id(),
                })
            }
            Token::Dot => {
                let field_name = match &self.curr_token {
                    Token::Ident(name) => name.clone(),
                    _ => {
                        return Err(format!(
                            "Expected identifier after dot, got {:?}",
                            self.curr_token
                        ));
                    }
                };
                self.consume_token();
                Ok(ExpressionNode::Index {
                    token: curr_token,
                    object: Box::new(left_exp.clone()),
                    index: Box::new(ExpressionNode::String {
                        token: Token::Str(field_name.clone()),
                        value: field_name,
                        id: 0,
                    }),
                    id: self.consume_id(),
                })
            }
            _ => {
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

                // Create the operation: left desugar_op right
                // e.g., x + 5 -> left_exp += 5 becomes left_exp = left_exp + 5
                let right_exp = match desugar_op {
                    Some(infix_op) => ExpressionNode::Infix {
                        token: curr_token.clone(),
                        left: Box::new(left_exp.clone()),
                        operator: infix_op,
                        right: Box::new(
                            self.parse_expression(self.get_precedence(&curr_token.clone()))?,
                        ),
                        id: self.consume_id(),
                    },
                    None => self.parse_expression(self.get_precedence(&curr_token.clone()))?,
                };
                Ok(ExpressionNode::Infix {
                    token: curr_token,
                    left: Box::new(left_exp.clone()),
                    operator,
                    right: Box::new(right_exp.clone()),
                    id: self.consume_id(),
                })
            }
        }
    }

    pub fn parse_mixfix(
        &mut self,
        condition_exp: &ExpressionNode,
    ) -> Result<ExpressionNode, String> {
        let curr_token = self.curr_token.clone();
        match curr_token {
            Token::Conditional => {
                self.consume_token();
                let then_expression = self.parse_expression(Precedence::Lowest)?;
                if !matches!(self.curr_token, Token::Colon) {
                    panic!("colon not found in mixfix");
                }
                self.consume_token();
                let else_expression = self.parse_expression(Precedence::Lowest)?;
                Ok(ExpressionNode::Ternary {
                    token: curr_token,
                    condition: Box::new(condition_exp.clone()),
                    then_expr: Box::new(then_expression),
                    else_expr: Box::new(else_expression),
                    id: self.consume_id(),
                })
            }
            _ => return Err(format!("Unexpected mixfix operator: {:?}", curr_token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::AstNode;

    use super::*;

    fn parse_expression(input: &str) -> Result<ExpressionNode, String> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse_expression(Precedence::Lowest)
    }

    fn parse_statement(input: &str) -> Result<StatementNode, String> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse_statement(true)
    }

    fn parse_program(input: &str) -> Result<Node, String> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    #[test]
    fn test_parse_program() {
        let input = "
        let x = 5 + 3;
        let y = func() {
            return \"hello\";
        };
        func test_func(x, y) {
            let arr = [x , y];
            let my_obj = {
                \"key\": \"value\"
            };
            return my_obj.key[1];
        };
        \"hello\";
        ";
        let ast = parse_program(input);
        match ast {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(
                    value.to_string(),
                    "let x = (5 + 3);let y = func() { return \"hello\"; };let test_func = func(x, y) { let arr = [x, y]; let my_obj = {\"key\": \"value\"}; return my_obj[\"key\"][1]; };\"hello\""
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "
        let x = 5 + 3;
        let y = func() {
            return \"hello\";
        };
        func test_func(x, y) {
            let arr = [x , y];
            let my_obj = {
                \"key\": \"value\"
            };
            if(x + y == 5) {
                return my_obj.key[1];
            };
            return my_obj.key[1];
        };
        \"hello\";
        ";
        let ast = parse_program(input);
        match ast {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(
                    value.to_string(),
                    "let x = (5 + 3);let y = func() { return \"hello\"; };let test_func = func(x, y) { let arr = [x, y]; let my_obj = {\"key\": \"value\"}; if ((x + y) == 5) { return my_obj[\"key\"][1]; }; return my_obj[\"key\"][1]; };\"hello\""
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "
        let x = 5 + 3;
        let y = func() {
            return \"hello\";
        };
        func test_func(x, y) {
            let arr = [x , y];
            let my_obj = {
                \"key\": \"value\"
            };
            if(x + y == 5) {
                return my_obj.key[1];
            } else {
                return my_obj.key[1];
            };
        };
        \"hello\";
        ";
        let ast = parse_program(input);
        match ast {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(
                    value.to_string(),
                    "let x = (5 + 3);let y = func() { return \"hello\"; };let test_func = func(x, y) { let arr = [x, y]; let my_obj = {\"key\": \"value\"}; if ((x + y) == 5) { return my_obj[\"key\"][1]; } else { return my_obj[\"key\"][1]; }; };\"hello\""
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }
    }

    #[test]
    fn test_parse_statement() {
        let input = "let x = 5 + 3;";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(Node::StatementNode(value).to_string(), "let x = (5 + 3);");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "let x = y = 5;";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(Node::StatementNode(value).to_string(), "let x = (y = 5);");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "let x = {\"harris\": {\"other\": \"nested\"}};";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(
                    Node::StatementNode(value).to_string(),
                    "let x = {\"harris\": {\"other\": \"nested\"}};"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "let x = {\"harris\": [{\"test\": \"nested\"}, 2, 3]};";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(
                    Node::StatementNode(value).to_string(),
                    "let x = {\"harris\": [{\"test\": \"nested\"}, 2, 3]};"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "for(x > 0){ let x = x + 1;};";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(
                    Node::StatementNode(value).to_string(),
                    "for (x > 0) { let x = (x + 1); };"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "func x(y, z){ let w = y + z; return w;};";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                // todo: fix the func declr printing
                assert_eq!(
                    Node::StatementNode(value).to_string(),
                    "let x = func(y, z) { let w = (y + z); return w; };"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "func x(y, z) { let w = (y + z); let u = func(n, m){ return n + m; }; return u(w, z); };";
        let statement = parse_statement(input);

        match statement {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                // todo: fix the block ; printing as well
                assert_eq!(
                    Node::StatementNode(value).to_string(),
                    "let x = func(y, z) { let w = (y + z); let u = func(n, m) { return (n + m); }; return u(w, z); };"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }
    }

    #[test]
    fn test_parse_expression() {
        let input = "5 + 3 + 2";

        let expression = parse_expression(input);

        match expression {
            Ok(value) => {
                // "5 + 3 + 2" parses left-to-right as ((5 + 3) + 2)
                assert_eq!(Node::ExpressionNode(value).to_string(), "((5 + 3) + 2)");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "5 + 3 * 2";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "(5 + (3 * 2))");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "5 + (3 + 2)";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "(5 + (3 + 2))");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "!5 + (3 + 2)";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "((!5) + (3 + 2))");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "true ? 5 + 3: 2 + 4";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "true ? (5 + 3) : (2 + 4)"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }
        let input = "5 + 3 ? !5 + (3 + 2) : 5 + 3 * 2";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "(5 + 3) ? ((!5) + (3 + 2)) : (5 + (3 * 2))"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x[5 + 3]";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x[(5 + 3)]");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x[True ? 5 : 3]";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x[True ? 5 : 3]");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x()";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x()");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x(1 + 2)";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x((1 + 2))");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x(1 + 2, 3)";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x((1 + 2), 3)");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x(1 + 2, 3, 5, 6)";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "x((1 + 2), 3, 5, 6)"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x(y())";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x(y())");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x(y(1, 3, 4), w(true ? 5 + 3 * 2 : 3, \"hello\"))";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "x(y(1, 3, 4), w(true ? (5 + (3 * 2)) : 3, \"hello\"))"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x += 5";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "(x = (x + 5))");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "[1, 5, \"harris\", x(), true ? 4 : 3 * 5]";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "[1, 5, \"harris\", x(), true ? 4 : (3 * 5)]"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "[]";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "[]");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "x([1, 2, 4], 5)";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "x([1, 2, 4], 5)");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "[[1, 2, 5], \"test\"]";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "[[1, 2, 5], \"test\"]"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "{\"harris\": 5 + 2}";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "{\"harris\": (5 + 2)}"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "{\"harris\": 5 + 2, \"other\": true ? 5 : -1, \"array\": [1, 3, x()]}";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "{\"harris\": (5 + 2), \"other\": true ? 5 : (-1), \"array\": [1, 3, x()]}"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "{}";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(Node::ExpressionNode(value).to_string(), "{}");
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "func(x, y, z) { let x = \"harris\"; }";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "func(x, y, z) { let x = \"harris\"; }"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "func(x, y, z) { let x = \"harris\"; return x; }";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "func(x, y, z) { let x = \"harris\"; return x; }"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "if(true){ let y = 5; }";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "if true { let y = 5; };"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }

        let input = "if(true){ let y = 5; } else { let y = z; }";
        let expression = parse_expression(input);
        match expression {
            Ok(value) => {
                assert_eq!(
                    Node::ExpressionNode(value).to_string(),
                    "if true { let y = 5; } else { let y = z; };"
                );
            }
            Err(e) => panic!("Expected expression node {}", e),
        }
    }
}
