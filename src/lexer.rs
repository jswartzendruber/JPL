#[derive(Debug)]
pub struct Token {
    contents: TokenContents,
    span: Span,
}

#[derive(Debug)]
pub enum TokenContents {
    Plus,
    Minus,

    Equal,

    LParen,
    RParen,

    Number(i64),
    QuotedString(String),

    Name(String),

    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl Token {
    pub fn new(contents: TokenContents, span: Span) -> Self {
        Self { contents, span }
    }
}

pub fn lex(bytes: &[u8]) -> Vec<Token> {
    let mut tokens = vec![];
    let mut index = 0;
    let mut line = 1;

    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }

            let num = match String::from_utf8_lossy(&bytes[start..index]).parse() {
                Ok(n) => n,
                Err(_) => panic!("Bad number on line {}", line),
            };

            tokens.push(Token::new(
                TokenContents::Number(num),
                Span::new(start, index - 1),
            ))
        } else if bytes[index].is_ascii_alphanumeric() {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_alphanumeric() {
                index += 1;
            }

            tokens.push(Token::new(
                TokenContents::Name(String::from(String::from_utf8_lossy(&bytes[start..index]))),
                Span::new(start, index - 1),
            ))
        } else if bytes[index] == b'"' {
            index += 1;
            let start = index;

            while index < bytes.len() - 1 && bytes[index] != b'"' {
                index += 1;
            }

            if bytes[index] != b'"' {
                panic!("Unterminated string on line {}", line)
            }

            tokens.push(Token::new(
                TokenContents::QuotedString(String::from(String::from_utf8_lossy(
                    &bytes[start..index],
                ))),
                Span::new(start, index - 1),
            ));

            index += 1;
        } else if bytes[index] == b'+' {
            tokens.push(Token::new(TokenContents::Plus, Span::new(index, index)));
            index += 1;
        } else if bytes[index] == b'=' {
            tokens.push(Token::new(TokenContents::Equal, Span::new(index, index)));
            index += 1;
        } else if bytes[index] == b'(' {
            tokens.push(Token::new(TokenContents::LParen, Span::new(index, index)));
            index += 1;
        } else if bytes[index] == b')' {
            tokens.push(Token::new(TokenContents::RParen, Span::new(index, index)));
            index += 1;
        } else if bytes[index].is_ascii_whitespace() {
            if bytes[index] == b' ' || bytes[index] == b'\t' || bytes[index] == b'\r' {
                index += 1;
            } else if bytes[index] == b'\n' {
                line += 1;
                index += 1;
            }
        } else {
            panic!(
                "unexpected token '{}' on line {}",
                bytes[index] as char, line
            )
        }
    }

    tokens.push(Token::new(TokenContents::Eof, Span::new(index, index)));

    tokens
}
