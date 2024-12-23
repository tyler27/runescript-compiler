#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub(crate) line: usize,
    pub(crate) position: usize,
    pub(crate) kind: Kind,
    pub(crate) value: String
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    // Brackets and delimiters
    RBracket,    // ]
    LBracket,    // [
    RParen,      // )
    LParen,      // (
    LBrace,      // {
    RBrace,      // }
    Semicolon,   // ;
    Comma,       // ,
    
    // Operators
    Equals,      // =
    BinaryOperator,  // +, -, *, /
    ComparisonOperator, // <, >, <=, >=, =
    
    // Special characters
    Underscore,  // _
    ScriptCall,  // ~ (gosub operator)
    
    // Keywords
    Trigger,     // proc, clientscript, etc
    Command,     // calc, map_members, etc
    Def,        // def_int, def_string, etc
    Return,     // return
    If,         // if
    While,      // while
    
    // Identifiers and literals
    Identifier,  // Regular identifiers
    LocalVar,    // $ prefixed variables
    Number,      // Numeric literals
    
    // Comments
    SingleLineComment,  // // comment
    MultiLineComment,   // /* comment */
    
    EOF         // End of file marker
}