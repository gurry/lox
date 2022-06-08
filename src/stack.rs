
use anyhow::{Result, bail};
#[derive(Debug)]
pub struct Stack<T>(Vec<T>);

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, item :T) {
        self.0.push(item)
    }

    pub fn pop(&mut self) -> Result<T> {
        if self.0.is_empty() {
            bail!("Stack underflow");
        }

        Ok(self.0.pop().unwrap())
    }

    pub fn peek(&self, pos: usize) -> Result<&T> 
    {
        let index = self.0.len() - (pos + 1);

        if index < 0 {
            bail!("Stack underflow");
        }

        Ok(&self.0[index])
    }
}