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
    memo_cache: HashMap<(String, Vec<i32>), i32>,
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
            script_vars: Vec::new(),
            scripts: HashMap::new(),
            current_script: None,
            call_stack: Vec::new(),
            instruction_count: 0,
            max_instructions: 10_000_000,
            memo_cache: HashMap::new(),
        }
    }

    pub fn register_script(&mut self, bytecode: ByteCode) {
        self.scripts.insert(bytecode.script_name.clone(), bytecode);
    }

    pub fn run_script(&mut self, name: &str, args: &[i32]) -> Result<i32, String> {
        println!("Executing {} with args: {:?}", name, args);
        
        // Clear any existing variables
        self.variables.clear();
        
        // Set up arguments
        for (i, &arg) in args.iter().enumerate() {
            let arg_name = format!("arg{}", i);
            println!("Setting {} = {}", arg_name, arg);
            self.variables.insert(arg_name, arg);
        }

        // Check memo cache first
        let cache_key = (name.to_string(), args.to_vec());
        if let Some(&cached_result) = self.memo_cache.get(&cache_key) {
            return Ok(cached_result);
        }

        let script = self.scripts.get(name).ok_or_else(|| format!("Script '{}' not found", name))?;
        let instructions = script.instructions.clone();
        
        // Save current state
        let old_ip = self.ip;
        let old_script = self.current_script.clone();
        let old_variables = self.variables.clone();
        let old_stack = self.stack.clone();
        
        // Reset instruction pointer and initialize new variables
        self.ip = 0;
        self.current_script = Some(name.to_string());
        self.variables.clear();
        self.stack.clear();
        
        // Initialize script arguments
        for (i, &arg) in args.iter().enumerate() {
            let arg_name = format!("arg{}", i);
            self.variables.insert(arg_name, arg);
        }
        
        // Execute instructions
        let mut result = Ok(0);
        while self.ip < instructions.len() {
            if self.instruction_count >= self.max_instructions {
                result = Err(format!("Execution exceeded maximum instruction count ({}).", self.max_instructions));
                break;
            }
            self.instruction_count += 1;
            
            let current_ip = self.ip;
            self.ip += 1;  // Advance instruction pointer by default
            
            match &instructions[current_ip] {
                Instruction::PushConstantInt(value) => {
                    println!("Pushing constant: {}", value);
                    self.stack.push(*value);
                }
                
                Instruction::PushIntLocal(name) => {
                    let value = self.variables.get(name).copied().unwrap_or(0);
                    println!("Pushing local {}: {}", name, value);
                    self.stack.push(value);
                }
                
                Instruction::PopIntLocal(name) => {
                    let value = self.stack.pop().unwrap_or(0);
                    println!("Popping into local {}: {}", name, value);
                    self.variables.insert(name.clone(), value);
                }
                
                Instruction::Add => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    match a.checked_add(b) {
                        Some(result) => self.stack.push(result),
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
                        Some(result) => self.stack.push(result),
                        None => {
                            result = Err("Integer overflow".to_string());
                            break;
                        }
                    }
                }
                
                Instruction::Multiply => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    match a.checked_mul(b) {
                        Some(result) => {
                            println!("Multiplying {} * {} = {}", a, b, result);
                            self.stack.push(result)
                        },
                        None => return Err("Integer overflow".to_string()),
                    }
                }
                
                Instruction::BranchGreaterThan(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    println!("Comparing {} > {}", a, b);
                    if a > b {
                        println!("Branch taken to {}", pos);
                        self.ip = *pos;
                    } else {
                        println!("Branch not taken");
                    }
                }
                
                Instruction::BranchGreaterThanOrEquals(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    println!("Comparing {} >= {}", a, b);
                    if a >= b {
                        println!("Branch taken to {}", pos);
                        self.ip = *pos;
                    } else {
                        println!("Branch not taken");
                    }
                }
                
                Instruction::BranchLessThan(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    println!("Comparing {} < {}", a, b);
                    if a < b {
                        println!("Branch taken to {}", pos);
                        self.ip = *pos;
                    } else {
                        println!("Branch not taken");
                    }
                }
                
                Instruction::BranchLessThanOrEquals(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    println!("Comparing {} <= {}", a, b);
                    if a <= b {
                        println!("Branch taken to {}", pos);
                        self.ip = *pos;
                    } else {
                        println!("Branch not taken");
                    }
                }
                
                Instruction::BranchEquals(pos) => {
                    let b = self.stack.pop().unwrap_or(0);
                    let a = self.stack.pop().unwrap_or(0);
                    println!("Comparing {} = {}", a, b);
                    if a == b {
                        println!("Branch taken to {}", pos);
                        self.ip = *pos;
                    } else {
                        println!("Branch not taken");
                    }
                }
                
                Instruction::BranchNot(pos) => {
                    let value = self.stack.pop().unwrap_or(0);
                    println!("Testing condition: {}", value);
                    if value == 0 {
                        println!("Branch taken to {}", pos);
                        self.ip = *pos;
                    } else {
                        println!("Branch not taken");
                    }
                }
                
                Instruction::Jump(pos) => {
                    println!("Jumping to {}", pos);
                    self.ip = *pos;
                }
                
                Instruction::GosubWithParams(script_name) => {
                    // Pop arguments in reverse order (since they were pushed in forward order)
                    let mut args = Vec::new();
                    let num_args = self.stack.pop().unwrap_or(0) as usize;
                    for _ in 0..num_args {
                        args.push(self.stack.pop().unwrap_or(0));
                    }
                    args.reverse(); // Put them back in the right order
                    
                    // Debug print
                    println!("Executing {} with args: {:?}", script_name, args);
                    
                    // Check memo cache first
                    let cache_key = (script_name.clone(), args.clone());
                    if let Some(&cached_result) = self.memo_cache.get(&cache_key) {
                        println!("Cache hit for {} with args {:?}: result = {}", script_name, args, cached_result);
                        self.stack.push(cached_result);
                        continue;
                    }
                    println!("Cache miss for {} with args {:?}", script_name, args);

                    // Save current state
                    let saved_ip = self.ip;
                    let saved_script = self.current_script.clone();
                    let saved_variables = self.variables.clone();
                    let saved_stack = self.stack.clone();
                    
                    // Set up new script execution
                    self.ip = 0;
                    self.current_script = Some(script_name.clone());
                    self.variables.clear();
                    self.stack.clear();
                    
                    // Set up arguments
                    for (i, &arg) in args.iter().enumerate() {
                        let arg_name = format!("arg{}", i);
                        println!("Setting {} = {}", arg_name, arg);
                        self.variables.insert(arg_name, arg);
                    }
                    
                    // Get the script
                    let script = match self.scripts.get(script_name) {
                        Some(script) => script,
                        None => {
                            result = Err(format!("Script '{}' not found", script_name));
                            break;
                        }
                    };
                    
                    // Execute the script
                    let mut script_result = Ok(0);
                    let script_instructions = script.instructions.clone();
                    while self.ip < script_instructions.len() {
                        if self.instruction_count >= self.max_instructions {
                            script_result = Err(format!("Execution exceeded maximum instruction count ({}).", self.max_instructions));
                            break;
                        }
                        self.instruction_count += 1;
                        
                        let current_ip = self.ip;
                        self.ip += 1;
                        
                        match &script_instructions[current_ip] {
                            Instruction::Return => {
                                let return_value = self.stack.pop().unwrap_or(0);
                                script_result = Ok(return_value);
                                break;
                            }
                            _ => {
                                // Handle other instructions recursively
                                match self.execute_instruction(&script_instructions[current_ip]) {
                                    Ok(_) => continue,
                                    Err(e) => {
                                        script_result = Err(e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    // Restore state
                    self.ip = saved_ip;
                    self.current_script = saved_script;
                    self.variables = saved_variables;
                    self.stack = saved_stack;
                    
                    match script_result {
                        Ok(value) => {
                            self.stack.push(value);
                            self.memo_cache.insert(cache_key, value);
                        }
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    }
                }
                
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap_or(0);
                    result = Ok(return_value);
                    break;
                }
                
                _ => {
                    // For now, just ignore other instructions
                    continue;
                }
            }
        }
        
        // Restore previous state
        self.ip = old_ip;
        self.current_script = old_script;
        self.variables = old_variables;
        self.stack = old_stack;
        
        result
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), String> {
        match instruction {
            Instruction::PushConstantInt(value) => {
                println!("Pushing constant: {}", value);
                self.stack.push(*value);
            }
            
            Instruction::PushIntLocal(name) => {
                let value = self.variables.get(name).copied().unwrap_or(0);
                println!("Pushing local {}: {}", name, value);
                self.stack.push(value);
            }
            
            Instruction::PopIntLocal(name) => {
                let value = self.stack.pop().unwrap_or(0);
                println!("Popping into local {}: {}", name, value);
                self.variables.insert(name.clone(), value);
            }
            
            Instruction::Add => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                match a.checked_add(b) {
                    Some(result) => self.stack.push(result),
                    None => return Err("Integer overflow".to_string()),
                }
            }
            
            Instruction::Subtract => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                match a.checked_sub(b) {
                    Some(result) => self.stack.push(result),
                    None => return Err("Integer overflow".to_string()),
                }
            }
            
            Instruction::Multiply => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                println!("Multiplying {} * {} = {}", a, b, a * b);
                self.stack.push(a * b);
            }
            
            Instruction::BranchGreaterThan(pos) => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                println!("Comparing {} > {}", a, b);
                if a > b {
                    println!("Branch taken to {}", pos);
                    self.ip = *pos;
                } else {
                    println!("Branch not taken");
                }
            }
            
            Instruction::BranchGreaterThanOrEquals(pos) => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                println!("Comparing {} >= {}", a, b);
                if a >= b {
                    println!("Branch taken to {}", pos);
                    self.ip = *pos;
                } else {
                    println!("Branch not taken");
                }
            }
            
            Instruction::BranchLessThan(pos) => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                println!("Comparing {} < {}", a, b);
                if a < b {
                    println!("Branch taken to {}", pos);
                    self.ip = *pos;
                } else {
                    println!("Branch not taken");
                }
            }
            
            Instruction::BranchLessThanOrEquals(pos) => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                println!("Comparing {} <= {}", a, b);
                if a <= b {
                    println!("Branch taken to {}", pos);
                    self.ip = *pos;
                } else {
                    println!("Branch not taken");
                }
            }
            
            Instruction::BranchEquals(pos) => {
                let b = self.stack.pop().unwrap_or(0);
                let a = self.stack.pop().unwrap_or(0);
                println!("Comparing {} = {}", a, b);
                if a == b {
                    println!("Branch taken to {}", pos);
                    self.ip = *pos;
                } else {
                    println!("Branch not taken");
                }
            }
            
            Instruction::BranchNot(pos) => {
                let value = self.stack.pop().unwrap_or(0);
                println!("Testing condition: {}", value);
                if value == 0 {
                    println!("Branch taken to {}", pos);
                    self.ip = *pos;
                } else {
                    println!("Branch not taken");
                }
            }
            
            Instruction::Jump(pos) => {
                println!("Jumping to {}", pos);
                self.ip = *pos;
            }
            
            _ => {
                // For now, just ignore other instructions
            }
        }
        
        Ok(())
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