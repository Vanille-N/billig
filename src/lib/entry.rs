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
    value: Amount,
    cat: Category,
    span: Span,
    tag: Tag,
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
    duration: Duration,
    window: Window,
    count: usize,
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

impl Entry {
    pub fn from(value: Amount, cat: Category, span: Span, tag: Tag) -> Self {
        Self { value, cat, span, tag }
    }
}

impl Span {
    pub fn from(duration: Duration, window: Window, count: usize) -> Self {
        Self { duration, window, count }
    }
}

impl Category {
    pub fn from(s: &str) -> Option<Self> {
        use Category::*;
        Some(match s {
            "Pay" => Salary,
            "Food" => Food,
            "Tech" => Tech,
            "Mov" => Movement,
            "Pro" => School,
            "Clean" => Cleaning,
            "Home" => Home,
            _ => return None,
        })
    }
}

impl Duration {
    pub fn from(s: &str) -> Option<Self> {
        use Duration::*;
        Some(match s {
            "Day" => Day,
            "Week" => Week,
            "Month" => Month,
            "Year" => Year,
            _ => return None,
        })
    }
}

impl Window {
    pub fn from(s: &str) -> Option<Self> {
        use Window::*;
        Some(match s {
            "Curr" => Current,
            "Post" => Posterior,
            "Ante" => Anterior,
            "Pred" => Precedent,
            "Succ" => Successor,
            _ => return None,
        })
    }
}

