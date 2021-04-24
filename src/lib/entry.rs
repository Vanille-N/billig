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
pub struct Amount(isize);

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
    Tech,
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

impl Amount {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn from(i: isize) -> Self {
        Self(i)
    }
}

use std::ops;
impl ops::Add for Amount {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl ops::AddAssign for Amount {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl ops::Neg for Amount {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
