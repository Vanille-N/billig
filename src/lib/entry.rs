use std::fmt;

use crate::lib::date::Date;

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

    pub fn period(&self, dt: Date) -> (Date, Date) {
        use Duration::*;
        use Window::*;
        let nb = self.count as isize;
        match (self.duration, self.window) {
            (Day, Precedent) => (dt.jump_day(nb), dt.prev()),
            (Day, Successor) => (dt.next(), dt.jump_day(nb)),
            (Day, Anterior) => (dt.jump_day(-nb).next(), dt),
            (Day, _) => (dt, dt.jump_day(nb).prev()),
            (Week, Current) => (dt.start_of_week(), dt.end_of_week().jump_day(7 * (nb - 1))),
            (Week, Anterior) => (dt.jump_day(-7 * nb).next(), dt),
            (Week, Posterior) => (dt, dt.jump_day(7 * nb).prev()),
            (Week, Precedent) => {
                let d = dt.start_of_week();
                (d.jump_day(-7 * nb), d.prev())
            }
            (Week, Successor) => {
                let d = dt.end_of_week();
                (d.next(), d.jump_day(7 * nb))
            }
            (Month, Current) => (dt.start_of_month(), dt.end_of_month().jump_month(nb - 1)),
            (Month, Posterior) => (dt, dt.jump_month(nb).prev()),
            (Month, Anterior) => (dt.jump_month(-nb).next(), dt),
            (Month, Precedent) => {
                let d = dt.start_of_month();
                (d.jump_month(-nb), d.prev())
            }
            (Month, Successor) => {
                let d = dt.start_of_month();
                (d.jump_month(-nb), d.prev())
            }
            (Year, Current) => (dt.start_of_year(), dt.end_of_year().jump_year(nb - 1)),
        }
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

