use crate::ast::{ExpressionNode, Node, StatementNode};
use serde::Serialize;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone)]
pub struct Scope {
    id: usize,
    parent: Option<usize>,
    symbols: HashMap<String, Symbol>,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    name: String,
    binding_id: usize,
    scope_id: usize,
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
        };
        table
    }

    fn enter_scope(&mut self) -> usize {
        let parent = (!self.scopes.is_empty()).then_some(self.current_scope);
        let id = self.next_scope_id;
        self.scopes.push(Rc::new(RefCell::new(Scope {
            parent,
            id,
            symbols: HashMap::new(),
        })));
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

    fn declare(&mut self, name: &str) -> usize {
        let mut scope = self.scopes[self.current_scope].borrow_mut();
        let scope_id = scope.id;
        let binding_id = self.next_binding_id;
        scope.symbols.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                binding_id,
                scope_id,
            },
        );
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

    pub fn resolve(&self, id: i32) -> Option<usize> {
        println!("{:?}", self.resolved);
        self.resolved.get(&id).copied()
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
                    value: name_str, ..
                } = name
                {
                    self.declare(name_str);
                }
            }
            StatementNode::Expression { expression, .. } => {
                self.walk_expression(expression);
            }
            StatementNode::Block { statements, .. } => {
                self.enter_scope();
                for stmt in statements {
                    self.walk_statement(stmt);
                }
                self.exit_scope();
            }
            StatementNode::Program { statements, .. } => {
                // program root enter scope here but dont leave scope
                self.enter_scope();
                for stmt in statements {
                    self.walk_node(stmt);
                }
            }
            StatementNode::FuncDeclr {
                identifier, func, ..
            } => {
                if let ExpressionNode::Identifier { value, .. } = identifier {
                    self.declare(value);
                }
                self.walk_expression(func);
            }
            StatementNode::For { for_block, .. } => {
                self.enter_scope();
                self.walk_statement(for_block);
                self.exit_scope();
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
                parameters, body, ..
            } => {
                self.enter_scope();
                for param in parameters {
                    if let ExpressionNode::Identifier { value, .. } = param.as_ref() {
                        self.declare(value);
                    }
                }
                self.walk_statement(body);
                self.exit_scope();
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
