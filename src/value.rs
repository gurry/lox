use std::fmt::Display;

use crate::scanner::TokenType;

#[derive(Debug,Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Nil,
    Boolean(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Nil => write!(f, "{}", "Nil"),
            Value::Boolean(b) => write!(f, "{}", b),
        }?;

        Ok(())
    }
}