use crate::bytecode::{ByteCode, Instruction};
use crate::parser::AstKind;
use std::collections::HashMap;

pub struct Compiler {
    scripts: HashMap<String, ByteCode>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
        }
    }

    pub fn compile_script(&mut self, name: String, ast: &AstKind) -> ByteCode {
        let mut bytecode = ByteCode::new(name.clone());
        
        match ast {
            AstKind::Trigger { body, args, .. } => {
                // Initialize arguments
                let mut arg_index = 0;
                for arg in args.iter().skip(1).step_by(2) {  // Skip type nodes and get variable names
                    if let AstKind::LocalVar(name) = &**arg {
                        let var_name = name.trim_start_matches('$');
                        bytecode.push(Instruction::PushIntLocal(format!("arg{}", arg_index)));
                        bytecode.push(Instruction::PopIntLocal(var_name.to_string()));
                        arg_index += 1;
                    }
                }
                
                self.compile_node(body, &mut bytecode);
                
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
        
        self.scripts.insert(name, bytecode.clone());
        bytecode
    }

    fn compile_node(&self, node: &AstKind, bytecode: &mut ByteCode) {
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
                match operator.as_str() {
                    // Comparison operators can be used directly
                    "<=" | ">=" | "<" | ">" | "=" => {
                        self.compile_node(lhs, bytecode);
                        self.compile_node(rhs, bytecode);
                        match operator.as_str() {
                            "<=" => {
                                bytecode.push(Instruction::BranchLessThanOrEquals(bytecode.instructions.len() + 3));
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
                            "=" => {
                                bytecode.push(Instruction::BranchEquals(bytecode.instructions.len() + 3));
                                bytecode.push(Instruction::PushConstantInt(0));
                                bytecode.push(Instruction::Jump(bytecode.instructions.len() + 2));
                                bytecode.push(Instruction::PushConstantInt(1));
                            },
                            _ => unreachable!(),
                        }
                    },
                    // Arithmetic operators must be inside calc()
                    "+" | "-" | "*" | "/" => {
                        println!("Found arithmetic operator {} outside calc()", operator);
                        panic!("Arithmetic expressions must be inside calc() function");
                    },
                    _ => panic!("Unknown operator: {}", operator),
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
                // Compile condition
                self.compile_node(expression, bytecode);
                
                // Add branch instruction
                let branch_pos = bytecode.instructions.len();
                bytecode.push(Instruction::BranchNot(0)); // Placeholder
                
                // Compile the true branch
                self.compile_node(return_statement, bytecode);
                
                // Add jump over false branch
                let jump_pos = bytecode.instructions.len();
                bytecode.push(Instruction::Jump(0)); // Placeholder
                
                // Update the branch position
                let false_branch_pos = bytecode.instructions.len();
                if let Instruction::BranchNot(_) = bytecode.instructions[branch_pos] {
                    bytecode.instructions[branch_pos] = Instruction::BranchNot(false_branch_pos);
                }
                
                // Compile false branch
                self.compile_node(value, bytecode);
                
                // Update the jump position
                let end_pos = bytecode.instructions.len();
                if let Instruction::Jump(_) = bytecode.instructions[jump_pos] {
                    bytecode.instructions[jump_pos] = Instruction::Jump(end_pos);
                }
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
                    _ => panic!("Unknown function: {}", name),
                }
            }
            
            AstKind::ScriptCall { script, arguments } => {
                // Push all arguments onto the stack
                for arg in arguments {
                    self.compile_node(arg, bytecode);
                }
                
                if let AstKind::Identifier(script_name) = &**script {
                    if arguments.is_empty() {
                        bytecode.push(Instruction::Gosub(script_name.clone()));
                    } else {
                        bytecode.push(Instruction::GosubWithParams(script_name.clone()));
                    }
                }
            }
            
            _ => {}
        }
    }
} 