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
}

#[derive(Debug)]
pub struct ParsedVarDecl {
    name: String,
}

#[derive(Debug)]
pub enum ParsedExpr {
    NumericConstant(NumberContents),
    BinaryOp(Box<ParsedExpr>, BinaryOperator, Box<ParsedExpr>),
    QuotedString(String),
    Var(String),
}

#[derive(Debug)]
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
            TokenContents::Name(n) => {
                if n.eq_ignore_ascii_case("print") {
                    self.advance();

                    match &self.current().contents {
                        TokenContents::LParen => {
                            self.advance();
                            Ok(())
                        }
                        _ => Err(JPLError::new(String::from("Expected left parenthesis."))),
                    }?;

                    match &self.current().contents {
                        TokenContents::Name(_) | TokenContents::Number(_) => {
                            self.advance();
                            Ok(())
                        }
                        _ => Err(JPLError::new(String::from("Expected variable or value."))),
                    }?;

                    match &self.current().contents {
                        TokenContents::RParen => {
                            self.advance();
                            Ok(())
                        }
                        _ => Err(JPLError::new(String::from("Expected right parenthesis."))),
                    }?;
                }
                return Ok(());
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
