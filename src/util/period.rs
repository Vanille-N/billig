//! An inclusive range of dates

use std::fmt;

use crate::util::date::{Date, Month};

pub trait Minimax: Ord {
    const MIN: Self;
    const MAX: Self;
}

impl Minimax for i64 {
    const MIN: i64 = Self::MIN;
    const MAX: i64 = Self::MAX;
}

/// `Period(a, b)` is the range of dates from `a` to `b` inclusive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Between<T>(pub T, pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval<T> {
    Between(T, T),
    After(T),
    Before(T),
    Empty,
    Unbounded,
}

impl<T> Interval<T>
where
    T: Minimax,
{
    pub fn into_between(self) -> Between<T> {
        let (start, end) = match self {
            Interval::Between(start, end) => (start, end),
            Interval::After(start) => (start, T::MAX),
            Interval::Before(end) => (T::MAX, end),
            Interval::Empty => (T::MAX, T::MIN),
            Interval::Unbounded => (T::MIN, T::MAX),
        };
        Between(start, end)
    }
}

impl Interval<Date> {
    pub fn bounded(self, errs: &mut error::Record, loc: &Loc, date: Date) -> Option<Between<Date>> {
        let (start, end) = match self {
            Interval::Empty => {
                errs.make("Period cannot be empty")
                    .span(loc, "provided here")
                    .text("Explicit periods must have a beginning and/or an end")
                    .hint("use START.. or ..END or START..END")
                    .hint("for a single day simply use `span Day`");
                return None;
            }
            Interval::Unbounded => {
                errs.make("Period cannot be unbounded")
                    .span(loc, "provided here")
                    .text("Explicit periods must have a beginning and/or an end")
                    .hint("use START.. or ..END or START..END")
                    .hint("for a single day simply use `span Day`");
                return None;
            }
            Interval::Between(start, end) => (start, end),
            Interval::After(start) => (start, date),
            Interval::Before(end) => (date, end),
        };
        if start > end {
            errs.make("Period is accidentally empty")
                .span(loc, "provided here")
                .text("This period has its END smaller than START")
                .hint("empty periods are forbidden here");
            return None;
        }
        Some(Between(start, end))
    }
}

impl<T> Between<T>
where
    T: Minimax,
{
    pub fn into_interval(self) -> Interval<T> {
        if self.0 > self.1 {
            Interval::Empty
        } else if self.0 == T::MIN {
            if self.1 == T::MAX {
                Interval::Unbounded
            } else {
                Interval::Before(self.1)
            }
        } else if self.1 == T::MAX {
            Interval::After(self.0)
        } else {
            Interval::Between(self.0, self.1)
        }
    }
}

impl<T> Between<T>
where
    T: Ord,
{
    pub fn unite(self, other: Self) -> Self {
        Self(self.0.min(other.0), self.1.max(other.1))
    }

    pub fn intersect(self, other: Self) -> Self {
        Self(self.0.max(other.0), self.1.min(other.1))
    }
}

impl fmt::Display for Between<Date> {
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

impl<T> Interval<T>
where
    T: Ord,
{
    pub fn normalized(self) -> Self {
        if let Interval::Between(start, end) = &self {
            if start > end {
                return Interval::Empty;
            }
        }
        self
    }

    pub fn intersect(self, other: Self) -> Self {
        use Interval::*;
        match (self, other) {
            (_, Empty) | (Empty, _) => Empty,
            (lhs, Unbounded) => lhs,
            (Unbounded, rhs) => rhs,
            (Between(start1, end1), Between(start2, end2)) => {
                Between(start1.max(start2), end1.min(end2))
            }
            (After(start1), Between(start2, end2)) => Between(start1.max(start2), end2),
            (Before(end1), Between(start2, end2)) => Between(start2, end1.min(end2)),
            (Between(start1, end1), After(start2)) => Between(start1.max(start2), end1),
            (Between(start1, end1), Before(end2)) => Between(start1, end1.min(end2)),
            (After(start1), After(start2)) => After(start1.max(start2)),
            (After(start1), Before(end2)) => Between(start1, end2),
            (Before(end1), After(start2)) => Between(start2, end1),
            (Before(end1), Before(end2)) => Before(end1.min(end2)),
        }
        .normalized()
    }

    pub fn unite(self, other: Self) -> Self {
        use Interval::*;
        match (self, other) {
            (_, Unbounded) | (Unbounded, _) => Unbounded,
            (lhs, Empty) => lhs,
            (Empty, rhs) => rhs,
            (Between(start1, end1), Between(start2, end2)) => {
                Between(start1.min(start2), end1.max(end2))
            }
            (After(start1), Between(start2, end2)) => Between(start1.min(start2), end2),
            (Before(end1), Between(start2, end2)) => Between(start2, end1.max(end2)),
            (Between(start1, end1), After(start2)) => Between(start1.min(start2), end1),
            (Between(start1, end1), Before(end2)) => Between(start1, end1.max(end2)),
            (After(start1), After(start2)) => After(start1.min(start2)),
            (After(start1), Before(end2)) => Between(start1, end2),
            (Before(end1), After(start2)) => Between(start2, end1),
            (Before(end1), Before(end2)) => Before(end1.max(end2)),
        }
        .normalized()
    }
}

impl Interval<PartialDate> {
    pub fn parse(path: &str, errs: &mut error::Record, s: &str) -> Option<Interval<PartialDate>> {
        let contents = match crate::load::parse::BilligParser::parse(Rule::period_only, s) {
            Ok(contents) => contents,
            Err(e) => {
                errs.make("Parsing failure").from(e);
                return None;
            }
        };
        let res = validate_partial_period(path, errs, contents);
        if errs.is_fatal() {
            None
        } else {
            res
        }
    }

    pub fn make(
        self,
        errs: &mut error::Record,
        loc: &Loc,
        reference: Date,
    ) -> Option<Interval<Date>> {
        match self {
            Interval::Empty => Some(Interval::Empty),
            Interval::Unbounded => Some(Interval::Unbounded),
            Interval::After(pdt) => Some(Interval::After(
                pdt.default_year(reference.year())
                    .default_month(if pdt.day.is_none() {
                        Month::Jan
                    } else {
                        reference.month()
                    })
                    .make(errs, loc, true)?,
            )),
            Interval::Before(pdt) => Some(Interval::Before(
                pdt.default_year(reference.year())
                    .default_month(if pdt.day.is_none() {
                        Month::Dec
                    } else {
                        reference.month()
                    })
                    .make(errs, loc, false)?,
            )),
            Interval::Between(start, end) => {
                let dstart = start
                    .default_year(reference.year())
                    .default_month(if start.day.is_none() {
                        Month::Jan
                    } else {
                        reference.month()
                    })
                    .make(errs, loc, true)?;
                let dend = if end.year.is_none() {
                    end.default_year(dstart.year())
                        .default_month(start.month.unwrap_or(if end.day.is_none() {
                            Month::Dec
                        } else {
                            reference.month()
                        }))
                } else {
                    end
                };
                let dend = dend.make(errs, loc, false)?;
                if dstart > dend {
                    errs.make("End before start of timeframe")
                        .span(loc, "this timeframe")
                        .text("Timeframe is empty")
                        .hint("If this is intentionnal consider using '()' instead");
                    None
                } else {
                    Some(Interval::Between(dstart, dend))
                }
            }
        }
    }
}

pub fn validate_partial_period(
    path: &str,
    errs: &mut error::Record,
    p: Pairs,
) -> Option<Interval<PartialDate>> {
    let inner = p.into_iter().next().unwrap();
    match inner.as_rule() {
        Rule::period_after => {
            let trunc =
                validate_partial_date(path, errs, inner.into_inner().into_iter().next().unwrap())?;
            Some(Interval::After(trunc))
        }
        Rule::period_before => {
            let end = inner.into_inner().into_iter().next();
            match end {
                Some(end) => {
                    let trunc = validate_partial_date(path, errs, end)?;
                    Some(Interval::Before(trunc))
                }
                None => Some(Interval::Unbounded),
            }
        }
        Rule::partial_date | Rule::full_date | Rule::marker_day | Rule::month_date => {
            let trunc = validate_partial_date(path, errs, inner)?;
            Some(Interval::Between(trunc, trunc))
        }
        Rule::period_between => {
            let mut inner = inner.into_inner();
            let fst = inner.next().unwrap();
            let start = validate_partial_date(path, errs, fst)?;
            let snd = inner.next().unwrap();
            let end = validate_partial_date(path, errs, snd)?;
            Some(Interval::Between(start, end))
        }
        Rule::period_empty => Some(Interval::Empty),
        Rule::period => validate_partial_period(path, errs, inner.into_inner()),
        _ => unreachable!("{:?}", inner),
    }
}

fn validate_full_date(path: &str, errs: &mut error::Record, p: Pair) -> Option<PartialDate> {
    let mut inner = p.into_inner();
    let year = inner.next().unwrap().as_str().parse::<u16>().unwrap();
    match inner.next() {
        None => Some(PartialDate {
            year: Some(year),
            ..Default::default()
        }),
        Some(month) => validate_month_date(path, errs, Some(year), month),
    }
}

fn validate_partial_date(path: &str, errs: &mut error::Record, p: Pair) -> Option<PartialDate> {
    match p.as_rule() {
        Rule::full_date => validate_full_date(path, errs, p),
        Rule::month_date => validate_month_date(path, errs, None, p),
        Rule::marker_day => validate_day_date(None, None, p),
        _ => unreachable!("{:?}", p),
    }
}

fn validate_month_date(
    path: &str,
    errs: &mut error::Record,
    year: Option<u16>,
    p: Pair,
) -> Option<PartialDate> {
    let mut inner = p.into_inner();
    let month = inner.next().unwrap();
    let loc = (path, month.as_span().clone());
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
        None => Some(PartialDate {
            year,
            month: Some(month),
            ..Default::default()
        }),
        Some(day) => validate_day_date(year, Some(month), day),
    }
}

fn validate_day_date(year: Option<u16>, month: Option<Month>, p: Pair) -> Option<PartialDate> {
    let day = p.as_str().parse::<u8>().unwrap();
    Some(PartialDate {
        year,
        month,
        day: Some(day),
    })
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartialDate {
    year: Option<u16>,
    month: Option<Month>,
    day: Option<u8>,
}

impl PartialDate {
    fn default_year(mut self, year: u16) -> Self {
        if self.year.is_none() {
            self.year = Some(year);
        }
        self
    }

    fn default_month(mut self, month: Month) -> Self {
        if self.month.is_none() {
            self.month = Some(month);
        }
        self
    }

    fn make(&self, errs: &mut error::Record, loc: &Loc, starting: bool) -> Option<Date> {
        let year = match self.year {
            None => {
                errs.make("Unspecified year")
                    .span(loc, "provided here")
                    .text("Impossible to guess year")
                    .hint("add YYYY- in front to indicate year of interest");
                return None;
            }
            Some(year) => year,
        };
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
    use super::*;

    macro_rules! dt {
        ( $y:tt - $m:tt - $d:tt ) => {{
            Date::from($y, $m, $d).unwrap()
        }}
    }

    macro_rules! pp {
        ( $start:expr, $end:expr => $fmt:expr ) => {{
            assert_eq!(&format!("{}", Between($start, $end)), $fmt);
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
        ( $s:tt, $b:tt, $res:tt ) => {{
            let mut err = crate::load::error::Record::new();
            match Interval::parse("raw", &mut err, $s).map(|pp| pp.make(&mut err, &("", pest::Span::new("", 0, 0).unwrap()), dt!(2021-Feb-1))).flatten() {
                Some(period) => {
                    if !$b { panic!("{} instead of a failure\nHelp: this should be rejected", period.into_period()); }
                    assert_eq!(&format!("{}", period.into_period()), $res);
                }
                None => {
                    if $b { panic!("{} instead of a success\nHelp: this should be accepted", err); }
                    let fmt = format!("{:?}", err);
                    if !fmt.contains($res) {
                        panic!("{} should contain {}\nHelp: this should be rejected for a different reason", err, $res);
                    }
                }
            }
        }};
        ( idem $s:tt ) => {{
            ps!($s, true, $s)
        }};
        ( $s:tt ok $res:tt ) => {{
            ps!($s, true, $res)
        }};
        ( $s:tt fail $res:tt ) => {{
            ps!($s, false, $res)
        }};
    }

    #[test]
    fn period_parse() {
        ps!(idem "2020-Jan-15..2021-Mar-17");
        ps!(idem "2020-Jan-15..Mar-17");
        ps!(idem "2020-Jan-15..17");
        ps!(idem "2020-Jan-15");
        ps!(idem "2020-Jan");
        ps!(idem "2020-Jan-1..15");
        ps!(idem "2020-Jan-15..31");
        ps!(idem "2020-Jan..Feb-15");
        ps!(idem "2020-Jan..Feb");
        ps!(idem "2020..2021-Mar-17");
        ps!(idem "2020-Feb-3..2021");
        ps!(idem "2020..2021-Mar");
        ps!(idem "2020");
        ps!(idem "2020..2023");
        ps!(idem "2020-Jan-3..2023-Feb");
        ps!(idem "2020-Jan-10..");
        ps!(idem "..2020");
        ps!(idem "2020..");
        ps!("2020..Mar" ok "2020-Jan..Mar");
        ps!("2020-Jan..15" ok "2020-Jan-1..15");
        ps!("2020-Jan-15..2020" ok "2020-Jan-15..Dec");
        ps!("2020..2020" ok "2020");
        ps!(idem "..");
        ps!("..Feb-15" ok "..2021-Feb-15");
        ps!("..1" ok "..2021-Feb-1");
        ps!("Mar.." ok "2021-Mar..");
        ps!("..Mar" ok "..2021-Mar");
        ps!("15.." ok "2021-Feb-15..");
        ps!("..15" ok "..2021-Feb-15");
        ps!("17..21" ok "2021-Feb-17..21");
        ps!("17..Mar-1" ok "2021-Feb-17..Mar-1");
        ps!("Mar-13..17" ok "2021-Mar-13..17");
        ps!("Mar-13..2021" ok "2021-Mar-13..Dec");
        ps!("Mar-13..Oct" ok "2021-Mar-13..Oct");
        ps!("15" ok "2021-Feb-15");
        ps!("Jan" ok "2021-Jan");
        ps!("Jan-15" ok "2021-Jan-15");
        ps!(idem "()");
        ps!("..0" fail "not in the range");
        ps!("..45" fail "not in the range");
        ps!("Bef.." fail "not a valid month");
        ps!("1..3..5" fail "expected EOF");
        ps!("2020202" fail "expected EOF");
        ps!("0000" fail "outside of the supported range");
        ps!("Jan-20..." fail "expected EOF");
        ps!("20..15" fail "Timeframe is empty");
    }
}
