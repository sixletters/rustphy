use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

/// Represents a lexical environment (scope) for variable bindings.
///
/// The environment implements lexical scoping using a parent chain.
/// Each environment can have a parent environment, forming a scope hierarchy
/// where inner scopes can access variables from outer scopes.
///
/// Uses `Rc<RefCell<>>` for shared ownership and interior mutability,
/// allowing multiple references to the same environment while still
/// being able to modify it.
#[derive(Debug)]
pub struct Environment {
    /// Optional reference to the parent environment for scope chaining.
    /// `None` indicates this is the global/root environment.
    pub parent: Option<Rc<RefCell<Environment>>>,

    /// Map of variable names to their values in this scope.
    pub values: HashMap<String, Value>,
}

/// Represents the possible runtime values in the Goophy VM.
///
/// This enum encapsulates all types of values that can be stored
/// in variables, passed as arguments, or used in computations.
#[derive(Clone, Debug)]
pub enum Value {
    /// A 128-bit signed integer value.
    Number(i128),

    /// A 64-bit floating-point value.
    Float(f64),

    /// A boolean value (true or false).
    Bool(bool),

    /// A string value.
    String(String),

    /// An identifier/variable name.
    Identifier(String),

    /// A symbolic value.
    Symbol(String),

    /// A closure (user-defined function) with captured environment.
    Closure {
        /// Parameter names for the closure.
        params: Vec<String>,
        /// Instruction address where the closure code begins.
        addr: usize,
        /// Captured environment (lexical scope) at closure creation time.
        env: Rc<RefCell<Environment>>,
    },

    /// A built-in function provided by the VM.
    Builtin {
        /// Name of the built-in function.
        name: BuiltinFn,
    },

    /// A variable that has been declared but not yet assigned a value.
    Unassigned,

    // Array for any value
    Array(Rc<RefCell<Vec<Value>>>),

    // Hashmaps, use string as hash first,
    HashMap(Rc<RefCell<HashMap<String, Value>>>),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum BuiltinFn {
    Print,
    PushArr,
    Len,
}

impl Value {
    /// Converts a Value to a serde_json::Value for serialization.
    ///
    /// This allows runtime values to be easily converted to JSON format
    /// for output, debugging, or transmission to JavaScript environments.
    ///
    /// # Returns
    /// A `serde_json::Value` representing this runtime value.
    ///
    /// # Examples
    /// ```
    /// use rust_impl::environment::Value;
    /// let val = Value::Number(42);
    /// let json = val.to_json_value();
    /// ```
    pub fn to_json_value(&self) -> serde_json::Value {
        use serde_json::json;

        match self {
            Value::Number(n) => {
                // i128 is too large for JSON number, convert to string if needed
                if *n >= i64::MIN as i128 && *n <= i64::MAX as i128 {
                    json!(*n as i64)
                } else {
                    json!(n.to_string())
                }
            }
            Value::Float(f) => json!(f),
            Value::Bool(b) => json!(b),
            Value::String(s) => json!(s),
            Value::Identifier(s) => json!(s),
            Value::Symbol(s) => json!({
                "type": "symbol",
                "value": s
            }),
            Value::Unassigned => json!(null),
            Value::Closure { params, addr, .. } => {
                json!({
                    "type": "closure",
                    "params": params,
                    "addr": addr
                })
            }
            Value::Array(arr) => {
                let items: Vec<serde_json::Value> =
                    arr.borrow().iter().map(|v| v.to_json_value()).collect();
                json!(items)
            }
            Value::HashMap(map) => {
                let obj: serde_json::Map<String, serde_json::Value> = map
                    .borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_json_value()))
                    .collect();
                json!(obj)
            }
            Value::Builtin { name } => {
                json!({
                    "type": "builtin",
                    "name": format!("{:?}", name)
                })
            }
        }
    }
}

impl Environment {
    /// Creates a new root environment with no parent.
    ///
    /// This is typically used for the global scope.
    ///
    /// # Returns
    /// A reference-counted, mutable environment.
    ///
    /// # Examples
    /// ```
    /// use rust_impl::environment::Environment;
    /// let env = Environment::new();
    /// ```
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            parent: None,
            values: HashMap::new(),
        }))
    }

    /// Creates a new child environment that extends the given parent.
    ///
    /// This is used to create nested scopes (e.g., function scopes, block scopes).
    /// The child environment can access variables from the parent, but variables
    /// declared in the child won't be visible to the parent.
    ///
    /// # Arguments
    /// * `parent` - The parent environment to extend from.
    ///
    /// # Returns
    /// A new environment with the specified parent.
    ///
    /// # Examples
    /// ```
    /// use rust_impl::environment::Environment;
    /// let global = Environment::new();
    /// let local = Environment::extend(global.clone());
    /// ```
    pub fn extend(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    /// Retrieves a value by name from this environment or its parent chain.
    ///
    /// Searches for the variable in the current scope first. If not found,
    /// recursively searches parent scopes until the variable is found or
    /// the root scope is reached.
    ///
    /// # Arguments
    /// * `name` - The variable name to look up.
    ///
    /// # Returns
    /// * `Some(Value)` if the variable is found in this scope or any parent scope.
    /// * `None` if the variable is not found in any scope.
    ///
    /// # Examples
    /// ```
    /// use rust_impl::environment::{Environment, Value};
    /// let env = Environment::new();
    /// env.borrow_mut().set_declare("x".to_string(), Value::Number(42));
    /// let value = env.borrow().get("x");
    /// ```
    pub fn get(&self, name: &str) -> Option<Value> {
        // Check current env if value exists, if not check parents recursively
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }

        // Check parent scopes recursively
        self.parent.as_ref().and_then(|p| p.borrow().get(name))
    }

    /// Declares a new variable in the current scope or updates an existing one.
    ///
    /// This always creates or updates the binding in the current scope,
    /// regardless of whether a variable with the same name exists in parent scopes.
    /// This implements variable shadowing.
    ///
    /// # Arguments
    /// * `name` - The variable name to declare.
    /// * `value` - The value to bind to the variable.
    ///
    /// # Examples
    /// ```
    /// use rust_impl::environment::{Environment, Value};
    /// let env = Environment::new();
    /// env.borrow_mut().set_declare("x".to_string(), Value::Number(10));
    /// ```
    pub fn set_declare(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// Assigns a value to an existing variable in this scope or parent scopes.
    ///
    /// Unlike `set_declare`, this will fail if the variable hasn't been declared.
    /// Searches for the variable in the current scope first, then recursively
    /// searches parent scopes. Updates the variable in the scope where it was
    /// first declared.
    ///
    /// # Arguments
    /// * `name` - The variable name to assign to.
    /// * `value` - The new value to assign.
    ///
    /// # Returns
    /// * `Ok(())` if the variable was found and updated.
    /// * `Err(String)` if the variable was not found in any scope.
    ///
    /// # Examples
    /// ```
    /// use rust_impl::environment::{Environment, Value};
    /// let env = Environment::new();
    /// env.borrow_mut().set_declare("x".to_string(), Value::Number(10));
    /// env.borrow_mut().set_assign("x", Value::Number(20)).unwrap();
    /// ```
    pub fn set_assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }

        // Otherwise search parents scope
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set_assign(name, value)
        } else {
            Err(format!("Variable '{}' not found", name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_environment() {
        let env = Environment::new();
        assert!(env.borrow().parent.is_none());
        assert!(env.borrow().values.is_empty());
    }

    #[test]
    fn test_extend_environment() {
        let parent = Environment::new();
        let child = Environment::extend(parent.clone());

        assert!(child.borrow().parent.is_some());
        assert!(child.borrow().values.is_empty());
    }

    #[test]
    fn test_set_declare_and_get() {
        let env = Environment::new();

        env.borrow_mut()
            .set_declare("x".to_string(), Value::Number(42));

        let value = env.borrow().get("x");
        assert!(value.is_some());

        if let Some(Value::Number(n)) = value {
            assert_eq!(n, 42);
        } else {
            panic!("Expected Number(42)");
        }
    }

    #[test]
    fn test_get_nonexistent_variable() {
        let env = Environment::new();
        let value = env.borrow().get("nonexistent");
        assert!(value.is_none());
    }

    #[test]
    fn test_get_from_parent_scope() {
        let parent = Environment::new();
        parent
            .borrow_mut()
            .set_declare("x".to_string(), Value::Number(100));

        let child = Environment::extend(parent.clone());

        let value = child.borrow().get("x");
        assert!(value.is_some());

        if let Some(Value::Number(n)) = value {
            assert_eq!(n, 100);
        } else {
            panic!("Expected Number(100)");
        }
    }

    #[test]
    fn test_shadowing() {
        let parent = Environment::new();
        parent
            .borrow_mut()
            .set_declare("x".to_string(), Value::Number(100));

        let child = Environment::extend(parent.clone());
        child
            .borrow_mut()
            .set_declare("x".to_string(), Value::Number(200));

        // Child should see its own value
        if let Some(Value::Number(n)) = child.borrow().get("x") {
            assert_eq!(n, 200);
        } else {
            panic!("Expected Number(200) in child");
        }

        // Parent should still have original value
        if let Some(Value::Number(n)) = parent.borrow().get("x") {
            assert_eq!(n, 100);
        } else {
            panic!("Expected Number(100) in parent");
        }
    }

    #[test]
    fn test_set_assign_local_variable() {
        let env = Environment::new();
        env.borrow_mut()
            .set_declare("x".to_string(), Value::Number(10));

        let result = env.borrow_mut().set_assign("x", Value::Number(20));
        assert!(result.is_ok());

        if let Some(Value::Number(n)) = env.borrow().get("x") {
            assert_eq!(n, 20);
        } else {
            panic!("Expected Number(20)");
        }
    }

    #[test]
    fn test_set_assign_parent_variable() {
        let parent = Environment::new();
        parent
            .borrow_mut()
            .set_declare("x".to_string(), Value::Number(10));

        let child = Environment::extend(parent.clone());

        let result = child.borrow_mut().set_assign("x", Value::Number(30));
        assert!(result.is_ok());

        // Parent should have updated value
        if let Some(Value::Number(n)) = parent.borrow().get("x") {
            assert_eq!(n, 30);
        } else {
            panic!("Expected Number(30) in parent");
        }

        // Child should also see updated value
        if let Some(Value::Number(n)) = child.borrow().get("x") {
            assert_eq!(n, 30);
        } else {
            panic!("Expected Number(30) in child");
        }
    }

    #[test]
    fn test_set_assign_undeclared_variable() {
        let env = Environment::new();
        let result = env.borrow_mut().set_assign("x", Value::Number(10));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Variable 'x' not found");
    }

    #[test]
    fn test_multiple_scope_levels() {
        let global = Environment::new();
        global
            .borrow_mut()
            .set_declare("global".to_string(), Value::Number(1));

        let middle = Environment::extend(global.clone());
        middle
            .borrow_mut()
            .set_declare("middle".to_string(), Value::Number(2));

        let inner = Environment::extend(middle.clone());
        inner
            .borrow_mut()
            .set_declare("inner".to_string(), Value::Number(3));

        // Inner can see all three
        assert!(matches!(
            inner.borrow().get("inner"),
            Some(Value::Number(3))
        ));
        assert!(matches!(
            inner.borrow().get("middle"),
            Some(Value::Number(2))
        ));
        assert!(matches!(
            inner.borrow().get("global"),
            Some(Value::Number(1))
        ));

        // Middle can see global and middle, but not inner
        assert!(inner.borrow().get("inner").is_some());
        assert!(matches!(
            middle.borrow().get("middle"),
            Some(Value::Number(2))
        ));
        assert!(matches!(
            middle.borrow().get("global"),
            Some(Value::Number(1))
        ));
        assert!(middle.borrow().get("inner").is_none());

        // Global can only see global
        assert!(matches!(
            global.borrow().get("global"),
            Some(Value::Number(1))
        ));
        assert!(global.borrow().get("middle").is_none());
        assert!(global.borrow().get("inner").is_none());
    }

    #[test]
    fn test_value_types() {
        let env = Environment::new();

        // Test Number
        env.borrow_mut()
            .set_declare("num".to_string(), Value::Number(42));
        assert!(matches!(env.borrow().get("num"), Some(Value::Number(42))));

        // Test Float
        env.borrow_mut()
            .set_declare("float".to_string(), Value::Float(3.14));
        assert!(
            matches!(env.borrow().get("float"), Some(Value::Float(f)) if (f - 3.14).abs() < f64::EPSILON)
        );

        // Test Bool
        env.borrow_mut()
            .set_declare("bool".to_string(), Value::Bool(true));
        assert!(matches!(env.borrow().get("bool"), Some(Value::Bool(true))));

        // Test Unassigned
        env.borrow_mut()
            .set_declare("unassigned".to_string(), Value::Unassigned);
        assert!(matches!(
            env.borrow().get("unassigned"),
            Some(Value::Unassigned)
        ));

        // Test Builtin
        env.borrow_mut().set_declare(
            "builtin".to_string(),
            Value::Builtin {
                name: BuiltinFn::Print,
            },
        );
        if let Some(Value::Builtin { name }) = env.borrow().get("builtin") {
            assert_eq!(name, BuiltinFn::Print);
        } else {
            panic!("Expected Builtin value");
        }
    }

    #[test]
    fn test_closure_value() {
        let env = Environment::new();
        let closure_env = Environment::new();

        let closure = Value::Closure {
            params: vec!["x".to_string(), "y".to_string()],
            addr: 100,
            env: closure_env.clone(),
        };

        env.borrow_mut().set_declare("func".to_string(), closure);

        if let Some(Value::Closure { params, addr, .. }) = env.borrow().get("func") {
            assert_eq!(params, vec!["x".to_string(), "y".to_string()]);
            assert_eq!(addr, 100);
        } else {
            panic!("Expected Closure value");
        }
    }

    #[test]
    fn test_override_in_same_scope() {
        let env = Environment::new();

        env.borrow_mut()
            .set_declare("x".to_string(), Value::Number(10));
        env.borrow_mut()
            .set_declare("x".to_string(), Value::Number(20));

        if let Some(Value::Number(n)) = env.borrow().get("x") {
            assert_eq!(n, 20);
        } else {
            panic!("Expected Number(20)");
        }
    }
}
