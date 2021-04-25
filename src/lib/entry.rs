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
    /// cached for performance
    period: (Date, Date),
    length: usize,
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
    pub fn from(date: Date, value: Amount, cat: Category, span: Span, tag: Tag) -> Self {
        let period = span.period(date);
        let length = period.1.index() - period.0.index() + 1;
        Self {
            value,
            cat,
            span,
            tag,
            period,
            length,
        }
    }
}
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
            (Day, Precedent) => (dt.jump_day(-nb), dt.prev()),
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
            (Month, Current) => (dt.start_of_month(), dt.jump_month(nb - 1).end_of_month()),
            (Month, Posterior) => (dt, dt.jump_month(nb).cap_day(dt.day())),
            (Month, Anterior) => (dt.jump_month(-nb).next(), dt),
            (Month, Precedent) => {
                let d = dt.start_of_month();
                (d.jump_month(-nb), d.prev())
            }
            (Month, Successor) => {
                let d = dt.end_of_month();
                (d.next(), d.jump_month(nb))
            }
            (Year, Current) => (dt.start_of_year(), dt.end_of_year().jump_year(nb - 1)),
            (Year, Posterior) => (dt, dt.jump_year(nb).cap_day(dt.day())),
            (Year, Anterior) => (dt.jump_year(-nb).next(), dt),
            (Year, Successor) => {
                let d = dt.end_of_year();
                (d.next(), d.jump_year(nb))
            }
            (Year, Precedent) => {
                let d = dt.start_of_year();
                (d.jump_year(-nb), d.prev())
            }
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

#[cfg(test)]
mod test {
    use super::{
        *,
        Window::*,
        Duration::*,
    };
    use crate::lib::date::{
        Month::*,
        Date,
    };
        
    macro_rules! dt {
        ( $y:tt - $m:tt - $d:tt ) => {
            Date::from($y, $m, $d).unwrap()
        }
    }
    macro_rules! span {
        ( $dur:tt < $win:tt > $nb:tt ) => {
            Span {
                duration: $dur,
                window: $win,
                count: $nb,
            }
        }
    }

    macro_rules! check {
        ( $date:expr, $span:expr, $start:expr, $end:expr ) => {
            assert_eq!($span.period($date), ($start, $end));
        }
    }

    #[test]
    fn day_jumps() {
        check!(dt!(2020-May-8), span!(Day<Current>3), dt!(2020-May-8), dt!(2020-May-10));
        check!(dt!(2020-Sep-1), span!(Day<Precedent>5), dt!(2020-Aug-27), dt!(2020-Aug-31));
        check!(dt!(2020-Dec-30), span!(Day<Anterior>2), dt!(2020-Dec-29), dt!(2020-Dec-30));
        check!(dt!(2020-Jan-1), span!(Day<Posterior>50), dt!(2020-Jan-1), dt!(2020-Feb-19));
        check!(dt!(2020-Feb-28), span!(Day<Successor>3), dt!(2020-Feb-29), dt!(2020-Mar-2));
    }

    #[test]
    fn week_jumps() {
        check!(dt!(2020-Mar-12), span!(Week<Current>2), dt!(2020-Mar-9), dt!(2020-Mar-22));
        check!(dt!(2020-Sep-8), span!(Week<Precedent>3), dt!(2020-Aug-17), dt!(2020-Sep-6));
        check!(dt!(2020-Aug-9), span!(Week<Successor>6), dt!(2020-Aug-10), dt!(2020-Sep-20));
        check!(dt!(2020-May-23), span!(Week<Anterior>4), dt!(2020-Apr-26), dt!(2020-May-23));
        check!(dt!(2020-Dec-30), span!(Week<Posterior>1), dt!(2020-Dec-30), dt!(2021-Jan-5));
    }

    #[test]
    fn month_jumps() {
        check!(dt!(2020-May-31), span!(Month<Current>5), dt!(2020-May-1), dt!(2020-Sep-30));
        check!(dt!(2020-Feb-29), span!(Month<Current>2), dt!(2020-Feb-1), dt!(2020-Mar-31));
        check!(dt!(2020-Feb-29), span!(Month<Posterior>12), dt!(2020-Feb-29), dt!(2021-Feb-28));
        check!(dt!(2020-Feb-28), span!(Month<Posterior>1), dt!(2020-Feb-28), dt!(2020-Mar-27));
        check!(dt!(2020-Aug-15), span!(Month<Successor>3), dt!(2020-Sep-1), dt!(2020-Nov-30));
        check!(dt!(2020-Jan-31), span!(Month<Successor>4), dt!(2020-Feb-1), dt!(2020-May-31));
        check!(dt!(2020-Nov-30), span!(Month<Precedent>4), dt!(2020-Jul-1), dt!(2020-Oct-31));
        check!(dt!(2020-Dec-1), span!(Month<Precedent>2), dt!(2020-Oct-1), dt!(2020-Nov-30));
        check!(dt!(2020-Mar-12), span!(Month<Anterior>24), dt!(2018-Mar-13), dt!(2020-Mar-12));
        check!(dt!(2020-Mar-1), span!(Month<Anterior>2), dt!(2020-Jan-2), dt!(2020-Mar-1));
        check!(dt!(2020-Feb-29), span!(Month<Anterior>1), dt!(2020-Jan-30), dt!(2020-Feb-29));
    }

    #[test]
    fn year_jumps() {
        check!(dt!(2020-Jan-15), span!(Year<Current>5), dt!(2020-Jan-1), dt!(2024-Dec-31));
        check!(dt!(2020-Feb-29), span!(Year<Posterior>2), dt!(2020-Feb-29), dt!(2022-Feb-28));
        check!(dt!(2020-Mar-1), span!(Year<Posterior>1), dt!(2020-Mar-1), dt!(2021-Feb-28));
        check!(dt!(2018-Oct-30), span!(Year<Successor>3), dt!(2019-Jan-1), dt!(2021-Dec-31));
        check!(dt!(2020-Dec-31), span!(Year<Precedent>10), dt!(2010-Jan-1), dt!(2019-Dec-31));
        check!(dt!(2020-Dec-31), span!(Year<Anterior>10), dt!(2011-Jan-1), dt!(2020-Dec-31));
    }
}
