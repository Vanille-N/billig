use crate::extract::{Tag, Amount};

#[derive(Debug)]
pub struct Entry {
    pub value: Amount,
    pub cat: Category,
    pub span: Span,
    pub tag: Tag,
}

#[derive(Debug)]
pub enum Category {
    School,
    Food,
    Home,
    Salary,
    Communication,
    Movement,
    Cleaning,
}

#[derive(Debug)]
pub struct Span {
    pub duration: Duration,
    pub window: Window,
    pub count: usize,
}

#[derive(Debug)]
pub enum Duration {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug)]
pub enum Window {
    Current,
    Posterior,
    Anterior,
    Precedent,
    Successor,
}
