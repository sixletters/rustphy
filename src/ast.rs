//! Abstract Syntax Tree (AST) representation for the Rustphy language.
//!
//! This module defines the AST node types used to represent parsed Rustphy code.
//! The AST is structured as a hierarchy of nodes, with each node representing
//! a syntactic construct in the language (statements, expressions, etc.).

use crate::instruction::{BINOPS, UNOPS};
use crate::token::Token;
use serde::Serialize;
use std::fmt;

/// Prefix operators used in prefix expressions.
///
/// These operators appear before their operand (e.g., `-x`, `!condition`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PrefixOp {
    /// Negation operator (`-`)
    Negative,
    /// Logical NOT operator (`!`)
    Not,
}

impl From<PrefixOp> for UNOPS {
    /// Converts a prefix operator to its corresponding unary operation instruction.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustphy::ast::PrefixOp;
    /// use rustphy::instruction::UNOPS;
    ///
    /// assert_eq!(UNOPS::from(PrefixOp::Negative), UNOPS::Negative);
    /// assert_eq!(UNOPS::from(PrefixOp::Not), UNOPS::Not);
    /// ```
    fn from(op: PrefixOp) -> Self {
        match op {
            PrefixOp::Negative => UNOPS::Negative,
            PrefixOp::Not => UNOPS::Not,
        }
    }
}

impl fmt::Display for PrefixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PrefixOp::Negative => "-",
            PrefixOp::Not => "!",
        };
        write!(f, "{}", s)
    }
}

/// Infix operators used in binary expressions.
///
/// These operators appear between two operands (e.g., `x + y`, `a == b`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum InfixOp {
    /// Addition operator (`+`)
    Add,
    /// Subtraction operator (`-`)
    Subtract,
    /// Multiplication operator (`*`)
    Multiply,
    /// Division operator (`/`)
    Divide,
    /// Less than comparison (`<`)
    Lt,
    /// Greater than comparison (`>`)
    Gt,
    /// Equality comparison (`==`)
    Eq,
    /// Inequality comparison (`!=`)
    NotEq,
    /// Logical AND operator (`&&`)
    And,
    /// Logical OR operator (`||`)
    Or,
    /// Assign
    Assign,
}

impl From<InfixOp> for BINOPS {
    /// Converts an infix operator to its corresponding binary operation instruction.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustphy::ast::InfixOp;
    /// use rustphy::instruction::BINOPS;
    ///
    /// assert_eq!(BINOPS::from(InfixOp::Add), BINOPS::Add);
    /// assert_eq!(BINOPS::from(InfixOp::Multiply), BINOPS::Multiply);
    /// ```
    fn from(op: InfixOp) -> Self {
        match op {
            InfixOp::Add => BINOPS::Add,
            InfixOp::Subtract => BINOPS::Minus,
            InfixOp::Multiply => BINOPS::Multiply,
            InfixOp::Divide => BINOPS::Divide,
            InfixOp::Lt => BINOPS::Lt,
            InfixOp::Gt => BINOPS::Gt,
            InfixOp::Eq => BINOPS::Eq,
            InfixOp::NotEq => BINOPS::Neq,
            InfixOp::And => BINOPS::And,
            InfixOp::Or => BINOPS::Or,
            InfixOp::Assign => BINOPS::Assign,
        }
    }
}

impl fmt::Display for InfixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InfixOp::Add => "+",
            InfixOp::Subtract => "-",
            InfixOp::Multiply => "*",
            InfixOp::Divide => "/",
            InfixOp::Lt => "<",
            InfixOp::Gt => ">",
            InfixOp::Eq => "==",
            InfixOp::NotEq => "!=",
            InfixOp::And => "&&",
            InfixOp::Or => "||",
            InfixOp::Assign => "=",
        };
        write!(f, "{}", s)
    }
}

/// Root node type that wraps either a statement or an expression.
///
/// This enum serves as a unified interface for all AST nodes, allowing
/// them to be treated uniformly when implementing the `AstNode` trait.
#[derive(Debug, Serialize)]
pub enum Node {
    /// A statement node (e.g., let, return, for, etc.)
    StatementNode(StatementNode),
    /// An expression node (e.g., identifiers, integers, function calls, etc.)
    ExpressionNode(ExpressionNode),
}

/// Represents all statement types in the Rustphy language.
///
/// Statements are instructions that perform actions but don't produce values
/// (though some like Expression statements wrap expressions).
#[derive(Debug, Serialize)]
pub enum StatementNode {
    /// The root program node containing all top-level statements.
    ///
    /// Note: Program nodes cannot be cloned due to containing trait objects.
    Program {
        statements: Vec<Box<Node>>,
        implicit_return: Option<ExpressionNode>,
        id: i32,
    },
    /// A variable declaration statement (`let name = value;`)
    Let {
        token: Token,
        value: ExpressionNode,
        name: ExpressionNode,
        id: i32,
    },
    /// A return statement (`return value;`)
    Return {
        token: Token,
        return_value: ExpressionNode,
        id: i32,
    },
    /// An expression used as a statement
    Expression {
        token: Token,
        expression: ExpressionNode,
        id: i32,
    },
    /// A block of statements enclosed in braces (`{ ... }`)
    Block {
        token: Token,
        statements: Vec<Box<StatementNode>>,
        implicit_return: Option<ExpressionNode>,
        id: i32,
    },
    /// A for loop statement (`for condition { ... }`)
    For {
        token: Token,
        condition: ExpressionNode,
        for_block: Box<StatementNode>,
        id: i32,
    },
    FuncDeclr {
        token: Token,
        identifier: ExpressionNode,
        func: ExpressionNode,
        id: i32,
    },
    Break {
        token: Token,
        id: i32,
    },
    Continue {
        token: Token,
        id: i32,
    },
}

impl Clone for StatementNode {
    fn clone(&self) -> Self {
        match self {
            StatementNode::Program { .. } => {
                panic!("Cannot clone Program nodes")
            }
            StatementNode::Let {
                token,
                value,
                name,
                id,
            } => StatementNode::Let {
                token: token.clone(),
                value: value.clone(),
                name: name.clone(),
                id: *id,
            },
            StatementNode::Return {
                token,
                return_value,
                id,
            } => StatementNode::Return {
                token: token.clone(),
                return_value: return_value.clone(),
                id: *id,
            },
            StatementNode::Expression {
                token,
                expression,
                id,
            } => StatementNode::Expression {
                token: token.clone(),
                expression: expression.clone(),
                id: *id,
            },
            StatementNode::Block {
                token,
                statements,
                implicit_return,
                id,
            } => StatementNode::Block {
                token: token.clone(),
                statements: statements.clone(),
                implicit_return: implicit_return.clone(),
                id: *id,
            },
            StatementNode::For {
                token,
                condition,
                for_block,
                id,
            } => StatementNode::For {
                token: token.clone(),
                condition: condition.clone(),
                for_block: for_block.clone(),
                id: *id,
            },
            StatementNode::FuncDeclr {
                token,
                identifier,
                func,
                id,
            } => StatementNode::FuncDeclr {
                token: token.clone(),
                identifier: identifier.clone(),
                func: func.clone(),
                id: *id,
            },
            StatementNode::Break { token, id } => StatementNode::Break {
                token: token.clone(),
                id: *id,
            },
            StatementNode::Continue { token, id } => StatementNode::Continue {
                token: token.clone(),
                id: *id,
            },
        }
    }
}

/// Represents all expression types in the Rustphy language.
///
/// Expressions are constructs that produce values when evaluated.
#[derive(Clone, Debug, Serialize)]
pub enum ExpressionNode {
    /// An identifier (variable or function name)
    Identifier {
        token: Token,
        value: String,
        id: i32,
    },
    /// An integer literal
    Integer { token: Token, value: i64, id: i32 },
    // A string literal
    String {
        token: Token,
        value: String,
        id: i32,
    },
    /// A prefix operator expression (e.g., `-x`, `!y`)
    Prefix {
        token: Token,
        operator: PrefixOp,
        right: Box<ExpressionNode>,
        id: i32,
    },
    // Index expression (e.g Arr[0])
    Index {
        token: Token,
        // Expression evaluates to an array or subscriptable object
        object: Box<ExpressionNode>,
        // Expression evaluates to an index/number
        index: Box<ExpressionNode>,
        id: i32,
    },
    /// An infix operator expression (e.g., `x + y`, `a == b`)
    Infix {
        token: Token,
        left: Box<ExpressionNode>,
        operator: InfixOp,
        right: Box<ExpressionNode>,
        id: i32,
    },
    /// A boolean literal (`true` or `false`)
    Boolean { token: Token, value: bool, id: i32 },
    Ternary {
        token: Token,
        condition: Box<ExpressionNode>,
        then_expr: Box<ExpressionNode>,
        else_expr: Box<ExpressionNode>,
        id: i32,
    },
    /// An if-else conditional expression
    If {
        token: Token,
        condition: Box<ExpressionNode>,
        if_block: Box<StatementNode>,
        else_block: Option<Box<StatementNode>>,
        id: i32,
    },
    /// A function literal (`func(params) { body }`)
    Function {
        token: Token,
        parameters: Vec<Box<ExpressionNode>>,
        body: Box<StatementNode>,
        id: i32,
    },
    /// A function call expression (`function(args)`)
    Call {
        token: Token,
        function: Box<ExpressionNode>,
        arguments: Vec<Box<ExpressionNode>>,
        id: i32,
    },
    // Array Literal
    Array {
        token: Token,
        elements: Vec<Box<ExpressionNode>>,
        id: i32,
    },
    HashMap {
        token: Token,
        pairs: Vec<(Box<ExpressionNode>, Box<ExpressionNode>)>,
        id: i32,
    },
}

/// Trait implemented by all AST nodes.
///
/// This trait provides common functionality for working with AST nodes,
/// including token access and string representation.
pub trait AstNode {
    /// Returns the token associated with this node.
    fn get_token(&self) -> Token;

    /// Returns the literal string representation of the node's token.
    fn token_literal(&self) -> String;

    /// Converts the node back to its source code representation.
    ///
    /// This is useful for debugging and for generating code from the AST.
    fn to_string(&self) -> String;
}

impl AstNode for Node {
    fn get_token(&self) -> Token {
        match self {
            Self::ExpressionNode(val) => match val {
                ExpressionNode::Identifier { token, .. } => token.clone(),
                ExpressionNode::Integer { token, .. } => token.clone(),
                ExpressionNode::Prefix { token, .. } => token.clone(),
                ExpressionNode::Infix { token, .. } => token.clone(),
                ExpressionNode::Boolean { token, .. } => token.clone(),
                ExpressionNode::If { token, .. } => token.clone(),
                ExpressionNode::Function { token, .. } => token.clone(),
                ExpressionNode::Call { token, .. } => token.clone(),
                ExpressionNode::String { token, .. } => token.clone(),
                ExpressionNode::Array { token, .. } => token.clone(),
                ExpressionNode::Index { token, .. } => token.clone(),
                ExpressionNode::HashMap { token, .. } => token.clone(),
                ExpressionNode::Ternary { token, .. } => token.clone(),
            },
            Self::StatementNode(val) => match val {
                StatementNode::Program { statements, .. } => {
                    // Program doesn't have a token, return the first statement's token if available
                    if let Some(first_stmt) = statements.first() {
                        return first_stmt.get_token().clone();
                    } else {
                        Token::Eof // Default token for empty programs
                    }
                }
                StatementNode::Let { token, .. } => token.clone(),
                StatementNode::Return { token, .. } => token.clone(),
                StatementNode::Expression { token, .. } => token.clone(),
                StatementNode::Block { token, .. } => token.clone(),
                StatementNode::For { token, .. } => token.clone(),
                StatementNode::FuncDeclr { token, .. } => token.clone(),
                StatementNode::Break { token, .. } => token.clone(),
                StatementNode::Continue { token, .. } => token.clone(),
            },
        }
    }

    fn token_literal(&self) -> String {
        let token = self.get_token();
        match token {
            Token::Illegal(c) => c.to_string(),
            Token::Eof => "EOF".to_string(),
            Token::Ident(s) => s,
            Token::Int(s) => s,
            Token::Str(s) => s,
            Token::Assign => "=".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Bang => "!".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Lt => "<".to_string(),
            Token::Gt => ">".to_string(),
            Token::Eq => "==".to_string(),
            Token::NotEq => "!=".to_string(),
            Token::And => "&&".to_string(),
            Token::Or => "||".to_string(),
            Token::PlusAssign => "+=".to_string(),
            Token::MinusAssign => "-=".to_string(),
            Token::AsteriskAssign => "*=".to_string(),
            Token::SlashAssign => "/=".to_string(),
            Token::Comma => ",".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
            Token::LBrace => "{".to_string(),
            Token::RBrace => "}".to_string(),
            Token::Function => "func".to_string(),
            Token::For => "for".to_string(),
            Token::Let => "let".to_string(),
            Token::True => "true".to_string(),
            Token::False => "false".to_string(),
            Token::If => "if".to_string(),
            Token::Else => "else".to_string(),
            Token::Return => "return".to_string(),
            Token::LSquare => "[".to_string(),
            Token::RSquare => "]".to_string(),
            Token::Break => "break".to_string(),
            Token::Continue => "continue".to_string(),
            Token::Colon => ":".to_string(),
            Token::Dot => ".".to_string(),
            Token::Conditional => "?".to_string(),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Self::ExpressionNode(val) => match val {
                ExpressionNode::Identifier { value, .. } => value.clone(),
                ExpressionNode::Integer { value, .. } => value.to_string(),
                ExpressionNode::Boolean { value, .. } => {
                    if *value {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                ExpressionNode::Ternary {
                    condition,
                    then_expr,
                    else_expr,
                    ..
                } => {
                    format!(
                        "{} ? {} : {}",
                        Node::ExpressionNode((**condition).clone()).to_string(),
                        Node::ExpressionNode((**then_expr).clone()).to_string(),
                        Node::ExpressionNode((**else_expr).clone()).to_string()
                    )
                }
                ExpressionNode::HashMap { pairs, .. } => {
                    let mut key_values = String::new(); // ✅ Make it mutable
                    key_values.push_str("{");
                    for (i, p) in pairs.iter().enumerate() {
                        if i > 0 {
                            key_values.push_str(", "); // Add comma between pairs
                        }
                        key_values.push_str(&format!(
                            "{}: {}",
                            Node::ExpressionNode(*p.0.clone()).to_string(),
                            Node::ExpressionNode(*p.1.clone()).to_string(),
                        ));
                    }
                    key_values.push_str("}");
                    key_values
                }
                ExpressionNode::Index { object, index, .. } => {
                    format!(
                        "{}[{}]",
                        Node::ExpressionNode((**object).clone()).to_string(),
                        Node::ExpressionNode((**index).clone()).to_string()
                    )
                }
                ExpressionNode::Prefix {
                    operator, right, ..
                } => {
                    format!(
                        "({}{})",
                        operator,
                        Node::ExpressionNode((**right).clone()).to_string()
                    )
                }
                ExpressionNode::Infix {
                    left,
                    operator,
                    right,
                    ..
                } => {
                    format!(
                        "({} {} {})",
                        Node::ExpressionNode((**left).clone()).to_string(),
                        operator,
                        Node::ExpressionNode((**right).clone()).to_string()
                    )
                }
                ExpressionNode::If {
                    condition,
                    if_block,
                    else_block,
                    ..
                } => {
                    let result = format!(
                        "if {} {}",
                        Node::ExpressionNode((**condition).clone()).to_string(),
                        Node::StatementNode((**if_block).clone()).to_string()
                    );

                    // Check if else block exists and has statements
                    if let Some(else_blk) = else_block {
                        if let StatementNode::Block { statements, .. } = else_blk.as_ref() {
                            if !statements.is_empty() {
                                return format!(
                                    "{} else {};",
                                    result,
                                    Node::StatementNode((**else_blk).clone()).to_string()
                                );
                            }
                        }
                    }
                    format!("{};", result)
                }
                ExpressionNode::Function {
                    parameters, body, ..
                } => {
                    let params: Vec<String> = parameters
                        .iter()
                        .map(|p| Node::ExpressionNode((**p).clone()).to_string())
                        .collect();
                    format!(
                        "func({}) {}",
                        params.join(", "),
                        Node::StatementNode((**body).clone()).to_string()
                    )
                }
                ExpressionNode::String { value, .. } => {
                    format!("\"{}\"", value)
                }
                ExpressionNode::Call {
                    function,
                    arguments,
                    ..
                } => {
                    let args: Vec<String> = arguments
                        .iter()
                        .map(|a| Node::ExpressionNode((**a).clone()).to_string())
                        .collect();
                    format!(
                        "{}({})",
                        Node::ExpressionNode((**function).clone()).to_string(),
                        args.join(", ")
                    )
                }
                ExpressionNode::Array { elements, .. } => {
                    let args: Vec<String> = elements
                        .iter()
                        .map(|a| Node::ExpressionNode((**a).clone()).to_string())
                        .collect();
                    format!("[{}]", args.join(", "))
                }
            },
            Self::StatementNode(val) => match val {
                StatementNode::Program {
                    statements,
                    implicit_return,
                    ..
                } => {
                    let mut result = statements
                        .iter()
                        .map(|stmt| stmt.to_string())
                        .collect::<Vec<String>>()
                        .join("");

                    // If there's an implicit return, append it without semicolon
                    if let Some(expr) = implicit_return {
                        result.push_str(&Node::ExpressionNode(expr.clone()).to_string());
                    }

                    result
                }
                StatementNode::Break { .. } => "break;".to_string(),
                StatementNode::Continue { .. } => "continue;".to_string(),
                StatementNode::Let { name, value, .. } => {
                    format!(
                        "let {} = {};",
                        Node::ExpressionNode(name.clone()).to_string(),
                        Node::ExpressionNode(value.clone()).to_string()
                    )
                }
                StatementNode::Return { return_value, .. } => {
                    format!(
                        "return {};",
                        Node::ExpressionNode(return_value.clone()).to_string()
                    )
                }
                StatementNode::Expression { expression, .. } => {
                    Node::ExpressionNode(expression.clone()).to_string()
                }
                StatementNode::Block {
                    statements,
                    implicit_return,
                    ..
                } => {
                    let mut stmts: Vec<String> = statements
                        .iter()
                        .map(|s| Node::StatementNode((**s).clone()).to_string())
                        .collect();

                    // Add implicit return if present
                    if let Some(ret_expr) = implicit_return {
                        stmts.push(Node::ExpressionNode(ret_expr.clone()).to_string());
                    }

                    format!("{{ {} }}", stmts.join(" "))
                }
                StatementNode::For {
                    condition,
                    for_block,
                    ..
                } => {
                    format!(
                        "for {} {};",
                        Node::ExpressionNode(condition.clone()).to_string(),
                        Node::StatementNode((**for_block).clone()).to_string()
                    )
                }
                StatementNode::FuncDeclr {
                    identifier, func, ..
                } => {
                    format!(
                        "let {} = {};",
                        Node::ExpressionNode(identifier.clone()).to_string(),
                        Node::ExpressionNode(func.clone()).to_string()
                    )
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_to_string() {
        let ident = ExpressionNode::Identifier {
            token: Token::Ident("myVar".to_string()),
            value: "myVar".to_string(),
            id: 0,
        };
        let node = Node::ExpressionNode(ident);
        assert_eq!(node.to_string(), "myVar");
    }

    #[test]
    fn test_integer_to_string() {
        let int = ExpressionNode::Integer {
            token: Token::Int("42".to_string()),
            value: 42,
            id: 0,
        };
        let node = Node::ExpressionNode(int);
        assert_eq!(node.to_string(), "42");
    }

    #[test]
    fn test_boolean_to_string() {
        let bool_true = ExpressionNode::Boolean {
            token: Token::True,
            value: true,
            id: 0,
        };
        let node_true = Node::ExpressionNode(bool_true);
        assert_eq!(node_true.to_string(), "true");

        let bool_false = ExpressionNode::Boolean {
            token: Token::False,
            value: false,
            id: 0,
        };
        let node_false = Node::ExpressionNode(bool_false);
        assert_eq!(node_false.to_string(), "false");
    }

    #[test]
    fn test_prefix_expression_to_string() {
        let prefix = ExpressionNode::Prefix {
            token: Token::Bang,
            operator: PrefixOp::Not,
            right: Box::new(ExpressionNode::Boolean {
                token: Token::True,
                value: true,
                id: 0,
            }),
            id: 0,
        };
        let node = Node::ExpressionNode(prefix);
        assert_eq!(node.to_string(), "(!true)");
    }

    #[test]
    fn test_infix_expression_to_string() {
        let infix = ExpressionNode::Infix {
            token: Token::Plus,
            left: Box::new(ExpressionNode::Integer {
                token: Token::Int("5".to_string()),
                value: 5,
                id: 0,
            }),
            operator: InfixOp::Add,
            right: Box::new(ExpressionNode::Integer {
                token: Token::Int("10".to_string()),
                value: 10,
                id: 0,
            }),
            id: 0,
        };
        let node = Node::ExpressionNode(infix);
        assert_eq!(node.to_string(), "(5 + 10)");
    }

    #[test]
    fn test_nested_infix_expression_to_string() {
        let nested = ExpressionNode::Infix {
            token: Token::Asterisk,
            left: Box::new(ExpressionNode::Infix {
                token: Token::Plus,
                left: Box::new(ExpressionNode::Integer {
                    token: Token::Int("2".to_string()),
                    value: 2,
                    id: 0,
                }),
                operator: InfixOp::Add,
                right: Box::new(ExpressionNode::Integer {
                    token: Token::Int("3".to_string()),
                    value: 3,
                    id: 0,
                }),
                id: 0,
            }),
            operator: InfixOp::Multiply,
            right: Box::new(ExpressionNode::Integer {
                token: Token::Int("4".to_string()),
                value: 4,
                id: 0,
            }),
            id: 0,
        };
        let node = Node::ExpressionNode(nested);
        assert_eq!(node.to_string(), "((2 + 3) * 4)");
    }

    #[test]
    fn test_let_statement_to_string() {
        let let_stmt = StatementNode::Let {
            token: Token::Let,
            name: ExpressionNode::Identifier {
                token: Token::Ident("x".to_string()),
                value: "x".to_string(),
                id: 0,
            },
            value: ExpressionNode::Integer {
                token: Token::Int("5".to_string()),
                value: 5,
                id: 0,
            },
            id: 0,
        };
        let node = Node::StatementNode(let_stmt);
        assert_eq!(node.to_string(), "let x = 5;");
    }

    #[test]
    fn test_return_statement_to_string() {
        let return_stmt = StatementNode::Return {
            token: Token::Return,
            return_value: ExpressionNode::Integer {
                token: Token::Int("42".to_string()),
                value: 42,
                id: 0,
            },
            id: 0,
        };
        let node = Node::StatementNode(return_stmt);
        assert_eq!(node.to_string(), "return 42;");
    }

    #[test]
    fn test_expression_statement_to_string() {
        let expr_stmt = StatementNode::Expression {
            token: Token::Ident("x".to_string()),
            expression: ExpressionNode::Identifier {
                token: Token::Ident("x".to_string()),
                value: "x".to_string(),
                id: 0,
            },
            id: 0,
        };
        let node = Node::StatementNode(expr_stmt);
        assert_eq!(node.to_string(), "x");
    }

    #[test]
    fn test_block_statement_to_string() {
        let block = StatementNode::Block {
            token: Token::LBrace,
            statements: vec![
                Box::new(StatementNode::Expression {
                    token: Token::Ident("x".to_string()),
                    expression: ExpressionNode::Identifier {
                        token: Token::Ident("x".to_string()),
                        value: "x".to_string(),
                        id: 0,
                    },
                    id: 0,
                }),
                Box::new(StatementNode::Return {
                    token: Token::Return,
                    return_value: ExpressionNode::Integer {
                        token: Token::Int("5".to_string()),
                        value: 5,
                        id: 0,
                    },
                    id: 0,
                }),
            ],
            implicit_return: None,
            id: 0,
        };
        let node = Node::StatementNode(block);
        assert_eq!(node.to_string(), "{ x return 5; }");
    }

    #[test]
    fn test_if_expression_without_else_to_string() {
        let if_expr = ExpressionNode::If {
            token: Token::If,
            condition: Box::new(ExpressionNode::Boolean {
                token: Token::True,
                value: true,
                id: 0,
            }),
            if_block: Box::new(StatementNode::Block {
                token: Token::LBrace,
                statements: vec![Box::new(StatementNode::Expression {
                    token: Token::Ident("x".to_string()),
                    expression: ExpressionNode::Identifier {
                        token: Token::Ident("x".to_string()),
                        value: "x".to_string(),
                        id: 0,
                    },
                    id: 0,
                })],
                implicit_return: None,
                id: 0,
            }),
            else_block: Some(Box::new(StatementNode::Block {
                token: Token::LBrace,
                statements: vec![],
                implicit_return: None,
                id: 0,
            })),
            id: 0,
        };
        let node = Node::ExpressionNode(if_expr);
        assert_eq!(node.to_string(), "if true { x };");
    }

    #[test]
    fn test_if_expression_with_else_to_string() {
        let if_expr = ExpressionNode::If {
            token: Token::If,
            condition: Box::new(ExpressionNode::Boolean {
                token: Token::True,
                value: true,
                id: 0,
            }),
            if_block: Box::new(StatementNode::Block {
                token: Token::LBrace,
                statements: vec![Box::new(StatementNode::Expression {
                    token: Token::Ident("x".to_string()),
                    expression: ExpressionNode::Identifier {
                        token: Token::Ident("x".to_string()),
                        value: "x".to_string(),
                        id: 0,
                    },
                    id: 0,
                })],
                implicit_return: None,
                id: 0,
            }),
            else_block: Some(Box::new(StatementNode::Block {
                token: Token::LBrace,
                statements: vec![Box::new(StatementNode::Expression {
                    token: Token::Ident("y".to_string()),
                    expression: ExpressionNode::Identifier {
                        token: Token::Ident("y".to_string()),
                        value: "y".to_string(),
                        id: 0,
                    },
                    id: 0,
                })],
                implicit_return: None,
                id: 0,
            })),
            id: 0,
        };
        let node = Node::ExpressionNode(if_expr);
        assert_eq!(node.to_string(), "if true { x } else { y };");
    }

    #[test]
    fn test_function_literal_to_string() {
        let func = ExpressionNode::Function {
            token: Token::Function,
            parameters: vec![
                Box::new(ExpressionNode::Identifier {
                    token: Token::Ident("x".to_string()),
                    value: "x".to_string(),
                    id: 0,
                }),
                Box::new(ExpressionNode::Identifier {
                    token: Token::Ident("y".to_string()),
                    value: "y".to_string(),
                    id: 0,
                }),
            ],
            body: Box::new(StatementNode::Block {
                token: Token::LBrace,
                statements: vec![Box::new(StatementNode::Return {
                    token: Token::Return,
                    return_value: ExpressionNode::Identifier {
                        token: Token::Ident("x".to_string()),
                        value: "x".to_string(),
                        id: 0,
                    },
                    id: 0,
                })],
                implicit_return: None,
                id: 0,
            }),
            id: 0,
        };
        let node = Node::ExpressionNode(func);
        assert_eq!(node.to_string(), "func(x, y) { return x; }");
    }

    #[test]
    fn test_call_expression_to_string() {
        let call = ExpressionNode::Call {
            token: Token::LParen,
            function: Box::new(ExpressionNode::Identifier {
                token: Token::Ident("add".to_string()),
                value: "add".to_string(),
                id: 0,
            }),
            arguments: vec![
                Box::new(ExpressionNode::Integer {
                    token: Token::Int("1".to_string()),
                    value: 1,
                    id: 0,
                }),
                Box::new(ExpressionNode::Integer {
                    token: Token::Int("2".to_string()),
                    value: 2,
                    id: 0,
                }),
            ],
            id: 0,
        };
        let node = Node::ExpressionNode(call);
        assert_eq!(node.to_string(), "add(1, 2)");
    }

    #[test]
    fn test_for_statement_to_string() {
        let for_stmt = StatementNode::For {
            token: Token::For,
            condition: ExpressionNode::Boolean {
                token: Token::True,
                value: true,
                id: 0,
            },
            for_block: Box::new(StatementNode::Block {
                token: Token::LBrace,
                statements: vec![Box::new(StatementNode::Expression {
                    token: Token::Ident("x".to_string()),
                    expression: ExpressionNode::Identifier {
                        token: Token::Ident("x".to_string()),
                        value: "x".to_string(),
                        id: 0,
                    },
                    id: 0,
                })],
                implicit_return: None,
                id: 0,
            }),
            id: 0,
        };
        let node = Node::StatementNode(for_stmt);
        assert_eq!(node.to_string(), "for true { x };");
    }

    #[test]
    fn test_get_token_expression() {
        let ident = ExpressionNode::Identifier {
            token: Token::Ident("test".to_string()),
            value: "test".to_string(),
            id: 0,
        };
        let node = Node::ExpressionNode(ident);
        assert_eq!(node.get_token(), Token::Ident("test".to_string()));
    }

    #[test]
    fn test_get_token_statement() {
        let let_stmt = StatementNode::Let {
            token: Token::Let,
            name: ExpressionNode::Identifier {
                token: Token::Ident("x".to_string()),
                value: "x".to_string(),
                id: 0,
            },
            value: ExpressionNode::Integer {
                token: Token::Int("5".to_string()),
                value: 5,
                id: 0,
            },
            id: 0,
        };
        let node = Node::StatementNode(let_stmt);
        assert_eq!(node.get_token(), Token::Let);
    }

    #[test]
    fn test_token_literal() {
        let return_stmt = StatementNode::Return {
            token: Token::Return,
            return_value: ExpressionNode::Integer {
                token: Token::Int("42".to_string()),
                value: 42,
                id: 0,
            },
            id: 0,
        };
        let node = Node::StatementNode(return_stmt);
        assert_eq!(node.token_literal(), "return");
    }

    #[test]
    fn test_clone_expression_node() {
        let original = ExpressionNode::Integer {
            token: Token::Int("42".to_string()),
            value: 42,
            id: 0,
        };
        let cloned = original.clone();
        let node1 = Node::ExpressionNode(original);
        let node2 = Node::ExpressionNode(cloned);
        assert_eq!(node1.to_string(), node2.to_string());
    }

    #[test]
    fn test_clone_statement_node() {
        let original = StatementNode::Return {
            token: Token::Return,
            return_value: ExpressionNode::Integer {
                token: Token::Int("42".to_string()),
                value: 42,
                id: 0,
            },
            id: 0,
        };
        let cloned = original.clone();
        let node1 = Node::StatementNode(original);
        let node2 = Node::StatementNode(cloned);
        assert_eq!(node1.to_string(), node2.to_string());
    }

    #[test]
    #[should_panic(expected = "Cannot clone Program nodes")]
    fn test_clone_program_node_panics() {
        let program = StatementNode::Program {
            statements: vec![],
            implicit_return: None,
            id: 0,
        };
        let _cloned = program.clone();
    }

    #[test]
    fn test_complex_expression() {
        // Test: let result = add(2 * 3, 4 + 5);
        let let_stmt = StatementNode::Let {
            token: Token::Let,
            name: ExpressionNode::Identifier {
                token: Token::Ident("result".to_string()),
                value: "result".to_string(),
                id: 0,
            },
            value: ExpressionNode::Call {
                token: Token::LParen,
                function: Box::new(ExpressionNode::Identifier {
                    token: Token::Ident("add".to_string()),
                    value: "add".to_string(),
                    id: 0,
                }),
                arguments: vec![
                    Box::new(ExpressionNode::Infix {
                        token: Token::Asterisk,
                        left: Box::new(ExpressionNode::Integer {
                            token: Token::Int("2".to_string()),
                            value: 2,
                            id: 0,
                        }),
                        operator: InfixOp::Multiply,
                        right: Box::new(ExpressionNode::Integer {
                            token: Token::Int("3".to_string()),
                            value: 3,
                            id: 0,
                        }),
                        id: 0,
                    }),
                    Box::new(ExpressionNode::Infix {
                        token: Token::Plus,
                        left: Box::new(ExpressionNode::Integer {
                            token: Token::Int("4".to_string()),
                            value: 4,
                            id: 0,
                        }),
                        operator: InfixOp::Add,
                        right: Box::new(ExpressionNode::Integer {
                            token: Token::Int("5".to_string()),
                            value: 5,
                            id: 0,
                        }),
                        id: 0,
                    }),
                ],
                id: 0,
            },
            id: 0,
        };
        let node = Node::StatementNode(let_stmt);
        assert_eq!(node.to_string(), "let result = add((2 * 3), (4 + 5));");
    }

    #[test]
    fn test_prefix_op_to_unops() {
        // Test conversion using the From trait
        assert_eq!(UNOPS::from(PrefixOp::Negative), UNOPS::Negative);
        assert_eq!(UNOPS::from(PrefixOp::Not), UNOPS::Not);

        // Test conversion using .into()
        let neg_unop: UNOPS = PrefixOp::Negative.into();
        assert!(matches!(neg_unop, UNOPS::Negative));

        let not_unop: UNOPS = PrefixOp::Not.into();
        assert!(matches!(not_unop, UNOPS::Not));
    }

    #[test]
    fn test_infix_op_to_binops() {
        // Test conversion using the From trait
        assert_eq!(BINOPS::from(InfixOp::Add), BINOPS::Add);
        assert_eq!(BINOPS::from(InfixOp::Subtract), BINOPS::Minus);
        assert_eq!(BINOPS::from(InfixOp::Multiply), BINOPS::Multiply);
        assert_eq!(BINOPS::from(InfixOp::Divide), BINOPS::Divide);
        assert_eq!(BINOPS::from(InfixOp::Lt), BINOPS::Lt);
        assert_eq!(BINOPS::from(InfixOp::Gt), BINOPS::Gt);
        assert_eq!(BINOPS::from(InfixOp::Eq), BINOPS::Eq);
        assert_eq!(BINOPS::from(InfixOp::NotEq), BINOPS::Neq);

        // Test conversion using .into()
        let add_binop: BINOPS = InfixOp::Add.into();
        assert!(matches!(add_binop, BINOPS::Add));

        let multiply_binop: BINOPS = InfixOp::Multiply.into();
        assert!(matches!(multiply_binop, BINOPS::Multiply));
    }

    #[test]
    fn test_prefix_op_display() {
        assert_eq!(format!("{}", PrefixOp::Negative), "-");
        assert_eq!(format!("{}", PrefixOp::Not), "!");
    }

    #[test]
    fn test_infix_op_display() {
        assert_eq!(format!("{}", InfixOp::Add), "+");
        assert_eq!(format!("{}", InfixOp::Subtract), "-");
        assert_eq!(format!("{}", InfixOp::Multiply), "*");
        assert_eq!(format!("{}", InfixOp::Divide), "/");
        assert_eq!(format!("{}", InfixOp::Lt), "<");
        assert_eq!(format!("{}", InfixOp::Gt), ">");
        assert_eq!(format!("{}", InfixOp::Eq), "==");
        assert_eq!(format!("{}", InfixOp::NotEq), "!=");
    }
}
