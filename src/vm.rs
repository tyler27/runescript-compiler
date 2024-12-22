use std::collections::HashMap;
use crate::bytecode::{ByteCode, Instruction};

#[derive(Debug)]
pub struct VM {
    ip: usize,
    stack: Vec<i32>,
    string_stack: Vec<String>,
    variables: HashMap<String, i32>,
    string_variables: HashMap<String, String>,
    arrays: HashMap<String, Vec<i32>>,
    script_vars: Vec<i32>,
    scripts: HashMap<String, ByteCode>,
    current_script: Option<String>,
    call_stack: Vec<(usize, Option<String>)>,
    instruction_count: usize,
    max_instructions: usize,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::new(),
            string_stack: Vec::new(),
            variables: HashMap::new(),
            string_variables: HashMap::new(),
            arrays: HashMap::new(),
            script_vars: vec![0; 1000],
            scripts: HashMap::new(),
            current_script: None,
            call_stack: Vec::new(),
            instruction_count: 0,
            max_instructions: 1000,
        }
    }

    pub fn register_script(&mut self, bytecode: ByteCode) {
        self.scripts.insert(bytecode.script_name.clone(), bytecode);
    }

    pub fn run_script(&mut self, name: &str, args: &[i32]) -> Result<i32, String> {
        let script = self.scripts.get(name).ok_or_else(|| format!("Script '{}' not found", name))?;
        let instructions = script.instructions.clone();
        
        // Save current state
        let old_ip = self.ip;
        let old_variables = self.variables.clone();
        
        // Reset instruction pointer and initialize new variables
        self.ip = 0;
        self.variables.clear();
        
        // Initialize script arguments
        for (i, &arg) in args.iter().enumerate() {
            let arg_name = format!("arg{}", i);
            self.variables.insert(arg_name, arg);
        }
        
        // Execute instructions
        let mut result = Ok(0);
        while self.ip < instructions.len() {
            if self.instruction_count >= 1000 {
                result = Err("Execution exceeded maximum instruction count (1000).".to_string());
                break;
            }
            self.instruction_count += 1;
            
            let current_ip = self.ip;
            self.ip += 1;  // Advance instruction pointer by default
            
            match &instructions[current_ip] {
                Instruction::PushIntLocal(name) => {
                    let value = self.variables.get(name).copied().unwrap_or(0);
                    self.stack.push(value);
                }
                
                Instruction::PopIntLocal(name) => {
                    let value = self.stack.pop().unwrap_or(0);
                    self.variables.insert(name.to_string(), value);
                }
                
                Instruction::PushConstantInt(value) => {
                    self.stack.push(*value);
                }
                
                Instruction::BranchLessThan(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    if a < b {
                        self.ip = *pos;
                    }
                }
                
                Instruction::BranchLessThanOrEquals(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    if a <= b {
                        self.ip = *pos;
                    }
                }
                
                Instruction::BranchEquals(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    if a == b {
                        self.ip = *pos;
                    }
                }
                
                Instruction::BranchNot(pos) => {
                    let value = self.stack.pop().unwrap_or(0);
                    if value == 0 {
                        self.ip = *pos;
                    }
                }
                
                Instruction::Jump(pos) => {
                    self.ip = *pos;
                }
                
                Instruction::Add => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    match a.checked_add(b) {
                        Some(result) => {
                            self.stack.push(result);
                        }
                        None => {
                            result = Err("Integer overflow".to_string());
                            break;
                        }
                    }
                }
                
                Instruction::Subtract => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    match a.checked_sub(b) {
                        Some(result) => {
                            self.stack.push(result);
                        }
                        None => {
                            result = Err("Integer overflow".to_string());
                            break;
                        }
                    }
                }
                
                Instruction::GosubWithParams(script_name) => {
                    let arg = self.stack.pop().unwrap_or(0);
                    match self.run_script(&script_name, &[arg]) {
                        Ok(value) => self.stack.push(value),
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    }
                }
                
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap_or(0);
                    println!("Result: {}", return_value);
                    result = Ok(return_value);
                    break;
                }
                
                _ => {
                    result = Err(format!("Unsupported instruction: {:?}", instructions[current_ip]));
                    break;
                }
            }
        }
        
        // Restore previous state
        self.ip = old_ip;
        self.variables = old_variables;
        
        result
    }

    fn call_script(&mut self, script_name: &str) -> Result<(), String> {
        if !self.scripts.contains_key(script_name) {
            return Err(format!("Script not found: {}", script_name));
        }
        
        // Save current instruction pointer and script
        if self.current_script.is_some() {
            self.call_stack.push((self.ip, self.current_script.clone()));
        }
        
        // Reset instruction pointer for new script
        self.ip = 0;
        self.current_script = Some(script_name.to_string());
        
        Ok(())
    }

    fn execute_bytecode(&mut self) -> Result<i32, String> {
        self.instruction_count = 0;
        
        while let Some(ref script_name) = self.current_script.clone() {
            self.instruction_count += 1;
            if self.instruction_count > self.max_instructions {
                return Err(format!("Execution exceeded maximum instruction count ({}).", self.max_instructions));
            }

            let result = self.run_script(script_name, &[]);
            
            if let Some((return_ip, return_script)) = self.call_stack.pop() {
                self.ip = return_ip;
                self.current_script = return_script;
                if self.call_stack.is_empty() && self.current_script.is_none() {
                    // Main script finished
                    return result;
                }
                continue;
            }
            
            return result;
        }
        
        Ok(self.stack.pop().unwrap_or(0))
    }
} 