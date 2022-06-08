
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
        if (pos + 1) > self.0.len() {
            bail!("Stack underflow");
        }

        let index = self.0.len() - (pos + 1);

        Ok(&self.0[index])
    }


    pub fn peek_front(&self, pos: usize) -> Result<&T> {
        if pos  >= self.0.len() {
            bail!("Stack overflow");
        }

        Ok(&self.0[pos])
    }

    pub fn set_front(&mut self, pos: usize, value: T) -> Result<()> {
        if pos  >= self.0.len() {
            bail!("Stack overflow");
        }

        self.0[pos] = value;

        Ok(())
    }
}