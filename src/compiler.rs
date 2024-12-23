use crate::bytecode::{ByteCode, Instruction};
use crate::parser::AstKind;
use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug)]
enum RecursivePattern {
    SingleRecursive {
        operation: String,
        param_expr: Option<Box<AstKind>>,
    },
    DoubleRecursive {
        operation: String,
    },
}

pub struct Compiler {
    scripts: HashMap<String, ByteCode>,
    current_script: Option<String>,  // Track the current script being compiled
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            current_script: None,
        }
    }

    pub fn compile_script(&mut self, name: String, ast: &AstKind) -> ByteCode {
        let mut bytecode = ByteCode::new(name.clone());
        
        // Set current script name
        self.current_script = Some(name.clone());
        
        match ast {
            AstKind::Trigger { body, args, .. } => {
                // Initialize arguments
                let mut arg_index = 0;
                let mut param_name = None;
                for arg in args.iter().skip(1).step_by(2) {  // Skip type nodes and get variable names
                    if let AstKind::LocalVar(name) = &**arg {
                        let var_name = name.trim_start_matches('$');
                        bytecode.push(Instruction::PushIntLocal(format!("arg{}", arg_index)));
                        bytecode.push(Instruction::PopIntLocal(var_name.to_string()));
                        if param_name.is_none() {
                            param_name = Some(var_name.to_string());
                        }
                        arg_index += 1;
                    }
                }
                
                // Check if this is a recursive function and transform it if needed
                let transformed_body = if let Some(param) = param_name {
                    println!("Found parameter '{}' from procedure declaration", param);
                    self.transform_recursive_to_iterative_with_param(body, param)
                } else {
                    println!("No parameter found in procedure declaration");
                    (**body).clone()
                };
                
                self.compile_node(&transformed_body, &mut bytecode);
                
                // Only add Return if the last instruction isn't already a Return
                if !matches!(bytecode.instructions.last(), Some(Instruction::Return)) {
                    bytecode.push(Instruction::Return);
                }
            }
            _ => {
                self.compile_node(ast, &mut bytecode);
                if !matches!(bytecode.instructions.last(), Some(Instruction::Return)) {
                    bytecode.push(Instruction::Return);
                }
            }
        }
        
        // Clear current script name
        self.current_script = None;
        
        self.scripts.insert(name, bytecode.clone());
        bytecode
    }

    fn transform_recursive_to_iterative_with_param(&self, node: &AstKind, param_name: String) -> AstKind {
        match node {
            AstKind::Block(statements) => {
                println!("Analyzing block for recursive pattern...");
                
                // Get the current script name
                let current_script = if let Some(name) = &self.current_script {
                    println!("Current script: {}", name);
                    name.clone()
                } else {
                    println!("No current script name found, skipping transformation");
                    return node.clone();
                };
                
                // Find base cases and recursive expression
                let mut base_cases = Vec::new();
                let mut recursive_expr = None;
                println!("Starting analysis of recursive function...");

                // Collect base cases and find recursive expression
                for stmt in statements {
                    match stmt {
                        AstKind::If { expression, value: _, return_statement } => {
                            println!("Found base case condition");
                            base_cases.push(stmt.clone());
                        }
                        AstKind::Return(expr) => {
                            if self.contains_recursive_call(expr) {
                                println!("Found recursive expression in return statement");
                                recursive_expr = Some(Box::new(expr.as_ref().clone()));
                            }
                        }
                        _ => {}
                    }
                }

                println!("Found {} base case(s)", base_cases.len());
                if recursive_expr.is_none() || base_cases.is_empty() {
                    println!("No recursion or base cases found, skipping transformation");
                    return node.clone();
                }

                println!("Starting transformation to iterative form...");
                let mut new_statements = Vec::new();

                // Initialize variables for iterative version
                let mut has_base_cases = false;
                for base_case in &base_cases {
                    if let AstKind::If { expression, value: _, return_statement } = base_case {
                        has_base_cases = true;
                        new_statements.push(base_case.clone());
                    }
                }

                if !has_base_cases {
                    return node.clone();
                }

                // Analyze recursive expression
                if let Some(expr) = &recursive_expr {
                    // Check if this is a tail recursive function
                    let is_tail_recursive = match &**expr {
                        AstKind::ScriptCall { script, arguments } => {
                            if let AstKind::Identifier(name) = &**script {
                                println!("Analyzing potential tail recursive call to: {}", name);
                                println!("Current script: {}", current_script);
                                println!("Number of arguments: {}", arguments.len());
                                
                                let is_tail = name == &current_script && arguments.len() == 2;
                                if is_tail {
                                    println!("Found tail recursive call with accumulator");
                                    println!("Arguments:");
                                    for (i, arg) in arguments.iter().enumerate() {
                                        println!("  Arg {}: {:?}", i, arg);
                                    }
                                } else {
                                    println!("Not a tail recursive call because:");
                                    if name != &current_script {
                                        println!("  - Call is to different function: {} != {}", name, current_script);
                                    }
                                    if arguments.len() != 2 {
                                        println!("  - Wrong number of arguments: {} (expected 2)", arguments.len());
                                    }
                                }
                                is_tail
                            } else {
                                println!("Not a tail recursive call - script is not an identifier");
                                false
                            }
                        },
                        _ => {
                            println!("Not a tail recursive call - expression is not a script call");
                            false
                        }
                    };

                    if is_tail_recursive {
                        println!("Found tail recursive pattern");
                        println!("Transforming to iterative form with accumulator...");
                        let mut new_statements = Vec::new();

                        // Initialize n with first argument
                        println!("Initializing n with first argument (arg0)");
                        new_statements.push(AstKind::Define {
                            name: "n".to_string(),
                            var_type: Type::Int,
                            value: Box::new(AstKind::LocalVar("arg0".to_string())),
                        });

                        // Initialize acc with second argument
                        println!("Initializing acc with second argument (arg1/accumulator)");
                        new_statements.push(AstKind::Define {
                            name: "acc".to_string(),
                            var_type: Type::Int,
                            value: Box::new(AstKind::LocalVar("arg1".to_string())),
                        });

                        // Add base case check
                        println!("Adding base case check for n <= 1");
                        new_statements.push(AstKind::If {
                            expression: Box::new(AstKind::BinaryExpression {
                                lhs: Box::new(AstKind::LocalVar("n".to_string())),
                                rhs: Box::new(AstKind::NumericLiteral(1)),
                                operator: "<=".to_string(),
                            }),
                            value: Box::new(AstKind::LocalVar("acc".to_string())),
                            return_statement: Box::new(AstKind::Return(Box::new(AstKind::LocalVar("acc".to_string())))),
                        });

                        // Create while loop condition: while n > 1
                        let loop_condition = AstKind::BinaryExpression {
                            lhs: Box::new(AstKind::LocalVar("n".to_string())),
                            rhs: Box::new(AstKind::NumericLiteral(1)),
                            operator: ">".to_string(),
                        };

                        let mut loop_body = Vec::new();

                        // Update accumulator: acc = n * acc
                        loop_body.push(AstKind::Assignment {
                            target: Box::new(AstKind::LocalVar("acc".to_string())),
                            value: Box::new(AstKind::FunctionCall {
                                name: "calc".to_string(),
                                arguments: vec![Box::new(AstKind::BinaryExpression {
                                    lhs: Box::new(AstKind::LocalVar("n".to_string())),
                                    rhs: Box::new(AstKind::LocalVar("acc".to_string())),
                                    operator: "*".to_string(),
                                })],
                            }),
                        });

                        // Decrement n: n = n - 1
                        loop_body.push(AstKind::Assignment {
                            target: Box::new(AstKind::LocalVar("n".to_string())),
                            value: Box::new(AstKind::FunctionCall {
                                name: "calc".to_string(),
                                arguments: vec![Box::new(AstKind::BinaryExpression {
                                    lhs: Box::new(AstKind::LocalVar("n".to_string())),
                                    rhs: Box::new(AstKind::NumericLiteral(1)),
                                    operator: "-".to_string(),
                                })],
                            }),
                        });

                        // Add the while loop
                        new_statements.push(AstKind::While {
                            condition: Box::new(loop_condition),
                            body: Box::new(AstKind::Block(loop_body)),
                        });

                        // Return final accumulator value
                        new_statements.push(AstKind::Return(Box::new(AstKind::LocalVar("acc".to_string()))));

                        println!("Tail recursion transformation complete");
                        return AstKind::Block(new_statements);
                    }

                    // Count recursive calls
                    fn count_recursive_calls(node: &AstKind, script_name: &str) -> i32 {
                        match node {
                            AstKind::ScriptCall { script, .. } => {
                                if let AstKind::Identifier(name) = &**script {
                                    if name == script_name {
                                        return 1;
                                    }
                                }
                                0
                            },
                            AstKind::FunctionCall { name: _, arguments } => {
                                arguments.iter().map(|arg| count_recursive_calls(arg, script_name)).sum()
                            },
                            AstKind::BinaryExpression { lhs, rhs, operator: _ } => {
                                count_recursive_calls(lhs, script_name) + count_recursive_calls(rhs, script_name)
                            },
                            _ => 0,
                        }
                    }

                    let recursive_calls = count_recursive_calls(expr, &current_script);
                    println!("Found {} recursive call(s) in expression", recursive_calls);

                    // Check for nested recursion
                    fn has_nested_recursion(node: &AstKind, script_name: &str) -> bool {
                        match node {
                            AstKind::ScriptCall { script, arguments } => {
                                if let AstKind::Identifier(name) = &**script {
                                    if name == script_name {
                                        // Check if any argument contains a recursive call
                                        arguments.iter().any(|arg| has_nested_recursion(arg, script_name))
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            },
                            AstKind::FunctionCall { arguments, .. } => {
                                arguments.iter().any(|arg| has_nested_recursion(arg, script_name))
                            },
                            AstKind::BinaryExpression { lhs, rhs, .. } => {
                                has_nested_recursion(lhs, script_name) || has_nested_recursion(rhs, script_name)
                            },
                            _ => false,
                        }
                    }

                    if has_nested_recursion(expr, &current_script) {
                        println!("Found nested recursion pattern, skipping transformation");
                        return node.clone();
                    }

                    match recursive_calls {
                        1 => {
                            println!("Analyzing single recursive call pattern...");
                            println!("Analyzing recursive pattern to determine initial value...");
                            
                            // Extract base case return value
                            let base_case_value = if let Some(base_case) = base_cases.first() {
                                if let AstKind::If { return_statement, .. } = base_case {
                                    if let AstKind::Return(expr) = &**return_statement {
                                        if let AstKind::NumericLiteral(n) = &**expr {
                                            *n
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            } else {
                                0
                            };
                            
                            // Single recursive call (factorial, power, sum_to_n)
                            println!("Initializing result variable for single recursion...");
                            new_statements.push(AstKind::Define {
                                name: "result".to_string(),
                                var_type: Type::Int,
                                value: Box::new(AstKind::NumericLiteral(base_case_value)),
                            });
                            println!("Initialized result variable with base case value: {}", base_case_value);

                            new_statements.push(AstKind::Define {
                                name: "i".to_string(),
                                var_type: Type::Int,
                                value: Box::new(AstKind::NumericLiteral(1)),
                            });
                            println!("Initialized counter variable with 1");

                            // Create while loop condition
                            println!("Creating loop condition with parameter: {}", param_name);
                            let loop_condition = AstKind::BinaryExpression {
                                lhs: Box::new(AstKind::LocalVar("i".to_string())),
                                rhs: Box::new(AstKind::LocalVar(param_name.clone())),
                                operator: "<=".to_string(),
                            };

                            // Create loop body
                            println!("Building loop body for iterative transformation...");
                            let mut loop_body = Vec::new();

                            // Extract operation from recursive expression
                            if let AstKind::FunctionCall { name, arguments } = expr.as_ref() {
                                if name == "calc" {
                                    if let Some(arg) = arguments.first() {
                                        if let AstKind::BinaryExpression { operator, .. } = &**arg {
                                            println!("Found operation '{}' in recursive expression", operator);
                                            // Update result based on operation
                                            match operator.as_str() {
                                                "*" => {
                                                    println!("Applying multiplication in loop body");
                                                    // For factorial: result = result * i
                                                    loop_body.push(AstKind::Assignment {
                                                        target: Box::new(AstKind::LocalVar("result".to_string())),
                                                        value: Box::new(AstKind::FunctionCall {
                                                            name: "calc".to_string(),
                                                            arguments: vec![Box::new(AstKind::BinaryExpression {
                                                                lhs: Box::new(AstKind::LocalVar("result".to_string())),
                                                                rhs: Box::new(AstKind::LocalVar("i".to_string())),
                                                                operator: "*".to_string(),
                                                            })],
                                                        }),
                                                    });
                                                    println!("Added multiplication: result = result * i");
                                                },
                                                "+" => {
                                                    // For sum_to_n: result = result + i
                                                    loop_body.push(AstKind::Assignment {
                                                        target: Box::new(AstKind::LocalVar("result".to_string())),
                                                        value: Box::new(AstKind::FunctionCall {
                                                            name: "calc".to_string(),
                                                            arguments: vec![Box::new(AstKind::BinaryExpression {
                                                                lhs: Box::new(AstKind::LocalVar("result".to_string())),
                                                                rhs: Box::new(AstKind::LocalVar("i".to_string())),
                                                                operator: "+".to_string(),
                                                            })],
                                                        }),
                                                    });
                                                },
                                                _ => {
                                                    // For other operations, use the original operator
                                                    loop_body.push(AstKind::Assignment {
                                                        target: Box::new(AstKind::LocalVar("result".to_string())),
                                                        value: Box::new(AstKind::FunctionCall {
                                                            name: "calc".to_string(),
                                                            arguments: vec![Box::new(AstKind::BinaryExpression {
                                                                lhs: Box::new(AstKind::LocalVar("result".to_string())),
                                                                rhs: Box::new(AstKind::LocalVar("i".to_string())),
                                                                operator: operator.clone(),
                                                            })],
                                                        }),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Increment counter
                            loop_body.push(AstKind::Assignment {
                                target: Box::new(AstKind::LocalVar("i".to_string())),
                                value: Box::new(AstKind::FunctionCall {
                                    name: "calc".to_string(),
                                    arguments: vec![Box::new(AstKind::BinaryExpression {
                                        lhs: Box::new(AstKind::LocalVar("i".to_string())),
                                        rhs: Box::new(AstKind::NumericLiteral(1)),
                                        operator: "+".to_string(),
                                    })],
                                }),
                            });

                            // Add the while loop
                            new_statements.push(AstKind::While {
                                condition: Box::new(loop_condition),
                                body: Box::new(AstKind::Block(loop_body)),
                            });

                            // Return final result
                            new_statements.push(AstKind::Return(Box::new(AstKind::LocalVar("result".to_string()))));
                        },
                        2 => {
                            // Double recursive call (Fibonacci)
                            // Handle base cases first
                            println!("WERE INSIDE DOUBLE RECURSIVE");
                            new_statements.push(AstKind::If {
                                expression: Box::new(AstKind::BinaryExpression {
                                    lhs: Box::new(AstKind::LocalVar(param_name.clone())),
                                    rhs: Box::new(AstKind::NumericLiteral(0)),
                                    operator: "=".to_string(),
                                }),
                                value: Box::new(AstKind::NumericLiteral(0)),
                                return_statement: Box::new(AstKind::Return(Box::new(AstKind::NumericLiteral(0)))),
                            });

                            new_statements.push(AstKind::If {
                                expression: Box::new(AstKind::BinaryExpression {
                                    lhs: Box::new(AstKind::LocalVar(param_name.clone())),
                                    rhs: Box::new(AstKind::NumericLiteral(1)),
                                    operator: "=".to_string(),
                                }),
                                value: Box::new(AstKind::NumericLiteral(1)),
                                return_statement: Box::new(AstKind::Return(Box::new(AstKind::NumericLiteral(1)))),
                            });

                            new_statements.push(AstKind::If {
                                expression: Box::new(AstKind::BinaryExpression {
                                    lhs: Box::new(AstKind::LocalVar(param_name.clone())),
                                    rhs: Box::new(AstKind::NumericLiteral(2)),
                                    operator: "=".to_string(),
                                }),
                                value: Box::new(AstKind::NumericLiteral(1)),
                                return_statement: Box::new(AstKind::Return(Box::new(AstKind::NumericLiteral(1)))),
                            });

                            // Initialize variables for iterative version
                            new_statements.push(AstKind::Define {
                                name: "prev".to_string(),
                                var_type: Type::Int,
                                value: Box::new(AstKind::NumericLiteral(0)),  // Start with fib(0)
                            });

                            new_statements.push(AstKind::Define {
                                name: "curr".to_string(),
                                var_type: Type::Int,
                                value: Box::new(AstKind::NumericLiteral(1)),  // Start with fib(1)
                            });

                            new_statements.push(AstKind::Define {
                                name: "next".to_string(),
                                var_type: Type::Int,
                                value: Box::new(AstKind::NumericLiteral(1)),  // Will be calculated
                            });

                            new_statements.push(AstKind::Define {
                                name: "i".to_string(),
                                var_type: Type::Int,
                                value: Box::new(AstKind::NumericLiteral(2)),  // Start from 2 since we handle 0,1 in base cases
                            });

                            // Create the loop
                            new_statements.push(AstKind::While {
                                condition: Box::new(AstKind::BinaryExpression {
                                    lhs: Box::new(AstKind::LocalVar("i".to_string())),
                                    rhs: Box::new(AstKind::LocalVar(param_name.clone())),
                                    operator: "<=".to_string(),
                                }),
                                body: Box::new(AstKind::Block(vec![
                                    // next = prev + curr
                                    AstKind::Assignment {
                                        target: Box::new(AstKind::LocalVar("next".to_string())),
                                        value: Box::new(AstKind::FunctionCall {
                                            name: "calc".to_string(),
                                            arguments: vec![Box::new(AstKind::BinaryExpression {
                                                lhs: Box::new(AstKind::LocalVar("prev".to_string())),
                                                rhs: Box::new(AstKind::LocalVar("curr".to_string())),
                                                operator: "+".to_string(),
                                            })],
                                        }),
                                    },
                                    // prev = curr
                                    AstKind::Assignment {
                                        target: Box::new(AstKind::LocalVar("prev".to_string())),
                                        value: Box::new(AstKind::LocalVar("curr".to_string())),
                                    },
                                    // curr = next
                                    AstKind::Assignment {
                                        target: Box::new(AstKind::LocalVar("curr".to_string())),
                                        value: Box::new(AstKind::LocalVar("next".to_string())),
                                    },
                                    // i = i + 1
                                    AstKind::Assignment {
                                        target: Box::new(AstKind::LocalVar("i".to_string())),
                                        value: Box::new(AstKind::FunctionCall {
                                            name: "calc".to_string(),
                                            arguments: vec![Box::new(AstKind::BinaryExpression {
                                                lhs: Box::new(AstKind::LocalVar("i".to_string())),
                                                rhs: Box::new(AstKind::NumericLiteral(1)),
                                                operator: "+".to_string(),
                                            })],
                                        }),
                                    },
                                ])),
                            });

                            // Return the final value
                            new_statements.push(AstKind::Return(Box::new(AstKind::LocalVar("curr".to_string()))));
                        },
                        _ => {
                            // Unsupported recursive pattern
                            return node.clone();
                        }
                    }
                } else {
                    return node.clone();
                }

                println!("Transformation complete.");
                AstKind::Block(new_statements)
            }
            _ => node.clone(),
        }
    }

    fn analyze_recursive_pattern(&self, expr: &AstKind, script_name: &str, param_name: &str) -> Option<RecursivePattern> {
        match expr {
            AstKind::FunctionCall { name, arguments } => {
                if name == "calc" {
                    if let Some(arg) = arguments.first() {
                        if let AstKind::BinaryExpression { lhs, rhs, operator } = &**arg {
                            // Check for double recursion (Fibonacci-style)
                            let mut recursive_calls = 0;
                            
                            fn count_recursive_calls(node: &AstKind, script_name: &str) -> i32 {
                                match node {
                                    AstKind::ScriptCall { script, .. } => {
                                        if let AstKind::Identifier(name) = &**script {
                                            if name == script_name {
                                                return 1;
                                            }
                                        }
                                        0
                                    },
                                    AstKind::FunctionCall { arguments, .. } => {
                                        arguments.iter().map(|arg| count_recursive_calls(arg, script_name)).sum()
                                    },
                                    AstKind::BinaryExpression { lhs, rhs, .. } => {
                                        count_recursive_calls(lhs, script_name) + count_recursive_calls(rhs, script_name)
                                    },
                                    _ => 0,
                                }
                            }

                            recursive_calls = count_recursive_calls(lhs, script_name) + count_recursive_calls(rhs, script_name);

                            if recursive_calls == 2 {
                                return Some(RecursivePattern::DoubleRecursive {
                                    operation: operator.clone(),
                                });
                            } else if recursive_calls == 1 {
                                // Analyze parameter modification
                                fn extract_param_expr(node: &AstKind, param_name: &str) -> Option<Box<AstKind>> {
                                    match node {
                                        AstKind::FunctionCall { name, arguments } => {
                                            if name == "calc" {
                                                if let Some(arg) = arguments.first() {
                                                    if let AstKind::BinaryExpression { lhs, rhs, .. } = &**arg {
                                                        if let AstKind::LocalVar(var_name) = &**lhs {
                                                            if var_name.trim_start_matches('$') == param_name {
                                                                return Some(rhs.clone());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            None
                                        },
                                        _ => None,
                                    }
                                }

                                let param_expr = extract_param_expr(expr, param_name);
                                return Some(RecursivePattern::SingleRecursive {
                                    operation: operator.clone(),
                                    param_expr,
                                });
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        None
    }

    fn contains_recursive_call(&self, node: &AstKind) -> bool {
        match node {
            AstKind::ScriptCall { script, .. } => {
                if let AstKind::Identifier(script_name) = &**script {
                    if let Some(current_script) = &self.current_script {
                        return script_name == current_script;
                    }
                }
                false
            }
            AstKind::Block(statements) => {
                statements.iter().any(|stmt| self.contains_recursive_call(stmt))
            }
            AstKind::If { expression, value, return_statement } => {
                self.contains_recursive_call(expression) ||
                self.contains_recursive_call(value) ||
                self.contains_recursive_call(return_statement)
            }
            AstKind::While { condition, body } => {
                self.contains_recursive_call(condition) ||
                self.contains_recursive_call(body)
            }
            AstKind::Return(expr) => self.contains_recursive_call(expr),
            AstKind::Assignment { target, value } => {
                self.contains_recursive_call(target) ||
                self.contains_recursive_call(value)
            }
            AstKind::Define { value, .. } => self.contains_recursive_call(value),
            AstKind::BinaryExpression { lhs, rhs, .. } => {
                self.contains_recursive_call(lhs) ||
                self.contains_recursive_call(rhs)
            }
            AstKind::FunctionCall { arguments, .. } => {
                arguments.iter().any(|arg| self.contains_recursive_call(arg))
            }
            _ => false,
        }
    }

    fn compile_node(&mut self, node: &AstKind, bytecode: &mut ByteCode) {
        match node {
            AstKind::NumericLiteral(n) => {
                bytecode.push(Instruction::PushConstantInt(*n));
            }
            
            AstKind::StringLiteral(s) => {
                bytecode.push(Instruction::PushConstantString(s.clone()));
            }
            
            AstKind::LocalVar(name) => {
                let var_name = name.trim_start_matches('$');
                bytecode.push(Instruction::PushIntLocal(var_name.to_string()));
            }
            
            AstKind::BinaryExpression { lhs, rhs, operator } => {
                // Compile left and right operands
                self.compile_node(lhs, bytecode);
                self.compile_node(rhs, bytecode);
                
                // Add appropriate comparison instruction
                match operator.as_str() {
                    "=" => {
                        bytecode.push(Instruction::BranchEquals(bytecode.instructions.len() + 3));
                        bytecode.push(Instruction::PushConstantInt(0));
                        bytecode.push(Instruction::Jump(bytecode.instructions.len() + 2));
                        bytecode.push(Instruction::PushConstantInt(1));
                    },
                    "<" => {
                        bytecode.push(Instruction::BranchLessThan(bytecode.instructions.len() + 3));
                        bytecode.push(Instruction::PushConstantInt(0));
                        bytecode.push(Instruction::Jump(bytecode.instructions.len() + 2));
                        bytecode.push(Instruction::PushConstantInt(1));
                    },
                    "<=" => {
                        bytecode.push(Instruction::BranchLessThanOrEquals(bytecode.instructions.len() + 3));
                        bytecode.push(Instruction::PushConstantInt(0));
                        bytecode.push(Instruction::Jump(bytecode.instructions.len() + 2));
                        bytecode.push(Instruction::PushConstantInt(1));
                    },
                    ">" => {
                        bytecode.push(Instruction::BranchGreaterThan(bytecode.instructions.len() + 3));
                        bytecode.push(Instruction::PushConstantInt(0));
                        bytecode.push(Instruction::Jump(bytecode.instructions.len() + 2));
                        bytecode.push(Instruction::PushConstantInt(1));
                    },
                    ">=" => {
                        bytecode.push(Instruction::BranchGreaterThanOrEquals(bytecode.instructions.len() + 3));
                        bytecode.push(Instruction::PushConstantInt(0));
                        bytecode.push(Instruction::Jump(bytecode.instructions.len() + 2));
                        bytecode.push(Instruction::PushConstantInt(1));
                    },
                    "+" => bytecode.push(Instruction::Add),
                    "-" => bytecode.push(Instruction::Subtract),
                    "*" => bytecode.push(Instruction::Multiply),
                    _ => panic!("Unsupported operator: {}", operator),
                }
            }
            
            AstKind::Assignment { target, value } => {
                self.compile_node(value, bytecode);
                if let AstKind::LocalVar(name) = &**target {
                    let var_name = name.trim_start_matches('$');
                    bytecode.push(Instruction::PopIntLocal(var_name.to_string()));
                }
            }
            
            AstKind::Define { name, value, .. } => {
                self.compile_node(value, bytecode);
                let var_name = name.trim_start_matches('$');
                bytecode.push(Instruction::PopIntLocal(var_name.to_string()));
            }
            
            AstKind::If { expression, value, return_statement } => {
                // Compile the condition
                self.compile_node(expression, bytecode);
                
                // Add branch instruction
                let jump_index = bytecode.instructions.len();
                bytecode.push(Instruction::BranchNot(0));  // Placeholder jump target
                
                // Compile the return statement if it exists
                if let AstKind::Return(expr) = &**return_statement {
                    self.compile_node(expr, bytecode);
                    bytecode.push(Instruction::Return);
                }
                
                // Add jump instruction to skip else block
                let else_jump_index = bytecode.instructions.len();
                bytecode.push(Instruction::Jump(0));  // Placeholder jump target
                
                // Update the branch target
                let current_len = bytecode.instructions.len();
                bytecode.instructions[jump_index] = Instruction::BranchNot(current_len);
                
                // Compile the value
                self.compile_node(value, bytecode);
                
                // Update the else jump target
                let current_len = bytecode.instructions.len();
                bytecode.instructions[else_jump_index] = Instruction::Jump(current_len);
            }
            
            AstKind::While { condition, body } => {
                let loop_start = bytecode.instructions.len();
                
                // Compile condition
                self.compile_node(condition, bytecode);
                
                // Add branch instruction to exit loop if condition is false
                let branch_pos = bytecode.instructions.len();
                bytecode.push(Instruction::BranchNot(0)); // Placeholder for end of loop
                
                // Compile body
                self.compile_node(body, bytecode);
                
                // Add jump back to start of loop
                bytecode.push(Instruction::Jump(loop_start));
                
                // Update the branch position to point to after the loop
                let end_pos = bytecode.instructions.len();
                bytecode.instructions[branch_pos] = Instruction::BranchNot(end_pos);
            }
            
            AstKind::Block(statements) => {
                for stmt in statements {
                    self.compile_node(stmt, bytecode);
                }
            }
            
            AstKind::Return(expr) => {
                self.compile_node(expr, bytecode);
                bytecode.push(Instruction::Return);
            }
            
            AstKind::FunctionCall { name, arguments } => {
                match name.as_str() {
                    "calc" => {
                        if let Some(arg) = arguments.first() {
                            if let AstKind::BinaryExpression { lhs, rhs, operator } = &**arg {
                                self.compile_node(lhs, bytecode);
                                self.compile_node(rhs, bytecode);
                                
                                match operator.as_str() {
                                    "+" => bytecode.push(Instruction::Add),
                                    "-" => bytecode.push(Instruction::Subtract),
                                    "*" => bytecode.push(Instruction::Multiply),
                                    "/" => bytecode.push(Instruction::Divide),
                                    _ => panic!("Unknown operator in calc(): {}", operator),
                                }
                            } else {
                                println!("Non-binary expression in calc(): {:?}", arg);
                                self.compile_node(arg, bytecode);
                            }
                        }
                    }
                    "abs" => {
                        if let Some(arg) = arguments.first() {
                            self.compile_node(arg, bytecode);
                            bytecode.push(Instruction::Abs);
                        }
                    }
                    _ => panic!("Unknown function: {}", name),
                }
            }
            
            AstKind::ScriptCall { script, arguments } => {
                // First compile the arguments in order
                for arg in arguments {
                    self.compile_node(arg, bytecode);
                }
                
                // Push the number of arguments
                bytecode.push(Instruction::PushConstantInt(arguments.len() as i32));
                
                // Then add the script call instruction
                if let AstKind::Identifier(script_name) = &**script {
                    bytecode.push(Instruction::GosubWithParams(script_name.clone()));
                } else {
                    panic!("Script call target must be an identifier");
                }
            }
            
            _ => {}
        }
    }
} 