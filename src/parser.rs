use crate::{
    lexer::{Token, TokenContents},
    JPLError,
};

pub struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, idx: 0 }
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
        match &self.current().contents {
            TokenContents::Name(_) => {
                self.advance();
                Ok(())
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

        match self.current().contents {
            TokenContents::QuotedString(_) => {
                self.advance();
                Ok(())
            }
            TokenContents::Number(_) => self.expression(),
            _ => Err(JPLError::new(String::from("Expected literal value."))),
        }?;

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
            TokenContents::Number(_) => self.expression(),
            _ => Err(JPLError::new(String::from("Expected variable or literal."))),
        }
    }

    fn expression(&mut self) -> Result<(), JPLError> {
        self.term()?;

        match self.current().contents {
            TokenContents::Plus => {
                self.advance();
                self.expression()?;
                Ok(())
            }
            TokenContents::Minus => {
                self.advance();
                self.expression()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn term(&mut self) -> Result<(), JPLError> {
        self.factor()?;

        match self.current().contents {
            TokenContents::Star => {
                self.advance();
                self.term()?;
                Ok(())
            }
            TokenContents::Slash => {
                self.advance();
                self.term()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn factor(&mut self) -> Result<(), JPLError> {
        match &self.current().contents {
            TokenContents::Number(_) => {
                self.advance();
                Ok(())
            }
            TokenContents::LParen => {
                self.advance();
                self.expression()?;
                match self.current().contents {
                    TokenContents::RParen => {
                        self.advance();
                        Ok(())
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
