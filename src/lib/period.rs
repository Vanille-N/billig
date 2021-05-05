use std::fmt;
use std::str::FromStr;

use crate::lib::{
    date::{Date, DateError, Month},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period(pub Date, pub Date);

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
            if self.0.month() == Month::Jan && self.0.day() == 1 && self.1.month() == Month::Dec && self.1.day() == 31 {
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
        merge_year(f)
    }
}

use pest::Parser;
use pest_derive::*;
use crate::load::error::Error;
use crate::load::error::Loc;

type Pair<'i> = pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = pest::iterators::Pairs<'i, Rule>;
type Result<T> = std::result::Result<T, Error<Rule>>;

#[derive(Parser)]
#[grammar = "lib/grammar-period.pest"]
struct PeriodParser;

impl Period {
    pub fn parse(s: &str) -> 

impl FromStr for Period {
    type Err = Error<Rule>;

    fn from_str(s: &str) -> Result<Period> {
        let contents = match PeriodParser::parse(Rule::period, s) {
            Ok(contents) => contents,
            Err(e) => {
                let mut err = Error::new("Parsing failure");
                err.from(e);
                return Err(err);
            }
        };
        validate_period(contents)
    }
}

fn validate_period(p: Pairs) -> Result<Period> {
    let inner = p.into_iter().next().unwrap();
    let loc = ("", inner.as_span().clone());
    match inner.as_rule() {
        Rule::after => {
            let trunc = validate_full_date(inner.into_inner().into_iter().next().unwrap())?;
            Ok(Period(trunc.make(&loc, true)?, Date::MAX))
        }
        Rule::before => {
            let end = inner.into_inner().into_iter().next();
            match end {
                Some(end) => {
                    let trunc = validate_full_date(end)?;
                    Ok(Period(Date::MIN, trunc.make(&loc, false)?))
                }
                None => Ok(Period(Date::MIN, Date::MAX)),
            }
        }
        Rule::full_date => {
            let trunc = validate_full_date(inner)?;
            Ok(Period(trunc.make(&loc, true)?, trunc.make(&loc, false)?))
        }
        Rule::range => {
            let mut inner = inner.into_inner().into_iter();
            let fst = inner.next().unwrap();
            let loc = ("", fst.as_span().clone());
            let start = validate_full_date(fst)?.make(&loc, true)?;
            let snd = inner.next().unwrap();
            let loc = ("", snd.as_span().clone());
            let end = validate_partial_date(start, snd)?.make(&loc, false)?;
            Ok(Period(start, end))
        }
        _ => unreachable!(),
    }
}

fn validate_full_date(p: Pair) -> Result<TruncDate> {
    let mut inner = p.into_inner().into_iter();
    let year = inner.next().unwrap().as_str().parse::<u16>().unwrap();
    match inner.next() {
        None => Ok(TruncDate { year, ..Default::default() }),
        Some(month) => validate_month_date(year, month),
    }
}

fn validate_partial_date(default: Date, p: Pair) -> Result<TruncDate> {
    match p.as_rule() {
        Rule::full_date => validate_full_date(p),
        Rule::month_date => validate_month_date(default.year(), p),
        Rule::day => validate_day_date(default.year(), default.month(), p),
        _ => unreachable!("{:?}", p),
    }
}

fn validate_month_date(year: u16, p: Pair) -> Result<TruncDate> {
    let mut inner = p.into_inner().into_iter();
    let month = inner.next().unwrap();
    let loc = ("", month.as_span().clone());
    let month = month.as_str().parse::<Month>()
        .ok()
        .ok_or_else(|| {
            let mut err = Error::new("Invalid Month");
            err.span(&loc, "provided here")
                .text(format!("'{}' is not a valid month", month.as_str()))
                .hint("Months are 'Jan', 'Feb', ..., 'Dec'");
            err
        })?;
    match inner.next() {
        None => Ok(TruncDate { year, month: Some(month), ..Default::default() }),
        Some(day) => validate_day_date(year, month, day),
    }
}

fn validate_day_date(year: u16, month: Month, p: Pair) -> Result<TruncDate> {
    let day = p.as_str().parse::<u8>().unwrap();
    Ok(TruncDate { year, month: Some(month), day: Some(day) })
}

#[derive(Default, Debug)]
struct TruncDate {
    year: u16,
    month: Option<Month>,
    day: Option<u8>,
}

impl TruncDate {
    fn make(&self, loc: &Loc, starting: bool) -> Result<Date> {
        let year = self.year;
        let month = self.month.unwrap_or(if starting { Month::Jan } else { Month::Dec });
        let day = self.day.unwrap_or(if starting { 1 } else { month.count(year) });
        match Date::from(year as usize, month, day as usize) {
            Ok(date) => Ok(date),
            Err(e) => {
                let mut err = Error::new("Invalid date");
                err.span(loc, "provided here")
                    .text(format!("{}", e))
                    .hint("choose a date that exists")
                    .hint(e.fix_hint());
                Err(err)
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
            match $s.parse::<Period>() {
                Ok(period) => assert_eq!(&format!("{}", period), $res),
                Err(err) => println!("{} ->\n{}", $s, err),
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
