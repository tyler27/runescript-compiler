use crate::token::Token;
use std::fmt;
use std::path::PathBuf;
use std::error::Error;

#[derive(Debug)]
pub enum CompilerError {
    FileNotFound(String),
    IO(std::io::Error),
    LexingError(LexingError),
    Syntax(SyntaxError),
}

impl Error for CompilerError {}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::IO(err) => writeln!(f, "IOError: {}", err),
            CompilerError::FileNotFound(err) => writeln!(f, "FileNotFoundError: {}", err),
            CompilerError::LexingError(err) => writeln!(f, "LexingError: {}", err),
            CompilerError::Syntax(err) => writeln!(f, "SyntaxError: {}", err),
        }
    }
}

#[derive(Debug)]
pub struct LexingError{
    pub(crate) path: PathBuf,
    pub(crate) message: String,
    pub(crate) line: usize,
    pub(crate) position: usize,
}

impl Error for LexingError {}

impl LexingError {
    pub fn new(path: PathBuf, message: String, line: usize, position: usize) -> Self {
        Self {
            path,
            message,
            line,
            position
        }
    }
}

#[derive(Debug)]
pub struct SyntaxError{
    pub(crate) path: PathBuf,
    pub(crate) message: String,
    pub(crate) line: usize,
    pub(crate) position: usize,
    pub(crate) char: String
}

impl Error for SyntaxError {}

impl SyntaxError {
    pub fn from_token(path: PathBuf, token: &Token, message: String) -> Self {
        Self {
            path,
            message,
            line: token.line,
            position: token.position,
            char: token.value.clone()
        }
    }
}

impl fmt::Display for LexingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "LexingError: {}\n  --> {}:{}:{}",
            self.message,
            self.path.display(),
            self.line + 1,
            self.position - 1,
        )
    }
}


impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "SyntaxError: {}\n  --> {}:{}:{}",
            self.message,
            self.path.display(),
            self.line + 1,
            self.position - 1,
        )
    }
}
