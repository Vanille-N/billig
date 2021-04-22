use std::fmt;

pub mod fields {
    pub use super::{
        Amount,
        Tag,
        Category,
        Span,
        Window,
        Duration,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Amount(pub isize);

#[derive(Debug, Clone)]
pub struct Tag(pub String);

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}E", self.0 / 100, self.0 % 100)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: Amount,
    pub cat: Category,
    pub span: Span,
    pub tag: Tag,
}

#[derive(Debug, Clone, Copy)]
pub enum Category {
    School,
    Food,
    Home,
    Salary,
    Communication,
    Movement,
    Cleaning,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub duration: Duration,
    pub window: Window,
    pub count: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Duration {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy)]
pub enum Window {
    Current,
    Posterior,
    Anterior,
    Precedent,
    Successor,
}
