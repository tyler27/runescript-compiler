#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Instruction {
    // Core language ops (0-99)
    PushConstantInt(i32) = 0,    // Push constant integer onto stack
    PushVarp(i32) = 1,          // Push variable parameter
    PopVarp(i32) = 2,           // Pop and store to variable parameter
    PushConstantString(String) = 3, // Push constant string onto stack
    PushVarn(i32) = 4,          // Push named variable
    PopVarn(i32) = 5,           // Pop and store to named variable
    Branch(usize) = 6,          // Branch if true
    BranchNot(usize) = 7,       // Branch if false
    BranchEquals(usize) = 8,    // Branch if equal
    BranchLessThan(usize) = 9,  // Branch if less than
    BranchGreaterThan(usize) = 10, // Branch if greater than
    PushVars(i32) = 11,         // Push script variable
    PopVars(i32) = 12,          // Pop and store to script variable
    Add = 13,                   // Add top two stack values
    Subtract = 14,              // Subtract top two stack values
    Multiply = 15,              // Multiply top two stack values
    Divide = 16,                // Divide top two stack values
    Return = 21,                // Return from current script
    Gosub(String) = 22,         // Call a script (without params)
    Jump(usize) = 23,           // Unconditional jump
    Switch(Vec<(i32, usize)>) = 24, // Switch statement
    BranchLessThanOrEquals(usize) = 31, // Branch if less than or equal
    BranchGreaterThanOrEquals(usize) = 32, // Branch if greater than or equal
    BranchNotEquals(usize) = 33, // Branch if not equal
    PushIntLocal(String) = 34,  // Push local integer variable
    PopIntLocal(String) = 35,   // Pop and store to local integer variable
    PushStringLocal(String) = 36, // Push local string variable
    PopStringLocal(String) = 37, // Pop and store to local string variable
    JoinString = 38,            // Concatenate strings
    PopIntDiscard = 39,         // Pop and discard integer
    PopStringDiscard = 40,      // Pop and discard string
    GosubWithParams(String) = 41, // Call a script with parameters
    JumpWithParams(usize) = 42, // Jump with parameters
    DefineArray(String, usize) = 44, // Define an array with size
    PushArrayInt(String) = 45,  // Push array element
    PopArrayInt(String) = 46,   // Pop and store to array element
}

#[derive(Debug, Clone)]
pub struct ByteCode {
    pub instructions: Vec<Instruction>,
    pub script_name: String,
    pub constants: Vec<i32>,
    pub strings: Vec<String>,
    pub locals: Vec<String>,
    pub arrays: Vec<String>,
}

impl ByteCode {
    pub fn new(script_name: String) -> Self {
        Self {
            instructions: Vec::new(),
            script_name,
            constants: Vec::new(),
            strings: Vec::new(),
            locals: Vec::new(),
            arrays: Vec::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn add_constant(&mut self, value: i32) -> usize {
        if let Some(pos) = self.constants.iter().position(|&x| x == value) {
            pos
        } else {
            self.constants.push(value);
            self.constants.len() - 1
        }
    }

    pub fn add_string(&mut self, value: String) -> usize {
        if let Some(pos) = self.strings.iter().position(|x| x == &value) {
            pos
        } else {
            self.strings.push(value);
            self.strings.len() - 1
        }
    }

    pub fn add_local(&mut self, name: String) -> usize {
        if let Some(pos) = self.locals.iter().position(|x| x == &name) {
            pos
        } else {
            self.locals.push(name);
            self.locals.len() - 1
        }
    }

    pub fn add_array(&mut self, name: String) -> usize {
        if let Some(pos) = self.arrays.iter().position(|x| x == &name) {
            pos
        } else {
            self.arrays.push(name);
            self.arrays.len() - 1
        }
    }
} 