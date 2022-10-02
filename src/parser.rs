use std::collections::HashMap;

use crate::{
    lexer::{NumberContents, Token, TokenContents},
    JPLError,
};

#[derive(Debug)]
pub struct Parser {
    pub statements: Vec<ParsedStatement>,
    tokens: Vec<Token>,
    idx: usize,
}

#[derive(Debug)]
pub enum ParsedStatement {
    Expression(ParsedExpr),
    VarDecl(ParsedVarDecl, ParsedExpr),
    FunctionCall(String, Vec<ParsedExpr>),
}

#[derive(Debug)]
pub struct ParsedVarDecl {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ParsedExpr {
    NumericConstant(NumberContents),
    BinaryOp(Box<ParsedExpr>, BinaryOperator, Box<ParsedExpr>),
    QuotedString(String),
    Var(String),
}

impl ParsedExpr {
    pub fn evaluate_expr_to_string(
        expr: &ParsedExpr,
        var_table: &HashMap<String, ParsedExpr>,
    ) -> String {
        match expr {
            ParsedExpr::BinaryOp(_, _, _) | ParsedExpr::NumericConstant(_) => {
                match Self::evaluate_expr(expr, var_table) {
                    NumberContents::Integer(i) => i.to_string(),
                    NumberContents::Floating(f) => f.to_string(),
                }
            }
            ParsedExpr::QuotedString(s) => s.to_string(),
            ParsedExpr::Var(v) => v.to_string(),
        }
    }

    pub fn evaluate_expr(expr: &ParsedExpr, var_table: &HashMap<String, ParsedExpr>) -> NumberContents {
        match expr {
            ParsedExpr::NumericConstant(n) => n.clone(),
            ParsedExpr::BinaryOp(n1, op, n2) => match op {
                BinaryOperator::Add => {
                    Self::evaluate_expr(n1, var_table) + Self::evaluate_expr(n2, var_table)
                }
                BinaryOperator::Subtract => {
                    Self::evaluate_expr(n1, var_table) - Self::evaluate_expr(n2, var_table)
                }
                BinaryOperator::Multiply => {
                    Self::evaluate_expr(n1, var_table) * Self::evaluate_expr(n2, var_table)
                }
                BinaryOperator::Divide => {
                    Self::evaluate_expr(n1, var_table) / Self::evaluate_expr(n2, var_table)
                }
            },
            ParsedExpr::QuotedString(_) => todo!(),
            ParsedExpr::Var(var_name) => Self::evaluate_expr(
                var_table.get(var_name).expect("Could not find variable"),
                var_table,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            statements: vec![],
            tokens,
            idx: 0,
        }
    }

    pub fn parse(&mut self) -> Result<(), JPLError> {
        while !self.is_at_end() {
            self.declaration()?;
        }

        Ok(())
    }

    fn declaration(&mut self) -> Result<(), JPLError> {
        match &self.current().contents {
            TokenContents::Name(n) => {
                if n.eq_ignore_ascii_case("let") {
                    self.advance();
                    self.var_declaration()?;
                }
            }
            _ => {}
        }

        self.statement()?;

        Ok(())
    }

    fn function_call(&mut self) -> Result<(), JPLError> {
        let name = match &self.current().contents {
            TokenContents::Name(n) => {
                let name = n.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(JPLError::new(String::from("Expected function name."))),
        }?;

        match &self.current().contents {
            TokenContents::LParen => {
                self.advance();
                Ok(())
            }
            _ => Err(JPLError::new(String::from("Expected left parenthesis."))),
        }?;

        // TODO: handle more than one argument
        let mut args = vec![];
        match &self.current().contents {
            TokenContents::QuotedString(s) => {
                args.push(ParsedExpr::QuotedString(s.to_string()));
                self.advance();
                Ok(())
            }
            _ => match self.expression() {
                Ok(expr) => {
                    args.push(expr);
                    Ok(())
                }
                Err(_) => Err(JPLError::new(String::from("Expected expression."))),
            },
        }?;

        match &self.current().contents {
            TokenContents::RParen => {
                self.advance();
                Ok(())
            }
            _ => Err(JPLError::new(String::from("Expected right parenthesis."))),
        }?;

        self.statements
            .push(ParsedStatement::FunctionCall(name.to_string(), args));
        Ok(())
    }

    fn var_declaration(&mut self) -> Result<(), JPLError> {
        let decl = match &self.current().contents {
            TokenContents::Name(n) => {
                let name = n.clone();
                self.advance();
                Ok(ParsedVarDecl { name })
            }
            _ => Err(JPLError::new(String::from("Expected variable name."))),
        }?;

        match &self.current().contents {
            TokenContents::Equal => {
                self.advance();
                Ok(())
            }
            _ => Err(JPLError::new(String::from("Expected equals sign."))),
        }?;

        let expr = match &self.current().contents {
            TokenContents::QuotedString(s) => {
                let quoted_string = s.clone();
                self.advance();
                Ok(ParsedExpr::QuotedString(quoted_string))
            }
            TokenContents::Number(_) | TokenContents::LParen => Ok(self.expression()?),
            _ => Err(JPLError::new(String::from("Expected literal value."))),
        }?;

        self.statements.push(ParsedStatement::VarDecl(decl, expr));

        Ok(())
    }

    fn statement(&mut self) -> Result<(), JPLError> {
        match &self.current().contents {
            TokenContents::Name(_) => {
                if &self.peek().contents == &TokenContents::LParen {
                    self.function_call()
                } else {
                    Ok(())
                }
            }
            TokenContents::Number(_) => {
                self.expression()?;
                Ok(())
            }
            _ => Err(JPLError::new(String::from("Expected variable or literal."))),
        }
    }

    fn expression(&mut self) -> Result<ParsedExpr, JPLError> {
        let lhs = self.term()?;

        match self.current().contents {
            TokenContents::Plus => {
                self.advance();
                let rhs = self.expression()?;

                Ok(ParsedExpr::BinaryOp(
                    Box::new(lhs),
                    BinaryOperator::Add,
                    Box::new(rhs),
                ))
            }
            TokenContents::Minus => {
                self.advance();
                let rhs = self.expression()?;

                Ok(ParsedExpr::BinaryOp(
                    Box::new(lhs),
                    BinaryOperator::Subtract,
                    Box::new(rhs),
                ))
            }
            _ => Ok(lhs),
        }
    }

    fn term(&mut self) -> Result<ParsedExpr, JPLError> {
        let lhs = self.factor()?;

        match self.current().contents {
            TokenContents::Star => {
                self.advance();
                let rhs = self.term()?;

                Ok(ParsedExpr::BinaryOp(
                    Box::new(lhs),
                    BinaryOperator::Multiply,
                    Box::new(rhs),
                ))
            }
            TokenContents::Slash => {
                self.advance();
                let rhs = self.term()?;

                Ok(ParsedExpr::BinaryOp(
                    Box::new(lhs),
                    BinaryOperator::Divide,
                    Box::new(rhs),
                ))
            }
            _ => Ok(lhs),
        }
    }

    fn factor(&mut self) -> Result<ParsedExpr, JPLError> {
        match &self.current().contents {
            TokenContents::Number(n) => {
                let number = n.clone();
                self.advance();
                Ok(ParsedExpr::NumericConstant(number))
            }
            TokenContents::Name(n) => {
                let name = n.clone();
                self.advance();
                Ok(ParsedExpr::Var(name))
            }
            TokenContents::LParen => {
                self.advance();
                let expr = self.expression()?;
                match self.current().contents {
                    TokenContents::RParen => {
                        self.advance();
                        Ok(expr)
                    }
                    _ => Err(JPLError::new(String::from("Expected closing parenthesis."))),
                }
            }
            _ => Err(JPLError::new(String::from(
                "Expected parenthesis or number.",
            ))),
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.idx]
    }

    fn peek(&self) -> &Token {
        match self.current().contents {
            TokenContents::Eof => self.current(),
            _ => &self.tokens[self.idx + 1],
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.idx += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.idx - 1]
    }

    fn is_at_end(&self) -> bool {
        match self.current().contents {
            TokenContents::Eof => true,
            _ => false,
        }
    }
}
