use crate::extract::{Tag, Amount};

#[derive(Debug)]
pub struct Entry {
    value: Amount,
    cat: Category,
    span: Span,
    tag: Tag,
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
    duration: Duration,
    window: Window,
    count: usize,
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
