use crate::ast::{ExpressionNode, Node, StatementNode};
use serde::Serialize;
use std::{cell::RefCell, collections::HashMap, io::SeekFrom, rc::Rc};

#[derive(Clone)]
enum ScopeType {
    Program,
    Function,
    Block,
}

#[derive(Clone, Debug)]
pub enum BindType {
    FunctionParam,
    FunctionDeclaration,
    VariableDeclaraion,
}

#[derive(Clone)]
pub struct Scope {
    pub id: usize,
    parent: Option<usize>,
    pub symbols: HashMap<String, Symbol>,
    pub scope_type: ScopeType,
    pub children: Vec<usize>, // children scope IDs
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub binding_id: usize,
    pub scope_id: usize,
    pub bind_type: BindType,
}

// Tree node used only for serialization
#[derive(Serialize)]
struct ScopeTree {
    id: usize,
    symbols: Vec<SymbolEntry>,
    children: Vec<ScopeTree>,
}

#[derive(Serialize)]
struct SymbolEntry {
    name: String,
    binding_id: usize,
}

pub struct SymbolTable<'a> {
    scopes: Vec<Rc<RefCell<Scope>>>,
    root_node: &'a Node,
    current_scope: usize,
    next_binding_id: usize,
    next_scope_id: usize,
    /// Maps node_id of each identifier usage to its resolved binding_id
    pub resolved: HashMap<i32, usize>,
    pub bindings: HashMap<usize, Symbol>, // binding_id → Symbol
    pub scope_bindings: HashMap<usize, Vec<usize>>, // scope_id → list of binding_ids
    pub node_scope: HashMap<i32, usize>,  // node_id → scope_id
}

// The a parameter on the impl is just like adding a new generic field
// it introduces the parameter
// the lifetime paramter in SymbolTable tells the compiler, we are implementing
// this for SymbolTable's parameterized by the lifetime paratmer
// it could be that we only implement methods for specific generic types
// rather than across all generic types
impl<'a> SymbolTable<'a> {
    pub fn new(root_node: &'a Node) -> Self {
        let table = Self {
            scopes: vec![],
            current_scope: 0,
            next_binding_id: 0,
            next_scope_id: 0,
            root_node,
            resolved: HashMap::new(),
            bindings: HashMap::new(),
            scope_bindings: HashMap::new(),
            node_scope: HashMap::new(),
        };
        table
    }

    fn enter_scope(&mut self, scope_type: ScopeType) -> usize {
        let parent = (!self.scopes.is_empty()).then_some(self.current_scope);
        let id = self.next_scope_id;
        self.scopes.push(Rc::new(RefCell::new(Scope {
            parent,
            id,
            symbols: HashMap::new(),
            scope_type: scope_type,
            children: vec![],
        })));
        // Add this scope to parent's children list, if parent exists
        if let Some(parent_id) = parent {
            self.scopes[parent_id].borrow_mut().children.push(id);
        }
        self.current_scope = id;
        self.next_scope_id += 1;
        id
    }

    fn exit_scope(&mut self) {
        self.current_scope = self.scopes[self.current_scope]
            .borrow()
            .parent
            .expect("exit_scope called on root scope");
    }

    fn declare(&mut self, name: &str, bind_type: BindType) -> usize {
        let mut scope = self.scopes[self.current_scope].borrow_mut();
        let scope_id = scope.id;
        let binding_id = self.next_binding_id;
        let symbol = Symbol {
            name: name.to_string(),
            binding_id,
            scope_id,
            bind_type,
        };

        scope.symbols.insert(name.to_string(), symbol.clone());
        self.bindings.insert(binding_id, symbol);

        // track which bindings belong to which scope
        self.scope_bindings
            .entry(scope_id)
            .or_insert(vec![])
            .push(binding_id);

        self.next_binding_id += 1;
        binding_id
    }

    fn lookup(&self, name: &str) -> Option<Symbol> {
        let current = self.scopes[self.current_scope].borrow();
        if let Some(sym) = current.symbols.get(name) {
            return Some(sym.clone());
        }
        let mut parent_idx = current.parent;
        while let Some(idx) = parent_idx {
            let scope = self.scopes[idx].borrow();
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym.clone());
            }
            parent_idx = scope.parent;
        }
        None
    }

    pub fn get_all_bindings_in_function_scope(&self, function_scope_id: usize) -> Vec<usize> {
        return self.collect_bindings_recursively(function_scope_id);
    }

    fn collect_bindings_recursively(&self, scope_id: usize) -> Vec<usize> {
        let mut bindings = self
            .scope_bindings
            .get(&scope_id)
            .cloned()
            .unwrap_or_else(Vec::new);

        let current_scope = match self.scopes.get(scope_id) {
            Some(value) => value.clone(),
            None => return bindings,
        };

        for child_scope_id in current_scope.borrow().children.iter() {
            let scope_type = match self.scopes.get(*child_scope_id) {
                Some(value) => value.borrow().scope_type.clone(),
                None => continue,
            };
            match scope_type {
                ScopeType::Block => {
                    bindings.extend(self.collect_bindings_recursively(*child_scope_id));
                }
                ScopeType::Function => {}
                ScopeType::Program => {}
            }
        }
        bindings
    }

    pub fn resolve(&self, id: i32) -> Option<usize> {
        self.resolved.get(&id).copied()
    }

    pub fn get_symbol(&self, binding_id: usize) -> Option<Symbol> {
        self.bindings.get(&binding_id).cloned()
    }

    pub fn get_bindings_in_scope(&self, scope_id: usize) -> &[usize] {
        self.scope_bindings
            .get(&scope_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_scope_for_node(&self, node_id: i32) -> Option<usize> {
        self.node_scope.get(&node_id).copied()
    }

    pub fn print_tree(&self) {
        if self.scopes.is_empty() {
            println!("(empty)");
            return;
        }
        let tree = self.build_tree(0);
        println!("{}", serde_json::to_string_pretty(&tree).unwrap());
    }

    fn build_tree(&self, scope_id: usize) -> ScopeTree {
        let scope = self.scopes[scope_id].borrow();
        let mut symbols: Vec<SymbolEntry> = scope
            .symbols
            .values()
            .map(|s| SymbolEntry {
                name: s.name.clone(),
                binding_id: s.binding_id,
            })
            .collect();
        symbols.sort_by_key(|s| s.binding_id);

        let children = self
            .scopes
            .iter()
            .filter_map(|s| {
                let s = s.borrow();
                if s.parent == Some(scope_id) {
                    Some(s.id)
                } else {
                    None
                }
            })
            .map(|id| self.build_tree(id))
            .collect();

        ScopeTree {
            id: scope_id,
            symbols,
            children,
        }
    }

    pub fn build(&mut self) {
        self.walk_node(self.root_node);
    }

    fn walk_node(&mut self, node: &Node) {
        match node {
            Node::ExpressionNode(expr) => self.walk_expression(expr),
            Node::StatementNode(stmt) => self.walk_statement(stmt),
        }
    }

    fn walk_statement(&mut self, node: &StatementNode) {
        match node {
            StatementNode::Let { name, value, .. } => {
                self.walk_expression(value);
                if let ExpressionNode::Identifier {
                    value: name_str,
                    id,
                    ..
                } = name
                {
                    let binding_id = self.declare(name_str, BindType::VariableDeclaraion);
                    self.resolved.insert(*id, binding_id);
                }
            }
            StatementNode::Expression { expression, .. } => {
                self.walk_expression(expression);
            }
            StatementNode::Block { statements, id, .. } => {
                let block_scope_id = self.enter_scope(ScopeType::Block);
                self.node_scope.insert(*id, block_scope_id);
                for stmt in statements {
                    self.walk_statement(stmt);
                }
                self.exit_scope();
            }
            StatementNode::Program { statements, id, .. } => {
                // program root enter scope here but dont leave scope
                let program_scope_id = self.enter_scope(ScopeType::Program);
                self.node_scope.insert(*id, program_scope_id);
                for stmt in statements {
                    self.walk_node(stmt);
                }
            }
            StatementNode::FuncDeclr {
                identifier, func, ..
            } => {
                if let ExpressionNode::Identifier { value, .. } = identifier {
                    self.declare(value, BindType::FunctionDeclaration);
                }
                self.walk_expression(func);
            }
            StatementNode::For { for_block, .. } => {
                self.walk_statement(for_block);
            }
            StatementNode::Return { return_value, .. } => {
                self.walk_expression(return_value);
            }
            _ => {}
        }
    }

    fn walk_expression(&mut self, node: &ExpressionNode) {
        match node {
            ExpressionNode::Identifier { value, id, .. } => {
                if let Some(sym) = self.lookup(value) {
                    self.resolved.insert(*id, sym.binding_id);
                }
            }
            ExpressionNode::Function {
                parameters,
                body,
                id,
                ..
            } => {
                let func_scope_id = self.enter_scope(ScopeType::Function);
                self.node_scope.insert(*id, func_scope_id);
                for param in parameters {
                    // todo: the block statement of the function
                    // is going to be in a different scope
                    if let ExpressionNode::Identifier { value, id, .. } = param.as_ref() {
                        let binding_id = self.declare(value, BindType::FunctionParam);
                        self.resolved.insert(*id, binding_id); // ← ADD THIS!
                    }
                }
                self.walk_statement(body);
                self.exit_scope();
            }
            ExpressionNode::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.walk_expression(condition);
                self.walk_expression(then_expr);
                self.walk_expression(else_expr);
            }
            ExpressionNode::Infix { left, right, .. } => {
                self.walk_expression(left);
                self.walk_expression(right);
            }
            ExpressionNode::Prefix { right, .. } => {
                self.walk_expression(right);
            }
            ExpressionNode::Call {
                function,
                arguments,
                ..
            } => {
                self.walk_expression(function);
                for arg in arguments {
                    self.walk_expression(arg);
                }
            }
            ExpressionNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                self.walk_expression(condition);
                self.walk_statement(if_block);
                if let Some(block) = else_block {
                    self.walk_statement(block);
                }
            }
            ExpressionNode::Index { object, index, .. } => {
                self.walk_expression(object);
                self.walk_expression(index);
            }
            ExpressionNode::Array { elements, .. } => {
                for el in elements {
                    self.walk_expression(el);
                }
            }
            ExpressionNode::HashMap { pairs, .. } => {
                for (k, v) in pairs {
                    self.walk_expression(k);
                    self.walk_expression(v);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn get_ast(input: &str) -> Result<Node, String> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    #[test]
    fn test_basic_let_binding_resolution() {
        let input = "
        let x = 1;
        let y = x + 2;
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        // x and y should be declared
        assert_eq!(symbol_table.bindings.len(), 2);

        let x_symbol = symbol_table.get_symbol(0).unwrap();
        assert_eq!(x_symbol.name, "x");
        assert_eq!(x_symbol.binding_id, 0);

        let y_symbol = symbol_table.get_symbol(1).unwrap();
        assert_eq!(y_symbol.name, "y");
        assert_eq!(y_symbol.binding_id, 1);
    }

    #[test]
    fn test_function_parameter_resolution() {
        let input = "
        func add(a, b) {
            return a + b;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        // add (0), a (1), b (2)
        assert_eq!(symbol_table.bindings.len(), 3);

        let a_symbol = symbol_table.get_symbol(1).unwrap();
        assert_eq!(a_symbol.name, "a");

        let b_symbol = symbol_table.get_symbol(2).unwrap();
        assert_eq!(b_symbol.name, "b");
    }

    #[test]
    fn test_nested_scope_resolution() {
        let input = "
        let x = 1;
        func outer() {
            let y = 2;
            func inner() {
                return x + y;
            };
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let x_symbol = symbol_table.get_symbol(0).unwrap();
        assert_eq!(x_symbol.name, "x");
        assert_eq!(x_symbol.scope_id, 0); // global scope

        let y_symbol = symbol_table.get_symbol(2).unwrap();
        assert_eq!(y_symbol.name, "y");
        assert!(y_symbol.scope_id > 0); // nested scope
    }

    #[test]
    fn test_scope_bindings_tracking() {
        let input = "
        let a = 1;
        let b = 2;
        func test() {
            let c = 3;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        // Scope 0 should have multiple bindings
        let scope_0_bindings = symbol_table.get_bindings_in_scope(0);
        assert!(scope_0_bindings.len() >= 3); // a, b, test

        // Multiple scopes should exist
        assert!(symbol_table.scope_bindings.len() > 1);
    }

    #[test]
    fn test_identifier_resolution_in_expression() {
        let input = "
        let x = 10;
        let y = x * 2;
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        // The resolved map should contain entries for identifier uses
        assert!(!symbol_table.resolved.is_empty());
    }

    #[test]
    fn test_get_all_bindings_in_function_scope_no_blocks() {
        let input = "
        func test() {
            let x = 1;
            let y = 2;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        // It is the only function so function_scope_id is 1
        let bindings = symbol_table.get_all_bindings_in_function_scope(1);

        // Should have x and y (no blocks, so just the function scope bindings)
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_get_all_bindings_in_function_scope_with_blocks() {
        let input = "
        func test() {
            let x = 1;
            if (true) {
                let y = 2;
            };
            let z = 3;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let bindings = symbol_table.get_all_bindings_in_function_scope(1);

        // Should have x, y, and z (y is in nested if block but should be collected)
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_get_all_bindings_excludes_nested_functions() {
        let input = "
        func outer() {
            let x = 1;
            func inner() {
                let y = 2;
            };
            let z = 3;
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let bindings = symbol_table.get_all_bindings_in_function_scope(1);

        // Should have x, inner (function name), and z
        // Should NOT have y (that's in nested function)
        // x=1, inner=2, z=3, so 3 bindings
        assert_eq!(bindings.len(), 3);

        // Verify y is NOT included
        let has_y = bindings.iter().any(|&bid| {
            symbol_table
                .get_symbol(bid)
                .map(|s| s.name == "y")
                .unwrap_or(false)
        });
        assert!(!has_y, "y should not be included (it's in nested function)");
    }

    #[test]
    fn test_get_all_bindings_nested_blocks() {
        let input = "
        func test() {
            let a = 1;
            if (true) {
                let b = 2;
                if (false) {
                    let c = 3;
                };
            };
        };
        ";

        let root = get_ast(input).unwrap();
        let mut symbol_table = SymbolTable::new(&root);
        symbol_table.build();

        let bindings = symbol_table.get_all_bindings_in_function_scope(1);

        // Should have a, b, and c (all nested blocks should be collected)
        assert_eq!(bindings.len(), 3);
    }
}
