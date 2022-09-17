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
        match &self.peek().contents {
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
        match &self.peek().contents {
            TokenContents::Name(_) => Ok(()),
            _ => Err(JPLError::new(String::from("Expected variable name."))),
        }?;
        self.advance();

        match &self.peek().contents {
            TokenContents::Equal => Ok(()),
            _ => Err(JPLError::new(String::from("Expected equals sign."))),
        }?;
        self.advance();

        match self.peek().contents {
            TokenContents::QuotedString(_) => Ok(()),
            TokenContents::Number(_) => Ok(()),
            _ => Err(JPLError::new(String::from("Expected literal value."))),
        }?;
        self.advance();

        Ok(())
    }

    fn statement(&mut self) -> Result<(), JPLError> {
        match &self.peek().contents {
            TokenContents::Name(n) => {
                if n.eq_ignore_ascii_case("print") {
                    self.advance();

                    match &self.peek().contents {
                        TokenContents::LParen => Ok(()),
                        _ => Err(JPLError::new(String::from("Expected left parenthesis."))),
                    }?;
                    self.advance();

                    match &self.peek().contents {
                        TokenContents::Name(_) => Ok(()),
                        _ => Err(JPLError::new(String::from("Expected variable."))),
                    }?;
                    self.advance();

                    match &self.peek().contents {
                        TokenContents::RParen => Ok(()),
                        _ => Err(JPLError::new(String::from("Expected right parenthesis."))),
                    }?;
                    self.advance();
                }
                return Ok(());
            }
            TokenContents::QuotedString(_) => Ok(()),
            TokenContents::Number(_) => Ok(()),
            _ => Err(JPLError::new(String::from("Expected variable or literal."))),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.idx]
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
        match self.peek().contents {
            TokenContents::Eof => true,
            _ => false,
        }
    }
}
