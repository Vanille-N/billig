//! An inclusive range of dates

use std::fmt;
use std::str::FromStr;

use crate::lib::date::{Date, Month};

/// `Period(a, b)` is the range of dates from `a` to `b` inclusive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period(pub Date, pub Date);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFrame {
    Between(Date, Date),
    After(Date),
    Before(Date),
    Empty,
    Unbounded,
}

impl TimeFrame {
    pub fn as_period(self) -> Period {
        use TimeFrame::*;
        let (start, end) = match self {
            Between(start, end) => (start, end),
            After(start) => (start, Date::MAX),
            Before(end) => (Date::MIN, end),
            Empty => (Date::MAX, Date::MIN),
            Unbounded => (Date::MIN, Date::MAX),
        };
        Period(start, end)
    }
}

impl Period {
    pub fn as_timeframe(self) -> TimeFrame {
        use TimeFrame::*;
        if self.0 > self.1 {
            Empty
        } else if self.0 == Date::MIN {
            if self.1 == Date::MAX {
                Unbounded
            } else {
                Before(self.1)
            }
        } else if self.1 == Date::MAX {
            After(self.0)
        } else {
            Between(self.0, self.1)
        }
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let merge_day = |f: &mut fmt::Formatter| {
            if self.0.day() == 1 && self.1.day() == self.1.month().count(self.1.year()) {
                Ok(())
            } else if self.0.day() == self.1.day() {
                write!(f, "-{}", self.0.day())
            } else {
                write!(f, "-{}..{}", self.0.day(), self.1.day())
            }
        };
        let shorten_month = |f: &mut fmt::Formatter| {
            write!(f, "-{}", self.0.month())?;
            if self.0.day() != 1 {
                write!(f, "-{}", self.0.day())?;
            }
            write!(f, "..{}", self.1.month())?;
            if self.1.day() != self.1.month().count(self.1.year()) {
                write!(f, "-{}", self.1.day())?;
            }
            Ok(())
        };
        let shorten_year = |f: &mut fmt::Formatter| {
            if self.0 != Date::MIN {
                write!(f, "{}", self.0.year())?;
                if self.0.month() != Month::Jan || self.0.day() != 1 {
                    write!(f, "-{}", self.0.month())?;
                    if self.0.day() != 1 {
                        write!(f, "-{}", self.0.day())?;
                    }
                }
            }
            write!(f, "..")?;
            if self.1 != Date::MAX {
                write!(f, "{}", self.1.year())?;
                if self.1.month() != Month::Dec || self.1.day() != 31 {
                    write!(f, "-{}", self.1.month())?;
                    if self.1.day() != self.1.month().count(self.1.year()) {
                        write!(f, "-{}", self.1.day())?;
                    }
                }
            }
            Ok(())
        };
        let merge_month = |f: &mut fmt::Formatter| {
            if self.0.month() == Month::Jan
                && self.0.day() == 1
                && self.1.month() == Month::Dec
                && self.1.day() == 31
            {
                Ok(())
            } else if self.0.month() == self.1.month() {
                write!(f, "-{}", self.0.month())?;
                merge_day(f)
            } else {
                shorten_month(f)
            }
        };
        let merge_year = |f: &mut fmt::Formatter| {
            if self.0.year() == self.1.year() {
                write!(f, "{}", self.0.year())?;
                merge_month(f)
            } else {
                shorten_year(f)
            }
        };
        if self.0 <= self.1 {
            merge_year(f)
        } else {
            write!(f, "()")
        }
    }
}

use crate::load::error::{self, Loc};
use pest::Parser;

use crate::load::parse::Rule;
type Pair<'i> = pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;

impl TimeFrame {
    pub fn normalized(self) -> Self {
        if let TimeFrame::Between(start, end) = self {
            if start > end {
                return TimeFrame::Empty;
            }
        }
        self
    }

    pub fn intersect(self, other: Self) -> Self {
        use TimeFrame::*;
        match (self, other) {
            (_, Empty) | (Empty, _) => Empty,
            (lhs, Unbounded) => lhs,
            (Unbounded, rhs) => rhs,
            (Between(start1, end1), Between(start2, end2)) => Between(start1.max(start2), end1.min(end2)),
            (After(start1), Between(start2, end2)) => Between(start1.max(start2), end2),
            (Before(end1), Between(start2, end2)) => Between(start2, end1.min(end2)),
            (Between(start1, end1), After(start2)) => Between(start1.max(start2), end1),
            (Between(start1, end1), Before(end2)) => Between(start1, end1.min(end2)),
            (After(start1), After(start2)) => After(start1.max(start2)),
            (After(start1), Before(end2)) => Between(start1, end2),
            (Before(end1), After(start2)) => Between(start2, end1),
            (Before(end1), Before(end2)) => Before(end1.min(end2)),
        }.normalized()
    }

    pub fn unite(self, other: Self) -> Self {
        use TimeFrame::*;
        match (self, other) {
            (_, Unbounded) | (Unbounded, _) => Unbounded,
            (lhs, Empty) => lhs,
            (Empty, rhs) => rhs,
            (Between(start1, end1), Between(start2, end2)) => Between(start1.min(start2), end1.max(end2)),
            (After(start1), Between(start2, end2)) => Between(start1.min(start2), end2),
            (Before(end1), Between(start2, end2)) => Between(start2, end1.max(end2)),
            (Between(start1, end1), After(start2)) => Between(start1.min(start2), end1),
            (Between(start1, end1), Before(end2)) => Between(start1, end1.max(end2)),
            (After(start1), After(start2)) => After(start1.min(start2)),
            (After(start1), Before(end2)) => Between(start1, end2),
            (Before(end1), After(start2)) => Between(start2, end1),
            (Before(end1), Before(end2)) => Before(end1.max(end2)),
        }.normalized()
    }
}

impl TimeFrame {
    pub fn parse(errs: &mut error::Record, s: &str) -> Option<TimeFrame> {
        let contents = match crate::load::parse::BilligParser::parse(Rule::period_only, s) {
            Ok(contents) => contents,
            Err(e) => {
                errs.make("Parsing failure")
                    .from(e);
                return None;
            }
        };
        let res = validate_timeframe(errs, contents);
        if errs.is_fatal() {
            None
        } else {
            res
        }
    }
}

fn validate_timeframe(errs: &mut error::Record, p: Pairs) -> Option<TimeFrame> {
    let inner = p.into_iter().next().unwrap();
    let loc = ("", inner.as_span().clone());
    match inner.as_rule() {
        Rule::period_after => {
            let trunc = validate_full_date(errs, inner.into_inner().into_iter().next().unwrap())?;
            Some(TimeFrame::After(trunc.make(errs, &loc, true)?))
        }
        Rule::period_before => {
            let end = inner.into_inner().into_iter().next();
            match end {
                Some(end) => {
                    let trunc = validate_full_date(errs, end)?;
                    Some(TimeFrame::Before(trunc.make(errs, &loc, false)?))
                }
                None => Some(TimeFrame::Unbounded),
            }
        }
        Rule::full_date => {
            let trunc = validate_full_date(errs, inner)?;
            Some(TimeFrame::Between(trunc.make(errs, &loc, true)?, trunc.make(errs, &loc, false)?))
        }
        Rule::period_between => {
            let mut inner = inner.into_inner();
            let fst = inner.next().unwrap();
            let loc = ("", fst.as_span().clone());
            let start = validate_full_date(errs, fst)?.make(errs, &loc, true)?;
            let snd = inner.next().unwrap();
            let loc = ("", snd.as_span().clone());
            let end = validate_partial_date(errs, start, snd)?.make(errs, &loc, false)?;
            if start <= end {
                Some(TimeFrame::Between(start, end))
            } else {
                errs.make("End before start of timeframe")
                    .span(&loc, "this timeframe")
                    .text("Timeframe is empty")
                    .hint("If this is intentionnal consider using '()' instead");
                Some(TimeFrame::Empty)
            }
        }
        Rule::period_empty => {
            Some(TimeFrame::Empty)
        }
        _ => unreachable!(),
    }
}

fn validate_full_date(errs: &mut error::Record, p: Pair) -> Option<TruncDate> {
    let mut inner = p.into_inner();
    let year = inner.next().unwrap().as_str().parse::<u16>().unwrap();
    match inner.next() {
        None => Some(TruncDate {
            year,
            ..Default::default()
        }),
        Some(month) => validate_month_date(errs, year, month),
    }
}

fn validate_partial_date(errs: &mut error::Record, default: Date, p: Pair) -> Option<TruncDate> {
    match p.as_rule() {
        Rule::full_date => validate_full_date(errs, p),
        Rule::month_date => validate_month_date(errs, default.year(), p),
        Rule::marker_day => validate_day_date(errs, default.year(), default.month(), p),
        _ => unreachable!("{:?}", p),
    }
}

fn validate_month_date(errs: &mut error::Record, year: u16, p: Pair) -> Option<TruncDate> {
    let mut inner = p.into_inner();
    let month = inner.next().unwrap();
    let loc = ("", month.as_span().clone());
    let month = match month.as_str().parse::<Month>() {
        Ok(month) => month,
        Err(()) => {
            errs.make("Invalid Month")
                .span(&loc, "provided here")
                .text(format!("'{}' is not a valid month", month.as_str()))
                .hint("Months are 'Jan', 'Feb', ..., 'Dec'");
            return None;
        }
    };
    match inner.next() {
        None => Some(TruncDate {
            year,
            month: Some(month),
            ..Default::default()
        }),
        Some(day) => validate_day_date(errs, year, month, day),
    }
}

fn validate_day_date(errs: &mut error::Record, year: u16, month: Month, p: Pair) -> Option<TruncDate> {
    let day = p.as_str().parse::<u8>().unwrap();
    Some(TruncDate {
        year,
        month: Some(month),
        day: Some(day),
    })
}

#[derive(Default, Debug)]
struct TruncDate {
    year: u16,
    month: Option<Month>,
    day: Option<u8>,
}

impl TruncDate {
    fn make(&self, errs: &mut error::Record, loc: &Loc, starting: bool) -> Option<Date> {
        let year = self.year;
        let month = self
            .month
            .unwrap_or(if starting { Month::Jan } else { Month::Dec });
        let day = self
            .day
            .unwrap_or(if starting { 1 } else { month.count(year) });
        match Date::from(year as usize, month, day as usize) {
            Ok(date) => Some(date),
            Err(e) => {
                errs.make("Invalid date")
                    .span(loc, "provided here")
                    .text(format!("{}", e))
                    .hint("choose a date that exists")
                    .hint(e.fix_hint());
                None
            }
        }
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use crate::lib::date::{Month::*, *};

    macro_rules! dt {
        ( $y:tt - $m:tt - $d:tt ) => {{
            Date::from($y, $m, $d).unwrap()
        }}
    }

    macro_rules! pp {
        ( $start:expr, $end:expr => $fmt:expr ) => {{
            assert_eq!(&format!("{}", Period($start, $end)), $fmt);
        }}
    }

    #[test]
    fn period_fmt() {
        pp!(dt!(2020-Jan-15), dt!(2021-Mar-17) => "2020-Jan-15..2021-Mar-17");
        pp!(dt!(2020-Jan-15), dt!(2020-Mar-17) => "2020-Jan-15..Mar-17");
        pp!(dt!(2020-Jan-15), dt!(2020-Jan-17) => "2020-Jan-15..17");
        pp!(dt!(2020-Jan-15), dt!(2020-Jan-15) => "2020-Jan-15");
        pp!(dt!(2020-Jan-1), dt!(2020-Jan-31) => "2020-Jan");
        pp!(dt!(2020-Jan-1), dt!(2020-Jan-15) => "2020-Jan-1..15");
        pp!(dt!(2020-Jan-15), dt!(2020-Jan-31) => "2020-Jan-15..31");
        pp!(dt!(2020-Jan-1), dt!(2020-Feb-15) => "2020-Jan..Feb-15");
        pp!(dt!(2020-Jan-1), dt!(2020-Feb-29) => "2020-Jan..Feb");
        pp!(dt!(2020-Jan-1), dt!(2021-Mar-17) => "2020..2021-Mar-17");
        pp!(dt!(2020-Feb-3), dt!(2021-Dec-31) => "2020-Feb-3..2021");
        pp!(dt!(2020-Jan-1), dt!(2021-Mar-31) => "2020..2021-Mar");
        pp!(dt!(2020-Jan-1), dt!(2020-Dec-31) => "2020");
        pp!(dt!(2020-Jan-1), dt!(2023-Dec-31) => "2020..2023");
        pp!(dt!(2020-Jan-3), dt!(2023-Feb-28) => "2020-Jan-3..2023-Feb");
    }

    macro_rules! ps {
        ( $s:expr => $res:expr ) => {{
            match Period::parse(&mut error::Record::new(), $s) {
                Some(period) => assert_eq!(&format!("{}", period), $res),
                None => println!("{} ->\n{}", $s, err),
            }
        }};
        ( $s:expr ) => {{
            ps!($s => $s)
        }};
    }

    #[test]
    fn period_parse() {
        ps!("2020-Jan-15..2021-Mar-17");
        ps!("2020-Jan-15..Mar-17");
        ps!("2020-Jan-15..17");
        ps!("2020-Jan-15");
        ps!("2020-Jan");
        ps!("2020-Jan-1..15");
        ps!("2020-Jan-15..31");
        ps!("2020-Jan..Feb-15");
        ps!("2020-Jan..Feb");
        ps!("2020..2021-Mar-17");
        ps!("2020-Feb-3..2021");
        ps!("2020..2021-Mar");
        ps!("2020");
        ps!("2020..2023");
        ps!("2020-Jan-3..2023-Feb");
        ps!("2020-Jan-10..");
        ps!("..2020");
        ps!("2020..Mar" => "2020-Jan..Mar");
        ps!("2020-Jan..15" => "2020-Jan-1..15");
        ps!("2020-Jan-15..2020" => "2020-Jan-15..Dec");
        ps!("2020..2020" => "2020");
        ps!("..");
    }
}
