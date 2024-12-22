use crate::error::SyntaxError;
use crate::token::{Kind, Token, Type};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AstKind {
    NumericLiteral(i32),
    StringLiteral(String),
    Identifier(String),
    Proc(String),
    BinaryExpression {
        lhs: Box<AstKind>,
        rhs: Box<AstKind>,
        operator: String,
    },
    Define {
        name: String,
        var_type: Type,
        value: Box<AstKind>
    },
    Program,
    Trigger {
        name: Box<AstKind>,
        kind: Box<AstKind>,
        args: Vec<Box<AstKind>>,
        body: Box<AstKind>,
        return_type: Box<AstKind>,
    },
    Integer,
    LocalVar(String),
    ReturnType,
    Return(Box<AstKind>),
    ConditionalExpression {
        lhs: Box<AstKind>,
        rhs: Box<AstKind>,
        value: Box<AstKind>,
    },
    If {
        expression: Box<AstKind>,
        value: Box<AstKind>,
        return_statement: Box<AstKind>,
    },
    AssignmentExpression,
    While {
        condition: Box<AstKind>,
        body: Box<AstKind>,
    },
    Block(Vec<AstKind>),
    FunctionCall {
        name: String,
        arguments: Vec<Box<AstKind>>,
    },
    Assignment {
        target: Box<AstKind>,
        value: Box<AstKind>,
    },
    ScriptCall {
        script: Box<AstKind>,
        arguments: Vec<Box<AstKind>>,
    },
}

#[derive(Debug, Clone)]
pub struct Script {
    pub body: Vec<AstKind>,
}

pub struct Parser {
    tokens: Vec<Token>,
    file_path: PathBuf,
}

impl Parser {
    pub(crate) fn new(vec: Vec<Token>, file_name: &PathBuf) -> Self {
        Self {
            tokens: vec,
            file_path: file_name.clone(),
        }
    }

    fn at(&self) -> &Token {
        self.tokens.first().unwrap()
    }

    fn next_token(&mut self) -> Token {
        self.tokens.remove(0)
    }

    pub(crate) fn parse(&mut self) -> Result<Script, SyntaxError> {
        let mut program = Script { body: Vec::new() };

        while !self.is_eof() {
            let body = self.parse_script_declaration()?;
            program.body.push(body)
        }

        Ok(program)
    }

    fn eat(&mut self, expecting: Kind) -> Result<(), SyntaxError> {
        let current = self.at();

        if current.kind != expecting {
            return Err(SyntaxError::from_token(
                self.file_path.clone(),
                self.at(),
                format!(
                    "Expecting {:?} but got unknown character {:?}",
                    expecting,
                    self.at().value
                ),
            ));
        }

        self.next_token(); // Consume expecting token.
        Ok(())
    }

    fn parse_script_declaration(&mut self) -> Result<AstKind, SyntaxError> {
        match self.at().kind {
            Kind::LBracket => {
                self.eat(Kind::LBracket)?;
                let kind = self.parse_primary_expression()?;
                self.eat(Kind::Comma)?;

                let primary_expression = self.parse_primary_expression();

                match primary_expression {
                    Ok(name) => {
                        self.eat(Kind::RBracket)?;

                        let mut args: Vec<Box<AstKind>> = Vec::new();

                        // Script declaration args
                        if self.at().kind == Kind::LParen {
                            self.eat(Kind::LParen)?;

                            while !self.is_eof() && self.at().kind != Kind::RParen {
                                if self.at().kind == Kind::Comma {
                                    self.eat(Kind::Comma)?;
                                }

                                // Parse type
                                let arg_type = self.parse_primary_expression()?;
                                args.push(Box::new(arg_type));
                                
                                // Parse variable name
                                if self.at().kind == Kind::LocalVar {
                                    let var = self.parse_primary_expression()?;
                                    args.push(Box::new(var));
                                } else {
                                    return Err(SyntaxError::from_token(
                                        self.file_path.clone(),
                                        self.at(),
                                        "Expected local variable name".to_string(),
                                    ));
                                }
                            }

                            self.eat(Kind::RParen)?;
                        }

                        let mut return_type: Box<AstKind> = Box::new(AstKind::ReturnType);

                        // Script declaration return type
                        if self.at().kind == Kind::LParen {
                            self.eat(Kind::LParen)?;

                            while !self.is_eof() && self.at().kind != Kind::RParen {
                                if self.at().kind == Kind::Comma {
                                    self.eat(Kind::Comma)?;
                                }
                                return_type = Box::new(self.parse_primary_expression()?)
                            }

                            self.eat(Kind::RParen)?;
                        }

                        // Parse all statements in the script body
                        let mut body_statements = Vec::new();
                        
                        // Keep parsing statements until we hit EOF or another trigger
                        while !self.is_eof() && self.at().kind != Kind::LBracket {
                            let stmt = self.parse_statement()?;
                            body_statements.push(stmt);
                        }

                        let trigger = Box::new(AstKind::Trigger {
                            name: Box::new(name),
                            kind: Box::new(kind),
                            body: Box::new(AstKind::Block(body_statements)),
                            args,
                            return_type,
                        });

                        Ok(*trigger)
                    }
                    Err(e) => {
                        if self.next_token().kind == Kind::Underscore {
                            // TODO: Fix this logic...
                            self.eat(Kind::RBracket)?;

                            Ok(AstKind::Trigger {
                                name: Box::new(AstKind::Identifier("_".to_string())),
                                kind: Box::new(kind),
                                body: Box::new(self.parse_statement()?),
                                args: Vec::new(),
                                return_type: Box::new(AstKind::ReturnType),
                            })
                        } else {
                            Err(SyntaxError::from_token(
                                self.file_path.clone(),
                                self.at(),
                                "Missing script declaration name. Syntax [trigger,declaration_name]"
                                    .to_string(),
                            ))
                        }
                    }
                }
            }
            _ => Err(SyntaxError::from_token(
                self.file_path.clone(),
                self.at(),
                format!("Unexpected token at script level: {:?}", self.at().kind),
            )),
        }
    }

    fn parse_statement(&mut self) -> Result<AstKind, SyntaxError> {
        match self.at().kind {
            Kind::Def => {
                let def_token = self.next_token();
                let var_type = self.get_type_from_def(&def_token.value)?;
                
                // Get the variable name
                let var_name = if let Kind::LocalVar = self.at().kind {
                    let _token = self.next_token();
                    let identifier = self.next_token();
                    identifier.value
                } else {
                    return Err(SyntaxError::from_token(
                        self.file_path.clone(),
                        self.at(),
                        "Expected local variable name".to_string(),
                    ));
                };

                // Check for initialization
                let initial_value = if self.at().kind == Kind::Equals {
                    self.eat(Kind::Equals)?;
                    let expr = self.parse_expression()?;
                    if self.at().kind == Kind::Semicolon {
                        self.eat(Kind::Semicolon)?;
                    }
                    expr
                } else {
                    if self.at().kind == Kind::Semicolon {
                        self.eat(Kind::Semicolon)?;
                    }
                    self.get_default_value_for_type(&var_type)
                };

                Ok(AstKind::Define {
                    name: var_name,
                    var_type,
                    value: Box::new(initial_value),
                })
            }
            Kind::If => {
                self.eat(Kind::If)?;
                self.eat(Kind::LParen)?;
                let condition = self.parse_expression()?;
                self.eat(Kind::RParen)?;

                let mut return_statement = Box::new(AstKind::ReturnType);

                let body = if self.at().kind == Kind::LBrace {
                    self.eat(Kind::LBrace)?;
                    let mut statements = Vec::new();

                    while !self.is_eof() && self.at().kind != Kind::RBrace {
                        let stmt = self.parse_statement()?;
                        if let AstKind::Return(value) = stmt {
                            return_statement = value;
                        } else {
                            statements.push(stmt);
                        }
                    }

                    self.eat(Kind::RBrace)?;
                    Box::new(AstKind::Block(statements))
                } else {
                    Box::new(self.parse_statement()?)
                };

                Ok(AstKind::If {
                    expression: Box::new(condition),
                    value: body,
                    return_statement,
                })
            }
            Kind::While => {
                self.eat(Kind::While)?;
                self.eat(Kind::LParen)?;
                let condition = self.parse_expression()?;
                self.eat(Kind::RParen)?;

                let body = if self.at().kind == Kind::LBrace {
                    self.eat(Kind::LBrace)?;
                    let mut statements = Vec::new();

                    while !self.is_eof() && self.at().kind != Kind::RBrace {
                        statements.push(self.parse_statement()?);
                    }

                    self.eat(Kind::RBrace)?;
                    Box::new(AstKind::Block(statements))
                } else {
                    Box::new(self.parse_statement()?)
                };

                Ok(AstKind::While {
                    condition: Box::new(condition),
                    body,
                })
            }
            Kind::Return => {
                self.eat(Kind::Return)?;
                self.eat(Kind::LParen)?;
                let expr = self.parse_expression()?;
                self.eat(Kind::RParen)?;
                if self.at().kind == Kind::Semicolon {
                    self.eat(Kind::Semicolon)?;
                }
                Ok(AstKind::Return(Box::new(expr)))
            }
            Kind::LocalVar => {
                let var = self.parse_primary_expression()?;
                if self.at().kind == Kind::Equals {
                    self.eat(Kind::Equals)?;
                    let value = self.parse_expression()?;
                    if self.at().kind == Kind::Semicolon {
                        self.eat(Kind::Semicolon)?;
                    }
                    Ok(AstKind::Assignment {
                        target: Box::new(var),
                        value: Box::new(value),
                    })
                } else {
                    Ok(var)
                }
            }
            _ => {
                let expr = self.parse_expression()?;
                if self.at().kind == Kind::Semicolon {
                    self.eat(Kind::Semicolon)?;
                }
                Ok(expr)
            }
        }
    }

    fn parse_expression(&mut self) -> Result<AstKind, SyntaxError> {
        let mut left = self.parse_additive_expression()?;

        if self.at().kind == Kind::ComparisonOperator || 
           (self.at().kind == Kind::Equals && (self.tokens.len() > 1 && self.tokens[1].kind != Kind::RParen)) {
            let operator = self.next_token().value;
            let right = self.parse_additive_expression()?;
            
            left = AstKind::BinaryExpression {
                operator,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> Result<AstKind, SyntaxError> {
        let mut left = self.parse_multiplicative_expression()?;

        while self.at().kind == Kind::BinaryOperator {
            let operator = self.next_token().value;
            let right = self.parse_multiplicative_expression()?;
            
            left = AstKind::BinaryExpression {
                lhs: Box::new(left),
                rhs: Box::new(right),
                operator,
            };
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<AstKind, SyntaxError> {
        let mut left = self.parse_primary_expression();

        match left {
            Ok(_) => {
                while !self.is_eof() && self.at().value == "*" || self.at().value == "/" {
                    let operator_token = self.next_token();
                    let right = self.parse_primary_expression();

                    left = Ok(AstKind::BinaryExpression {
                        operator: operator_token.value,
                        lhs: Box::from(left?),
                        rhs: Box::from(right?),
                    });
                }

                left
            }
            Err(e) => Err(e),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<AstKind, SyntaxError> {
        match self.at().kind {
            Kind::Number => {
                let token = self.next_token();
                let value: i32 = token.value.parse().unwrap();
                Ok(AstKind::NumericLiteral(value))
            }
            Kind::Identifier => {
                let token = self.next_token();
                if token.value == "calc" {
                    self.eat(Kind::LParen)?;
                    let expr = self.parse_additive_expression()?;
                    self.eat(Kind::RParen)?;
                    Ok(AstKind::FunctionCall {
                        name: "calc".to_string(),
                        arguments: vec![Box::new(expr)],
                    })
                } else {
                    Ok(AstKind::Identifier(token.value))
                }
            }
            Kind::Return => {
                self.eat(Kind::Return)?;
                let expression = self.parse_expression()?;
                self.eat(Kind::Semicolon)?;
                Ok(AstKind::Return(Box::from(expression)))
            }
            Kind::LocalVar => {
                self.eat(Kind::LocalVar)?;
                let identifier = self.next_token();
                Ok(AstKind::LocalVar(identifier.value))
            }
            Kind::LParen => {
                self.eat(Kind::LParen)?;
                let expr = self.parse_expression()?;
                self.eat(Kind::RParen)?;
                Ok(expr)
            }
            Kind::Equals => {
                self.eat(Kind::Equals)?;
                Ok(AstKind::AssignmentExpression)
            }
            Kind::LBracket => {
                self.eat(Kind::LBracket)?;
                let expr = self.parse_expression()?;
                self.eat(Kind::RBracket)?;
                Ok(expr)
            }
            Kind::Trigger => self.parse_trigger(),
            Kind::If => {
                self.eat(Kind::If)?;
                self.eat(Kind::LParen)?;
                let expr = self.parse_expression()?;
                self.eat(Kind::Equals)?;
                let value: AstKind = self.parse_expression()?;
                self.eat(Kind::RParen)?;
                self.eat(Kind::LBrace)?;
                let return_statement = self.parse_expression()?;
                self.eat(Kind::RBrace)?;
                Ok(AstKind::If {
                    expression: Box::from(expr),
                    value: Box::from(value),
                    return_statement: Box::from(return_statement),
                })
            }
            Kind::Command => {
                let command_name = self.next_token().value;
                self.eat(Kind::LParen)?;
                let mut arguments = Vec::new();
                
                while self.at().kind != Kind::RParen {
                    if !arguments.is_empty() {
                        self.eat(Kind::Comma)?;
                    }
                    arguments.push(Box::new(self.parse_expression()?));
                }
                
                self.eat(Kind::RParen)?;
                
                Ok(AstKind::FunctionCall {
                    name: command_name,
                    arguments,
                })
            },
            Kind::ScriptCall => {
                self.eat(Kind::ScriptCall)?;
                let script_name = self.parse_primary_expression()?;
                
                let mut arguments = Vec::new();
                if self.at().kind == Kind::LParen {
                    self.eat(Kind::LParen)?;
                    
                    while self.at().kind != Kind::RParen {
                        if !arguments.is_empty() {
                            self.eat(Kind::Comma)?;
                        }
                        arguments.push(Box::new(self.parse_expression()?));
                    }
                    
                    self.eat(Kind::RParen)?;
                }
                
                Ok(AstKind::ScriptCall {
                    script: Box::new(script_name),
                    arguments,
                })
            },
            _ => Err(SyntaxError::from_token(
                self.file_path.clone(),
                self.at(),
                format!("Unexpected token found during parsing {:?}", self.at().value),
            )),
        }
    }

    fn parse_trigger(&mut self) -> Result<AstKind, SyntaxError> {
        let next_token = &self.next_token();
        let name = next_token.value.parse::<String>().unwrap();

        match name.as_str() {
            "proc" => {
                let proc = AstKind::Proc(name);
                Ok(proc)
            }
            _ => Err(SyntaxError::from_token(
                self.file_path.clone(),
                self.at(),
                format!("Unexpected trigger type provided: {:?}", self.at().value),
            )),
        }
    }

    fn is_eof(&self) -> bool {
        self.at().kind == Kind::EOF
    }

    fn parse_numeric_literal(&mut self) -> Result<AstKind, SyntaxError> {
        let token = self.next_token();
        let number = token.value.parse::<i32>().unwrap();
        Ok(AstKind::NumericLiteral(number))
    }

    fn parse_definition(&mut self) -> Result<AstKind, SyntaxError> {
        let def_token = self.next_token();
        let var_type = self.get_type_from_def(&def_token.value)?;
        
        // Get the variable name
        let var_name = if let Kind::LocalVar = self.at().kind {
            let _token = self.next_token();
            let identifier = self.next_token();
            identifier.value
        } else {
            return Err(SyntaxError::from_token(
                self.file_path.clone(),
                self.at(),
                "Expected local variable name".to_string(),
            ));
        };

        // Check for initialization
        let initial_value = if self.at().kind == Kind::Equals {
            self.eat(Kind::Equals)?;
            let expr = self.parse_expression()?;
            if self.at().kind == Kind::Semicolon {
                self.eat(Kind::Semicolon)?;
            }
            expr
        } else {
            if self.at().kind == Kind::Semicolon {
                self.eat(Kind::Semicolon)?;
            }
            self.get_default_value_for_type(&var_type)
        };

        Ok(AstKind::Define {
            name: var_name,
            var_type,
            value: Box::new(initial_value)
        })
    }

    fn get_type_from_def(&self, def_str: &str) -> Result<Type, SyntaxError> {
        match def_str {
            "def_int" => Ok(Type::Int),
            "def_boolean" => Ok(Type::Boolean),
            "def_string" => Ok(Type::String),
            "def_loc" => Ok(Type::Loc),
            "def_npc" => Ok(Type::Npc),
            "def_obj" => Ok(Type::Obj),
            "def_coord" => Ok(Type::Coord),
            "def_namedobj" => Ok(Type::NamedObj),
            "def_playeruid" => Ok(Type::PlayerUid),
            "def_npcuid" => Ok(Type::NpcUid),
            "def_stat" => Ok(Type::Stat),
            "def_component" => Ok(Type::Component),
            "def_interface" => Ok(Type::Interface),
            "def_inv" => Ok(Type::Inv),
            "def_enum" => Ok(Type::Enum),
            "def_struct" => Ok(Type::Struct),
            "def_param" => Ok(Type::Param),
            "def_dbtable" => Ok(Type::DbTable),
            "def_dbrow" => Ok(Type::DbRow),
            "def_dbcolumn" => Ok(Type::DbColumn),
            "def_varp" => Ok(Type::Varp),
            "def_mesanim" => Ok(Type::MesAnim),
            _ => Err(SyntaxError::from_token(
                self.file_path.clone(),
                self.at(),
                format!("Unknown type definition: {}", def_str),
            )),
        }
    }

    fn get_default_value_for_type(&self, var_type: &Type) -> AstKind {
        match var_type {
            Type::Int => AstKind::NumericLiteral(0),
            Type::Boolean => AstKind::NumericLiteral(0), // false
            Type::String => AstKind::StringLiteral(String::new()),
            // Add default values for other types...
            _ => AstKind::NumericLiteral(0), // temporary default
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigType {
    Floor,      // .flo
    IdKit,      // .idk
    Location,   // .loc 
    Npc,        // .npc
    Object,     // .obj
    Sequence,   // .seq
    Spotanim,   // .spotanim
    Varp,       // .varp
    Param,      // .param
    Enum,       // .enum
    Struct      // .struct
}
