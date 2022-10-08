use crate::{
    lexer::{Token, TokenContents},
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
    VarDecl(ParsedVarDecl, ParsedExpr),
    FunctionCall(String, Vec<ParsedExpr>),
}

#[derive(Debug)]
pub struct ParsedVarDecl {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedExpr {
    IntegerConstant(i64),
    FloatConstant(f64),
    BinaryOp(Box<ParsedExpr>, BinaryOperator, Box<ParsedExpr>),
    QuotedString(String),
    Var(String),
}

#[derive(Debug, Clone, PartialEq)]
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
            _ => Err(JPLError::new(
                "Expected function name.".to_string(),
                self.current().line,
            )),
        }?;

        match &self.current().contents {
            TokenContents::LParen => {
                self.advance();
                Ok(())
            }
            _ => Err(JPLError::new(
                "Expected left parenthesis.".to_string(),
                self.current().line,
            )),
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
                Err(_) => Err(JPLError::new(
                    "Expected expression.".to_string(),
                    self.current().line,
                )),
            },
        }?;

        match &self.current().contents {
            TokenContents::RParen => {
                self.advance();
                Ok(())
            }
            _ => Err(JPLError::new(
                "Expected right parenthesis.".to_string(),
                self.current().line,
            )),
        }?;

        self.statements
            .push(ParsedStatement::FunctionCall(name, args));
        Ok(())
    }

    fn var_declaration(&mut self) -> Result<(), JPLError> {
        let decl = match &self.current().contents {
            TokenContents::Name(n) => {
                let name = n.clone();
                self.advance();
                Ok(ParsedVarDecl { name })
            }
            _ => Err(JPLError::new(
                "Expected variable name.".to_string(),
                self.current().line,
            )),
        }?;

        match &self.current().contents {
            TokenContents::Equal => {
                self.advance();
                Ok(())
            }
            _ => Err(JPLError::new(
                "Expected equals sign.".to_string(),
                self.current().line,
            )),
        }?;

        let expr = self.expression()?;

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
            TokenContents::Integer(_) | TokenContents::Float(_) => {
                self.expression()?;
                Ok(())
            }
            _ => Err(JPLError::new(
                "Expected variable or literal.".to_string(),
                self.current().line,
            )),
        }
    }

    fn expression(&mut self) -> Result<ParsedExpr, JPLError> {
        let mut lhs = self.term()?;

        while let TokenContents::Plus | TokenContents::Minus = self.current().contents {
            let op = match self.current().contents {
                TokenContents::Plus => BinaryOperator::Add,
                TokenContents::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let rhs = match self.current().contents {
                TokenContents::Plus | TokenContents::Minus => {
                    self.advance();
                    self.term()?
                }
                _ => lhs.clone(),
            };

            lhs = ParsedExpr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<ParsedExpr, JPLError> {
        let mut lhs = self.factor()?;

        while let TokenContents::Star | TokenContents::Slash = self.current().contents {
            let op = match self.current().contents {
                TokenContents::Star => BinaryOperator::Multiply,
                TokenContents::Slash => BinaryOperator::Divide,
                _ => unreachable!(),
            };
            let rhs = match self.current().contents {
                TokenContents::Star | TokenContents::Slash => {
                    self.advance();
                    self.factor()?
                }
                _ => lhs.clone(),
            };

            lhs = ParsedExpr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
        }

        Ok(lhs)
    }

    fn factor(&mut self) -> Result<ParsedExpr, JPLError> {
        match &self.advance().contents {
            TokenContents::Integer(i) => Ok(ParsedExpr::IntegerConstant(*i)),
            TokenContents::Float(f) => Ok(ParsedExpr::FloatConstant(*f)),
            TokenContents::Name(s) => Ok(ParsedExpr::Var(s.to_string())),
            TokenContents::LParen => {
                let expr = self.expression()?;
                match &self.advance().contents {
                    TokenContents::RParen => Ok(expr),
                    _ => Err(JPLError::new(
                        "Expected closing parenthesis.".to_string(),
                        self.current().line,
                    )),
                }
            }
            _ => Err(JPLError::new(
                "Expected parenthesis or number.".to_string(),
                self.current().line,
            )),
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
