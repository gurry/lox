use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}", self.lexeme, self.line)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen, RightParen, LeftBrace, RightBrace, Comma,
    Dot, Minus, Plus, Semicolon, Slash, Star,

    Bang, BangEqual, Equal, EqualEqual, Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, String(String), Number(f64),

    And, Class, Else, False, Fun, For, If, Nil, Or, Print,
    Return, Super, This, True, Var, While,

    Eof
}