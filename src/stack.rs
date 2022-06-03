
#[derive(Debug)]
pub struct Stack<T>(Vec<T>);

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, item :T) {
        self.0.push(item)
    }

    pub fn pop(&mut self, item :T) -> Option<T> {
        if self.0.is_empty() {
            return None;
        }

        self.0.pop()
    }
}