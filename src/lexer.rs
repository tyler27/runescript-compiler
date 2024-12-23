use std::iter;
use std::iter::from_fn;
use std::path::PathBuf;
use crate::error::LexingError;
use crate::token::{Kind, Token};

pub struct Lexer<'a> {
    source_code: &'a str,
    file_name: &'a PathBuf,
    line: usize,
    position: usize,
    current: usize,
    chars: Vec<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, file_name: &'a PathBuf) -> Self {
        Self {
            source_code: input,
            file_name,
            line: 0,
            position: 0,
            current: 0,
            chars: input.chars().collect(),
        }
    }

    fn at(&self) -> char {
        if self.current >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn advance(&mut self) {
        self.current += 1;
        self.position += 1;
    }

    fn is_eof(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn create_token(&mut self, kind: Kind, value: String) -> Token {
        Token {
            line: self.line,
            position: self.position,
            kind,
            value,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexingError> {
        let mut tokens = Vec::new();
        let mut iter = self.source_code.chars().peekable();

        while let Some(ch) = iter.next() {
            self.position += 1;

            match ch {
                '\n' => {
                    self.line += 1;
                    self.position = 0;
                    continue;
                }
                ch if ch.is_whitespace() => {
                    continue
                },
                '[' => {
                    let token = self.create_token(Kind::LBracket, ch.to_string());
                    tokens.push(token);
                },
                ']' => {
                    let token = self.create_token(Kind::RBracket, ch.to_string());
                    tokens.push(token);
                },
                '{' => {
                    let token = self.create_token(Kind::LBrace, ch.to_string());
                    tokens.push(token);
                },
                '}' => {
                    let token = self.create_token(Kind::RBrace, ch.to_string());
                    tokens.push(token);
                },
                '~' => {
                    let token = self.create_token(Kind::ScriptCall, "~".to_string());
                    tokens.push(token);
                },
                '(' => {
                    let token = self.create_token(Kind::LParen, ch.to_string());
                    tokens.push(token);
                },
                ')' => {
                    let token = self.create_token(Kind::RParen, ch.to_string());
                    tokens.push(token);
                },
                '$' => {
                    let token = self.create_token(Kind::LocalVar, ch.to_string());
                    tokens.push(token);
                },
                '=' => {
                    let mut is_comparison = false;
                    for i in (0..tokens.len()).rev() {
                        if tokens[i].kind == Kind::If || tokens[i].kind == Kind::While {
                            is_comparison = true;
                            break;
                        }
                        if tokens[i].kind == Kind::Def || tokens[i].kind == Kind::LocalVar {
                            break;
                        }
                    }
                    
                    if is_comparison {
                        let token = self.create_token(Kind::ComparisonOperator, ch.to_string());
                        tokens.push(token);
                    } else {
                        let token = self.create_token(Kind::Equals, ch.to_string());
                        tokens.push(token);
                    }
                },
                '<' => {
                    if iter.peek() == Some(&'=') {
                        iter.next();  // consume the '='
                        self.position += 1;
                        let token = self.create_token(Kind::ComparisonOperator, "<=".to_string());
                        tokens.push(token);
                    } else {
                        let token = self.create_token(Kind::ComparisonOperator, "<".to_string());
                        tokens.push(token);
                    }
                },
                '>' => {
                    if iter.peek() == Some(&'=') {
                        iter.next();  // consume the '='
                        self.position += 1;
                        let token = self.create_token(Kind::ComparisonOperator, ">=".to_string());
                        tokens.push(token);
                    } else {
                        let token = self.create_token(Kind::ComparisonOperator, ">".to_string());
                        tokens.push(token);
                    }
                },
                '+' | '-' | '*' => {
                    let token = self.create_token(Kind::BinaryOperator, ch.to_string());
                    tokens.push(token);
                },
                '/' => {
                    if let Some(next_char) = iter.peek() {
                        match next_char {
                            '/' => {
                                // Single-line comment
                                iter.next(); // consume the second '/'
                                self.position += 1;
                                let comment: String = iter.by_ref()
                                    .take_while(|&c| c != '\n')
                                    .collect();
                                self.position += comment.len();
                                let token = self.create_token(Kind::SingleLineComment, comment);
                                tokens.push(token);
                                continue;
                            },
                            '*' => {
                                // Multi-line comment
                                iter.next(); // consume the '*'
                                self.position += 1;
                                let mut comment = String::new();
                                let mut depth = 1;
                                let mut prev_char = '\0';
                                
                                while depth > 0 {
                                    match iter.next() {
                                        Some(c) => {
                                            self.position += 1;
                                            if c == '\n' {
                                                self.line += 1;
                                                self.position = 0;
                                            }
                                            
                                            if prev_char == '/' && c == '*' {
                                                depth += 1;
                                            } else if prev_char == '*' && c == '/' {
                                                depth -= 1;
                                                if depth == 0 {
                                                    // Remove the last '*' from the comment
                                                    comment.pop();
                                                    break;
                                                }
                                            }
                                            
                                            comment.push(c);
                                            prev_char = c;
                                        },
                                        None => {
                                            return Err(LexingError::new(
                                                self.file_name.clone(),
                                                "Unterminated multi-line comment".to_string(),
                                                self.line,
                                                self.position
                                            ));
                                        }
                                    }
                                }
                                
                                let token = self.create_token(Kind::MultiLineComment, comment);
                                tokens.push(token);
                                continue;
                            },
                            _ => {
                                let token = self.create_token(Kind::BinaryOperator, ch.to_string());
                                tokens.push(token);
                            }
                        }
                    } else {
                        let token = self.create_token(Kind::BinaryOperator, ch.to_string());
                        tokens.push(token);
                    }
                },
                ';' => {
                    let token = self.create_token(Kind::Semicolon, ch.to_string());
                    tokens.push(token);
                },
                ',' => {
                    let token = self.create_token(Kind::Comma, ch.to_string());
                    tokens.push(token);
                },
                '_' => {
                    let token = self.create_token(Kind::Underscore, ch.to_string());
                    tokens.push(token);
                },
                c => {
                    if c.is_alphabetic() || c == '_' {
                        let ident: String = iter::once(ch)
                            .chain(from_fn(|| iter.by_ref().next_if(|s| s.is_alphanumeric() || *s == '_')))
                            .collect::<String>()
                            .parse()
                            .unwrap();

                        self.position += ident.len();

                        match self.get_keyword_token(&ident) {
                            Ok(keyword_token) => {
                                let token = self.create_token(keyword_token, ident);
                                tokens.push(token);
                            },
                            Err(_err) => {
                                let token = self.create_token(Kind::Identifier, ident);
                                tokens.push(token);
                            },
                        }
                    } else if c.is_ascii_digit() {
                        let number: String = iter::once(ch)
                            .chain(from_fn(|| iter.by_ref().next_if(|s| s.is_ascii_digit())))
                            .collect::<String>()
                            .parse()
                            .unwrap();

                        self.position += number.len();
                        let token = self.create_token(Kind::Number, number);
                        tokens.push(token);
                    } else {
                        return Err(LexingError::new(
                            self.file_name.clone(),
                            format!("Unrecognized character {}", ch),
                            self.line,
                            self.position,
                        ));
                    }
                }
            }
        }

        let eof_token = Token {
            line: self.line,
            position: self.position,
            kind: Kind::EOF,
            value: "EndOfFile".to_string(),
        };
        tokens.push(eof_token);

        Ok(tokens)
    }

    pub fn get_keyword_token(&self, ident: &String) -> Result<Kind, LexingError> {
        match ident.as_str() {
            "proc" | "clientscript" | "label" | "debugproc" => Ok(Kind::Trigger),
            "def_int" | "def_string" | "def_coord" | "def_loc" | 
            "def_obj" | "def_npc" | "def_boolean" | "def_namedobj" |
            "def_playeruid" | "def_npcuid" | "def_stat" | "def_component" |
            "def_interface" | "def_inv" | "def_enum" | "def_struct" |
            "def_param" | "def_dbtable" | "def_dbrow" | "def_dbcolumn" |
            "def_varp" | "def_mesanim" => Ok(Kind::Def),
            "if" => Ok(Kind::If),
            "while" => Ok(Kind::While),
            "return" => Ok(Kind::Return),
            "calc" => Ok(Kind::Command),
            _ => Ok(Kind::Identifier),
        }
    }
}
