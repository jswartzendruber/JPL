use crate::{
    lexer::{Token, TokenContents},
    JPLError,
};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

#[derive(Debug)]
pub enum ParsedStatement {
    VarDecl(String, ParsedExpr),
    FunctionCall(String, Vec<ParsedExpr>),
    FunctionDeclaration(String, String, Vec<ParsedStatement>),
    ReturnStatement(ParsedExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedExpr {
    IntegerConstant(i64),
    FloatConstant(f64),
    BinaryOp(Box<ParsedExpr>, BinaryOperator, Box<ParsedExpr>),
    QuotedString(String),
    Var(String),
    FunctionCall(String, Vec<ParsedExpr>),
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
        Self { tokens, idx: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<ParsedStatement>, JPLError> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<ParsedStatement, JPLError> {
        match &self.current().contents {
            TokenContents::Name(n) => {
                if n.eq_ignore_ascii_case("let") {
                    self.advance();
                    self.var_declaration()
                } else if n.eq_ignore_ascii_case("function") {
                    self.advance();
                    self.function_declaration()
                } else if self.peek().contents == TokenContents::LParen {
                    self.function_call()
                } else {
                    Err(JPLError::new(
                        "unrecognized literal".to_string(),
                        self.current().line,
                    ))
                }
            }
            _ => Err(JPLError::new(
                "Expected thing".to_string(),
                self.current().line,
            )),
        }
    }

    fn function_call(&mut self) -> Result<ParsedStatement, JPLError> {
        let name = match &self.advance().contents {
            TokenContents::Name(n) => Ok(n.clone()),
            _ => Err(JPLError::new(
                "Expected function name.".to_string(),
                self.current().line,
            )),
        }?;
        self.expect(TokenContents::LParen, "Expected left parenthesis.")?;

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

        self.expect(TokenContents::RParen, "Expected right parenthesis.")?;

        Ok(ParsedStatement::FunctionCall(name, args))
    }

    fn function_declaration(&mut self) -> Result<ParsedStatement, JPLError> {
        let function_name = match &self.advance().contents {
            TokenContents::Name(s) => Ok(s.clone()),
            _ => Err(JPLError::new(
                "Expected function name.".to_string(),
                self.previous().line,
            )),
        }?;

        self.expect(TokenContents::LParen, "Expected left parenthesis.")?;

        let argument_name = match &self.advance().contents {
            TokenContents::Name(s) => Ok(s.clone()),
            _ => Err(JPLError::new(
                "Expected argument name.".to_string(),
                self.previous().line,
            )),
        }?;

        self.expect(TokenContents::RParen, "Expected right parenthesis.")?;
        self.expect(TokenContents::LCurly, "Expected left curly brace.")?;

        let mut function_body = vec![];
        while self.current().contents != TokenContents::RCurly {
            match &self.current().contents {
                TokenContents::Name(n) => {
                    if n.eq_ignore_ascii_case("return") {
                        self.advance();
                        let expr = self.expression()?;
                        function_body.push(ParsedStatement::ReturnStatement(expr));
                    }
                }
                _ => function_body.push(self.statement()?),
            }
        }

        self.expect(TokenContents::RCurly, "Expected right curly brace.")?;

        Ok(ParsedStatement::FunctionDeclaration(
            function_name,
            argument_name,
            function_body,
        ))
    }

    fn var_declaration(&mut self) -> Result<ParsedStatement, JPLError> {
        let name = match &self.advance().contents {
            TokenContents::Name(name) => Ok(name.clone()),
            _ => Err(JPLError::new(
                "Expected variable name.".to_string(),
                self.previous().line,
            )),
        }?;

        self.expect(TokenContents::Equal, "Expected equals sign.")?;

        let expr = self.expression()?;

        Ok(ParsedStatement::VarDecl(name, expr))
    }

    fn expression(&mut self) -> Result<ParsedExpr, JPLError> {
        let mut lhs = self.term()?;

        while let TokenContents::Plus | TokenContents::Minus = self.current().contents {
            let op = match self.advance().contents {
                TokenContents::Plus => BinaryOperator::Add,
                TokenContents::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let rhs = self.term()?;
            lhs = ParsedExpr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<ParsedExpr, JPLError> {
        let mut lhs = self.factor()?;

        while let TokenContents::Star | TokenContents::Slash = self.current().contents {
            let op = match self.advance().contents {
                TokenContents::Star => BinaryOperator::Multiply,
                TokenContents::Slash => BinaryOperator::Divide,
                _ => unreachable!(),
            };
            let rhs = self.factor()?;
            lhs = ParsedExpr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
        }

        Ok(lhs)
    }

    fn factor(&mut self) -> Result<ParsedExpr, JPLError> {
        match &self.current().contents {
            TokenContents::Integer(i) => {
                let num = i.clone();
                self.advance();
                Ok(ParsedExpr::IntegerConstant(num))
            }
            TokenContents::Float(f) => {
                let num = f.clone();
                self.advance();
                Ok(ParsedExpr::FloatConstant(num))
            }
            TokenContents::Name(s) => {
                if self.peek().contents == TokenContents::LParen {
                    if let ParsedStatement::FunctionCall(name, args) = self.function_call()? {
                        Ok(ParsedExpr::FunctionCall(name, args))
                    } else {
                        Err(JPLError::new(
                            "Expected function call.".to_string(),
                            self.current().line,
                        ))
                    }
                } else {
                    let name = s.clone();
                    self.advance();
                    Ok(ParsedExpr::Var(name))
                }
            }
            TokenContents::LParen => {
                let expr = self.expression()?;
                match &self.advance().contents {
                    TokenContents::RParen => Ok(expr),
                    _ => Err(JPLError::new(
                        "Expected closing parenthesis.".to_string(),
                        self.previous().line,
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

    fn expect(&mut self, expected: TokenContents, err_msg: &str) -> Result<&Token, JPLError> {
        if self.current().contents == expected {
            Ok(self.advance())
        } else {
            Err(JPLError::new(err_msg.to_string(), self.previous().line))
        }
    }

    fn advance(&mut self) -> &Token {
        println!("eat {:?}", self.current());
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
