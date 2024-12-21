use std::collections::HashMap;
use crate::parser::AstKind;

pub struct Evaluator {
    pub variables: HashMap<String, i32>,
    scripts: HashMap<String, AstKind>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scripts: HashMap::new(),
        }
    }

    pub fn register_script(&mut self, name: String, ast: AstKind) {
        self.scripts.insert(name, ast);
    }

    pub fn eval(&mut self, ast: &AstKind) -> i32 {
        match ast {
            AstKind::NumericLiteral(n) => *n,
            AstKind::StringLiteral(_) => 0,

            AstKind::LocalVar(name) => {
                let var_name = name.trim_start_matches('$');
                self.variables.get(var_name).unwrap_or(&0).clone()
            },

            AstKind::BinaryExpression { lhs, rhs, operator } => {
                let left = self.eval(lhs);
                let right = self.eval(rhs);
                match operator.as_str() {
                    "+" => left + right,
                    "-" => left - right,
                    "*" => left * right,
                    "/" => left / right,
                    "<=" => if left <= right { 1 } else { 0 },
                    ">=" => if left >= right { 1 } else { 0 },
                    "<" => if left < right { 1 } else { 0 },
                    ">" => if left > right { 1 } else { 0 },
                    "=" => if left == right { 1 } else { 0 },
                    _ => panic!("Unknown operator: {}", operator),
                }
            },

            AstKind::Assignment { target, value } => {
                if let AstKind::LocalVar(name) = &**target {
                    let var_name = name.trim_start_matches('$');
                    let val = self.eval(value);
                    self.variables.insert(var_name.to_string(), val);
                    val
                } else {
                    panic!("Invalid assignment target");
                }
            },

            AstKind::Define { name, var_type: _, value } => {
                let val = self.eval(value);
                let var_name = name.trim_start_matches('$');
                self.variables.insert(var_name.to_string(), val);
                val
            },

            AstKind::If { expression, value, return_statement } => {
                let condition = self.eval(expression);
                if condition != 0 {
                    self.eval(return_statement)
                } else {
                    self.eval(value)
                }
            },

            AstKind::While { condition, body } => {
                let mut last_value = 0;
                while self.eval(condition) != 0 {
                    last_value = self.eval(body);
                }
                last_value
            },

            AstKind::Block(statements) => {
                let mut last_value = 0;
                for stmt in statements {
                    match stmt {
                        AstKind::Return(expr) => return self.eval(expr),
                        AstKind::If { .. } => {
                            let result = self.eval(stmt);
                            if result != 0 {
                                return result;
                            }
                        },
                        _ => { last_value = self.eval(stmt); }
                    }
                }
                last_value
            },

            AstKind::Return(expr) => {
                self.eval(expr)
            },

            AstKind::FunctionCall { name, arguments } => {
                match name.as_str() {
                    "calc" => {
                        if let Some(arg) = arguments.first() {
                            self.eval(arg)
                        } else {
                            panic!("calc requires one argument");
                        }
                    },
                    _ => panic!("Unknown function: {}", name),
                }
            },

            AstKind::ScriptCall { script, arguments } => {
                if let AstKind::Identifier(script_name) = &**script {
                    let mut arg_values = Vec::new();
                    for arg in arguments {
                        arg_values.push(self.eval(arg));
                    }
                    self.eval_script(script_name, &arg_values)
                } else {
                    panic!("Invalid script call target");
                }
            },

            AstKind::Trigger { body, .. } => {
                self.eval(body)
            },

            _ => 0,
        }
    }

    pub fn eval_script(&mut self, name: &str, args: &[i32]) -> i32 {
        let script = if let Some(s) = self.scripts.get(name) {
            s.clone()
        } else {
            panic!("Script not found: {}", name);
        };

        let old_vars = self.variables.clone();
        self.variables.clear();
        
        if let AstKind::Trigger { args: script_args, .. } = &script {
            // Zip parameter names with argument values and insert into variables
            for (param, &value) in script_args.iter()
                .filter_map(|arg| if let AstKind::LocalVar(name) = &**arg {
                    Some(name.trim_start_matches('$'))
                } else {
                    None
                })
                .zip(args.iter()) {
                self.variables.insert(param.to_string(), value);
            }
        }
        
        let result = match &script {
            AstKind::Trigger { body, .. } => self.eval(body),
            _ => self.eval(&script),
        };
        self.variables = old_vars;
        result
    }
} 