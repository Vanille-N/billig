use crate::extract::{Amount, Tag};

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
