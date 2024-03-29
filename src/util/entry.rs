//! Implementations directly related to entries and their fields

use std::fmt;
use std::str::FromStr;

use num_derive::FromPrimitive;

use crate::util::date::{Between, Date};

/// Contents of entries
pub mod fields {
    pub use super::{Amount, Category, Duration, Span, Tag, Window};
}

/// A quantity of money with cent precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub isize);

/// A label for an expense
#[derive(Debug, Clone)]
pub struct Tag(pub String);

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}.{:02}€",
            if self.0 >= 0 { "" } else { "-" },
            self.0.abs() / 100,
            (self.0 % 100).abs()
        )
    }
}

impl crate::util::period::Minimax for Amount {
    const MIN: Self = Self(isize::MIN);
    const MAX: Self = Self(isize::MAX);
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
    period: Between<Date>,
    /// cached length of the period for performance
    length: usize,
    tag: Option<Tag>,
}

/// Kinds of expenses
#[derive(Debug, Clone, Copy, FromPrimitive, Eq, PartialEq, Hash)]
pub enum Category {
    Salary,
    Home,
    School,
    Cleaning,
    Movement,
    Tech,
    Food,
    Fun,
}

/// Generic period generator when given a reference date
#[derive(Debug, Clone, Copy)]
pub struct Span {
    duration: Duration,
    window: Window,
    count: usize,
}

/// Granularity of `Span` length
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Duration {
    Day,
    Week,
    Month,
    Year,
}

impl Duration {
    pub fn text_frequency(self) -> &'static str {
        match self {
            Duration::Day => "Daily",
            Duration::Week => "Weekly",
            Duration::Month => "Monthly",
            Duration::Year => "Yearly",
        }
    }
}

/// Position of `Span` relative to reference date
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Window {
    Current,
    Posterior,
    Anterior,
    Precedent,
    Successor,
}

impl Category {
    pub const COUNT: usize = 8;

    pub fn sign(self) -> bool {
        use Category::*;
        match self {
            Salary => true,
            _ => false,
        }
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

impl std::iter::Sum for Amount {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut sum = Amount(0);
        for x in iter {
            sum += x;
        }
        sum
    }
}

impl Entry {
    /// Aggregate elements into a single entry
    pub fn from(value: Amount, cat: Category, period: Between<Date>, tag: Tag) -> Self {
        let length = period.1.index() - period.0.index() + 1;
        Self {
            value,
            cat,
            tag: Some(tag),
            period,
            length,
        }
    }

    /// Calculate intersection with a period, discard the label
    pub fn intersect_loss(&self, period: Between<Date>) -> Option<Self> {
        let start = period.0.max(self.period.0);
        let end = period.1.min(self.period.1);
        if start > end {
            return None;
        }
        let idx_old = self.period.0.index() as isize;
        let idx_new = (start.index(), end.index());
        let before_end = self.value.0 * (idx_new.1 as isize + 1 - idx_old) / self.length as isize;
        let before_start = self.value.0 * (idx_new.0 as isize - idx_old) / self.length as isize;
        #[cfg(test)]
        {
            println!(
                "Truncating {}..{} to {}..{}",
                self.period.0.index(),
                self.period.1.index(),
                idx_new.0,
                idx_new.1
            );
            println!("    {} -> {} - {}", self.value.0, before_end, before_start);
        }
        Some(Self {
            value: Amount(before_end - before_start),
            period: Between(start, end),
            length: idx_new.1 - idx_new.0 + 1,
            tag: None,
            cat: self.cat,
        })
    }

    /// Calculate intersection with a period, keep the label
    pub fn intersect(mut self, period: Between<Date>) -> Option<Self> {
        let tag = self.tag.take();
        self.intersect_loss(period).map(|mut entry| {
            entry.tag = tag;
            entry
        })
    }

    pub fn value(&self) -> Amount {
        self.value
    }

    pub fn category(&self) -> Category {
        self.cat
    }

    pub fn period(&self) -> Between<Date> {
        self.period
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = format!("{}", self.value);
        let padding = " ".repeat(10_usize.saturating_sub(value.len()));
        write!(
            f,
            "{}..{}: \t{}{}\t ({:?}",
            self.period.0, self.period.1, padding, value, self.cat
        )?;
        if let Some(t) = &self.tag {
            write!(f, "/{}", t.0)?;
        }
        write!(f, ")")
    }
}

impl Span {
    pub fn from(duration: Duration, window: Window, count: usize) -> Self {
        Self {
            duration,
            window,
            count,
        }
    }

    /// Use reference date to create a range of dates
    pub fn period(&self, dt: Date) -> Between<Date> {
        use Duration::*;
        use Window::*;
        let nb = self.count as isize;
        let (start, end) = match (self.duration, self.window) {
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
        };
        Between(start, end)
    }
}

impl FromStr for Category {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        use Category::*;
        Ok(match s {
            "Pay" => Salary,
            "Food" => Food,
            "Tech" => Tech,
            "Mov" => Movement,
            "Pro" => School,
            "Clean" => Cleaning,
            "Home" => Home,
            "Fun" => Fun,
            _ => return Err(()),
        })
    }
}

impl FromStr for Duration {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        use Duration::*;
        Ok(match s {
            "Day" => Day,
            "Week" => Week,
            "Month" => Month,
            "Year" => Year,
            _ => return Err(()),
        })
    }
}

impl FromStr for Window {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        use Window::*;
        Ok(match s {
            "Curr" => Current,
            "Post" => Posterior,
            "Ante" => Anterior,
            "Pred" => Precedent,
            "Succ" => Successor,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use num_traits::FromPrimitive;
    use super::{Duration::*, Window::*, *};
    use crate::util::date::{Date, Month::*};

    #[test]
    fn count_categories() {
        for i in 0..Category::COUNT {
            assert!(Category::from_usize(i).is_some());
        }
        assert!(Category::from_usize(Category::COUNT).is_none());
    }

    macro_rules! dt {
        ( $y:tt - $m:tt - $d:tt ) => {
            Date::from($y, $m, $d).unwrap()
        };
    }
    macro_rules! span {
        ( $dur:tt < $win:tt > $nb:tt ) => {
            Span {
                duration: $dur,
                window: $win,
                count: $nb,
            }
        };
    }

    macro_rules! check {
        ( $date:expr, $span:expr, $start:expr, $end:expr ) => {
            assert_eq!($span.period($date), Between($start, $end));
        };
    }

    #[test]
    fn day_jumps() {
        check!(
            dt!(2020-May-8),
            span!(Day<Current>3),
            dt!(2020-May-8),
            dt!(2020-May-10)
        );
        check!(
            dt!(2020-Sep-1),
            span!(Day<Precedent>5),
            dt!(2020-Aug-27),
            dt!(2020-Aug-31)
        );
        check!(
            dt!(2020-Dec-30),
            span!(Day<Anterior>2),
            dt!(2020-Dec-29),
            dt!(2020-Dec-30)
        );
        check!(
            dt!(2020-Jan-1),
            span!(Day<Posterior>50),
            dt!(2020-Jan-1),
            dt!(2020-Feb-19)
        );
        check!(
            dt!(2020-Feb-28),
            span!(Day<Successor>3),
            dt!(2020-Feb-29),
            dt!(2020-Mar-2)
        );
    }

    #[test]
    fn week_jumps() {
        check!(
            dt!(2020-Mar-12),
            span!(Week<Current>2),
            dt!(2020-Mar-9),
            dt!(2020-Mar-22)
        );
        check!(
            dt!(2020-Sep-8),
            span!(Week<Precedent>3),
            dt!(2020-Aug-17),
            dt!(2020-Sep-6)
        );
        check!(
            dt!(2020-Aug-9),
            span!(Week<Successor>6),
            dt!(2020-Aug-10),
            dt!(2020-Sep-20)
        );
        check!(
            dt!(2020-May-23),
            span!(Week<Anterior>4),
            dt!(2020-Apr-26),
            dt!(2020-May-23)
        );
        check!(
            dt!(2020-Dec-30),
            span!(Week<Posterior>1),
            dt!(2020-Dec-30),
            dt!(2021-Jan-5)
        );
    }

    #[test]
    fn month_jumps() {
        check!(
            dt!(2020-May-31),
            span!(Month<Current>5),
            dt!(2020-May-1),
            dt!(2020-Sep-30)
        );
        check!(
            dt!(2020-Feb-29),
            span!(Month<Current>2),
            dt!(2020-Feb-1),
            dt!(2020-Mar-31)
        );
        check!(
            dt!(2020-Feb-29),
            span!(Month<Posterior>12),
            dt!(2020-Feb-29),
            dt!(2021-Feb-28)
        );
        check!(
            dt!(2020-Feb-28),
            span!(Month<Posterior>1),
            dt!(2020-Feb-28),
            dt!(2020-Mar-27)
        );
        check!(
            dt!(2020-Aug-15),
            span!(Month<Successor>3),
            dt!(2020-Sep-1),
            dt!(2020-Nov-30)
        );
        check!(
            dt!(2020-Jan-31),
            span!(Month<Successor>4),
            dt!(2020-Feb-1),
            dt!(2020-May-31)
        );
        check!(
            dt!(2020-Nov-30),
            span!(Month<Precedent>4),
            dt!(2020-Jul-1),
            dt!(2020-Oct-31)
        );
        check!(
            dt!(2020-Dec-1),
            span!(Month<Precedent>2),
            dt!(2020-Oct-1),
            dt!(2020-Nov-30)
        );
        check!(
            dt!(2020-Mar-12),
            span!(Month<Anterior>24),
            dt!(2018-Mar-13),
            dt!(2020-Mar-12)
        );
        check!(
            dt!(2020-Mar-1),
            span!(Month<Anterior>2),
            dt!(2020-Jan-2),
            dt!(2020-Mar-1)
        );
        check!(
            dt!(2020-Feb-29),
            span!(Month<Anterior>1),
            dt!(2020-Jan-30),
            dt!(2020-Feb-29)
        );
    }

    #[test]
    fn year_jumps() {
        check!(
            dt!(2020-Jan-15),
            span!(Year<Current>5),
            dt!(2020-Jan-1),
            dt!(2024-Dec-31)
        );
        check!(
            dt!(2020-Feb-29),
            span!(Year<Posterior>2),
            dt!(2020-Feb-29),
            dt!(2022-Feb-28)
        );
        check!(
            dt!(2020-Mar-1),
            span!(Year<Posterior>1),
            dt!(2020-Mar-1),
            dt!(2021-Feb-28)
        );
        check!(
            dt!(2018-Oct-30),
            span!(Year<Successor>3),
            dt!(2019-Jan-1),
            dt!(2021-Dec-31)
        );
        check!(
            dt!(2020-Dec-31),
            span!(Year<Precedent>10),
            dt!(2010-Jan-1),
            dt!(2019-Dec-31)
        );
        check!(
            dt!(2020-Dec-31),
            span!(Year<Anterior>10),
            dt!(2011-Jan-1),
            dt!(2020-Dec-31)
        );
    }

    macro_rules! bogus {
        ( $val:expr, $start:expr, $end:expr ) => {{
            let value = Amount($val);
            let start = $start;
            let end = $end;
            Entry {
                value,
                cat: Category::Food,
                tag: None,
                period: Between(start, end),
                length: end.index() - start.index() + 1,
            }
        }};
    }

    #[test]
    fn intersections() {
        assert_eq!(
            bogus!(365, dt!(2021-Jan-1), dt!(2021-Dec-31))
                .intersect(Between(dt!(2021-Feb-1), dt!(2021-Feb-15)))
                .unwrap()
                .value,
            Amount(15)
        );
        assert_eq!(
            bogus!(365, dt!(2021-Jan-1), dt!(2021-Dec-31))
                .intersect(Between(dt!(2020-Sep-15), dt!(2021-Jan-15)))
                .unwrap()
                .value,
            Amount(15)
        );
        {
            let entry = bogus!(1763, dt!(2020-Mar-13), dt!(2020-Sep-27));
            let sections = vec![
                dt!(2020-Mar-13),
                dt!(2020-Apr-5),
                dt!(2020-Jun-30),
                dt!(2020-Sep-28),
            ];
            let splits = sections
                .windows(2)
                .map(|w| entry.clone().intersect(Between(w[0], w[1].prev())).unwrap());
            assert_eq!(entry.value, splits.map(|e| e.value).sum())
        }
    }
}
