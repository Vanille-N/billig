//! Day-precise time management, with a focus on edge cases
//!
//! Dates are `YYYY-Mmm-DD`, not number of seconds, and provide an interface
//! for dealing with durations that are expressed in number of days, weeks, months
//! or years.
//!
//! They also support weekday calculations, and jumping to the boundaries of
//! a time frame (see for example `start_of_week` or `end_of_month`)

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;

/// A date with day-precision
///
/// Supports years in the range 1000..=9999, but weekday conversion
/// is not guaranteed accurate before 1900.
///
/// All methods execute in constant time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: Month,
    day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{:02}", self.year, self.month, self.day)
    }
}

/// Twelve months in the year, identified by their 3-letter abbreviations
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, PartialOrd, Ord)]
pub enum Month {
    Jan = 0,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    /// Parse a month from its stringified name (`"Jan"`, `"Feb"`, `"Mar"`, ...)
    ///
    /// # Panics
    ///
    /// This function will panic if the string is not a valid 3-character month name.
    ///
    /// It is meant to translate text matched by the grammar, not validate arbitrary
    /// user input.
    pub fn from(s: &str) -> Self {
        use Month::*;
        match s {
            "Jan" => Jan,
            "Feb" => Feb,
            "Mar" => Mar,
            "Apr" => Apr,
            "May" => May,
            "Jun" => Jun,
            "Jul" => Jul,
            "Aug" => Aug,
            "Sep" => Sep,
            "Oct" => Oct,
            "Nov" => Nov,
            "Dec" => Dec,
            _ => unreachable!(),
        }
    }

    /// Month directly succeeding the current one with wrapping
    pub fn next(self) -> Self {
        Self::from_isize((self as isize + 1) % 12).unwrap()
    }

    /// Month directly preceding the current one with wrapping
    pub fn prev(self) -> Self {
        Self::from_isize((self as isize + 11) % 12).unwrap()
    }

    /// Number of days in this month of the given year
    pub fn count(self, year: u16) -> u8 {
        use Month::*;
        match self {
            Jan | Mar | May | Jul | Aug | Oct | Dec => 31,
            Apr | Jun | Sep | Nov => 30,
            Feb => if is_leap(year) { 29 } else { 28 },
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Weekday with Monday-first week convention
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
pub enum Weekday {
    Mon = 0,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    /// Weekday directly succeeding the current one with wrapping
    pub fn next(self) -> Self {
        Self::from_isize((self as isize + 1) % 7).unwrap()
    }

    /// Weekday directly preceding the current one with wrapping
    pub fn prev(self) -> Self {
        Self::from_isize((self as isize + 6) % 7).unwrap()
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Ways in which a date taken from user input can be wrong
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateError {
    /// year is outside of 1000..=9999
    UnsupportedYear(usize),
    /// Feb 29 of a non-leap year
    NotBissextile(usize),
    /// Feb 30 or Feb 31 or 31st day of a 30-day month
    MonthTooShort(Month, usize),
    /// day outside of 1..=31
    InvalidDay(usize),
}

impl Date {
    /// Validate year-month-day into date
    pub fn from(year: usize, month: Month, day: usize) -> Result<Self, DateError> {
        if !(1000..=9999).contains(&year) {
            Err(DateError::UnsupportedYear(year))
        } else if day == 0 || day > 31 {
            Err(DateError::InvalidDay(day))
        } else if day <= month.count(year as u16) as usize {
            Ok(Self { year: year as u16, month, day: day as u8 })
        } else if day >= 30 {
            Err(DateError::MonthTooShort(month, day))
        } else {
            Err(DateError::NotBissextile(year))
        }
    }

    /// `self.day` accessor
    pub fn day(&self) -> u8 {
        self.day
    }

    /// `self.month` accessor
    pub fn month(&self) -> Month {
        self.month
    }

    /// `self.year` accessor
    pub fn year(&self) -> u16 {
        self.year
    }

    /// Biject the dates with integers
    ///
    /// This indexing is guaranteed consistent in the sense that
    /// for any date `d`,
    ///
    ///     assert_eq!(d.index() + 1, d.next().index());
    pub fn index(self) -> usize {
        let leaps = {
            let years = if self.month <= Month::Feb {
                self.year as usize - 1
            } else {
                self.year as usize
            };
            // count leap years before current
            (years / 4) - (years / 100) + (years / 400) 
        };
        let mut n = self.year as usize * 365 + self.day as usize;
        // partially elapsed current year
        n += [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][self.month as usize];
        n += leaps; // each leap year adds one day
        n
    }

    /// Get day of week
    pub fn weekday(self) -> Weekday {
        let offset = 2; // essentially the weekday of 0000-Jan-01
        Weekday::from_usize((self.index() - offset) % 7).unwrap()
    }

    pub fn next(self) -> Self {
        if self.month.count(self.year) == self.day {
            if self.month == Month::Dec {
                Self { year: self.year + 1, month: Month::Jan, day: 1 }
            } else {
                Self { month: self.month.next(), day: 1, ..self }
            }
        } else {
            Self { day: self.day + 1, ..self }
        }
    }

    pub fn prev(self) -> Self {
        if self.day == 1 {
            if self.month == Month::Jan {
                Self { year: self.year - 1, month: Month::Dec, day: 31 }
            } else {
                let month = self.month.prev();
                Self { month, day: month.count(self.year), ..self }
            }
        } else {
            Self { day: self.day - 1, ..self }
        }
    }


    /// `count` days before/after current date
    pub fn jump_day(self, count: isize) -> Self {
        let full_count = count;
        // first rough approximation to get
        // the year and month as close as possible
        let (d, count) = if count > 30 {
            let target = self.index() as isize + count;
            let adjust_year = self.jump_year(count / 365);
            let adjust_month = adjust_year.jump_month((target - adjust_year.index() as isize) / 31);
            #[cfg(test)]
            println!("{} -> {} -> {}", self, adjust_year, adjust_month);
            (adjust_month, target - adjust_month.index() as isize)
        } else {
            (self, count)
        };
        let mut d = d;
        if count > 0 {
            let mut count = count as u8;
            while count > 0 {
                let diff = (d.month.count(d.year) - d.day).min(count);
                d.day += diff;
                count -= diff;
                if count > 0 {
                    d = d.next();
                    count -= 1;
                }
            }
        } else {
            let mut count = (-count) as u8;
            while count > 0 {
                let diff = (d.day - 1).min(count);
                d.day -= diff;
                count -= diff;
                if count > 0 {
                    d = d.prev();
                    count -= 1;
                }
            }
        }
        assert_eq!(d.index() as isize, self.index() as isize + full_count);
        d
    }

    /// `count` months before/after current date
    ///
    /// Day will be truncated to fit in the new month:
    /// adding one month to `2000-Jan-31` makes it `2000-Feb-29`
    pub fn jump_month(self, count: isize) -> Self {
        let (year, month) = {
            let mut year = self.year as isize;
            let mut month = self.month as isize + count;
            while month < 0 {
                month += 12;
                year -= 1;
            }
            while month >= 12 {
                month -= 12;
                year += 1;
            }
            (year as u16, Month::from_isize(month).unwrap())
        };
        Self {
            year,
            month,
            day: self.day.min(month.count(year)),
        }
    }

    /// `count` years before/after current date
    ///
    /// Day will be truncated in the rare case it is needed:
    /// adding one year to `2000-Feb-29` makes it `2001-Feb-28`
    pub fn jump_year(self, count: isize) -> Self {
        let year = (self.year as isize + count) as u16;
        if self.month == Month::Feb && self.day == 29 && !is_leap(year) {
            Self { year, day: 28, ..self }
        } else {
            Self { year, ..self }
        }
    }

    /// Get date of the first day of the current month
    pub fn start_of_month(self) -> Self {
        Self { day: 1, ..self }
    }
    
    /// Get date of the last day of the current month
    pub fn end_of_month(self) -> Self {
        Self { day: self.month.count(self.year), ..self }
    }

    /// Jan 1st of the current year
    pub fn start_of_year(self) -> Self {
        Self { day: 1, month: Month::Jan, ..self }
    }

    /// Dec 31st of the current year
    pub fn end_of_year(self) -> Self {
        Self { day: 31, month: Month::Dec, ..self }
    }
    
    /// First Monday before the current date
    pub fn start_of_week(self) -> Self {
        self.jump_day(-(self.weekday() as isize))
    }
    
    /// First Sunday after the current date
    pub fn end_of_week(self) -> Self {
        self.jump_day(6 - self.weekday() as isize)
    }

    /// Set maximum value for day
    pub fn cap_day(mut self, d: u8) -> Self {
        let m = self.month;
        while self.day >= d && self.month == m {
            self = self.prev();
        }
        self
    }
}


fn is_leap(year: u16) -> bool {
    if year % 400 == 0 {
        true
    } else if year % 100 == 0 {
        false
    } else {
        year % 4 == 0
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DateError::*;
        match self {
            UnsupportedYear(y) => write!(f, "{} is outside of the supported range for years", y),
            NotBissextile(y) => write!(f, "{} is not bissextile, Feb 29 does not exist", y),
            MonthTooShort(m, d) => write!(
                f,
                "{} is a short month, it does not have a {}th day",
                m, d,
            ),
            InvalidDay(d) => write!(f, "{} is not a valid day", d),
        }
    }
}

impl DateError {
    /// What message to show to help fix the date error
    pub fn fix_hint(self) -> String {
        use DateError::*;
        match self {
            UnsupportedYear(_) => "year should be between 1000 and 9999 inclusive".to_string(),
            NotBissextile(y) => format!("did you mean {y}-Feb-28 or {y}-Mar-01 ?", y = y),
            MonthTooShort(m, d) => format!("{} is only {} days long", m,
                if m == Month::Feb { 28.max(d - 1) } else { 30 }
            ),
            InvalidDay(d) => format!("{} is not in the range 1 ..= 31", d),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        *,
        Month::*,
        Weekday::*,
    };

    #[test]
    fn bissextile_check() {
        macro_rules! yes {
            ( $y:expr ) => { assert!(is_leap($y)); }
        }
        macro_rules! no {
            ( $y:expr ) => { assert!(!is_leap($y)); }
        }
        yes!(2004);
        no!(2100);
        yes!(2000);
        no!(2001);
        no!(2010);
        yes!(2012);
    }

    macro_rules! ok {
        ( $y:tt - $m:tt - $d:tt ) => {
            assert_eq!(Date::from($y, $m, $d), Ok(Date { year: $y, month: $m, day: $d }));
        }
    }
    macro_rules! short {
        ( $y:tt - $m:tt - $d:tt ) => {
            assert_eq!(Date::from($y, $m, $d), Err(DateError::MonthTooShort($m, $d)));
        }
    }
    macro_rules! nbiss {
        ( $y:tt - $m:tt - $d:tt ) => {
            assert_eq!(Date::from($y, $m, $d), Err(DateError::NotBissextile($y)));
        }
    }
    macro_rules! invalid {
        ( $y:tt - $m:tt - $d:tt ) => {
            assert_eq!(Date::from($y, $m, $d), Err(DateError::InvalidDay($d)));
        }
    }
    
    #[test]
    fn long_months() {
        ok!(2020-Jan-31);
        ok!(2020-Mar-31);
        short!(2020-Apr-31);
        ok!(2020-May-31);
        short!(2020-Jun-31);
        ok!(2020-Jul-31);
        ok!(2020-Aug-31);
        short!(2020-Sep-31);
        ok!(2020-Oct-31);
        short!(2020-Nov-31);
        ok!(2020-Dec-31);
    }

    #[test]
    fn normal_days() {
        invalid!(2020-Dec-45);
        invalid!(2020-Jan-32);
        invalid!(2020-Jan-0);
        ok!(2020-Mar-20);
        ok!(2020-Apr-10);
    }

    #[test]
    fn february() {
        short!(2020-Feb-31);
        short!(2020-Feb-30);
        ok!(2020-Feb-29);
        ok!(2020-Feb-28);
        short!(2021-Feb-31);
        short!(2021-Feb-30);
        nbiss!(2021-Feb-29);
        ok!(2021-Feb-28);
    }

    macro_rules! dt {
        ( $y:tt - $m:tt - $d:tt ) => {
            Date::from($y, $m, $d).unwrap()
        }
    }
    
    macro_rules! day {
        ( $d:expr => $w:expr ) => {
            assert_eq!($d.weekday(), $w)
        }
    }
    #[test]
    fn weekday_references() {
        // accross a week
        day!(dt!(2000-Jan-1) => Sat);
        day!(dt!(2000-Jan-2) => Sun);
        day!(dt!(2000-Jan-3) => Mon);
        day!(dt!(2000-Jan-4) => Tue);
        day!(dt!(2000-Jan-5) => Wed);
        day!(dt!(2000-Jan-6) => Thu);
        day!(dt!(2000-Jan-7) => Fri);
        day!(dt!(2000-Jan-8) => Sat);
        // accross months
        day!(dt!(2000-Feb-1) => Tue);
        day!(dt!(2000-Mar-1) => Wed);
        day!(dt!(2000-Apr-1) => Sat);
        day!(dt!(2000-May-1) => Mon);
        day!(dt!(2000-Jun-1) => Thu);
        day!(dt!(2000-Jul-1) => Sat);
        day!(dt!(2000-Aug-1) => Tue);
        day!(dt!(2000-Sep-1) => Fri);
        day!(dt!(2000-Oct-1) => Sun);
        day!(dt!(2000-Nov-1) => Wed);
        day!(dt!(2000-Dec-1) => Fri);
        // accross years
        day!(dt!(2000-Dec-31) => Sun);
        day!(dt!(2001-Dec-31) => Mon);
        day!(dt!(2002-Dec-31) => Tue);
        day!(dt!(2003-Dec-31) => Wed);
        day!(dt!(2004-Dec-31) => Fri);
        day!(dt!(2005-Dec-31) => Sat);
        day!(dt!(2006-Dec-31) => Sun);
        day!(dt!(2007-Dec-31) => Mon);
        day!(dt!(2008-Dec-31) => Wed);
        day!(dt!(2009-Dec-31) => Thu);
        day!(dt!(2010-Dec-31) => Fri);
        // accross centuries
        day!(dt!(2000-Jul-14) => Fri);
        day!(dt!(2100-Jul-14) => Wed);
        day!(dt!(2200-Jul-14) => Mon);
        day!(dt!(2300-Jul-14) => Sat);
        day!(dt!(2400-Jul-14) => Fri);
        day!(dt!(2500-Jul-14) => Wed);
    }

    #[test]
    fn weekday_consistent() {
        let mut d = Date::from(2000, Jan, 1).unwrap();
        let end = Date::from(3000, Dec, 31).unwrap();
        while d < end {
            let ds = d.next();
            let w = d.weekday().next();
            let ws = ds.weekday();
            if w != ws {
                panic!("date {}, successor {}, expected {} == {}", d, ds, w, ws);
            }
            d = ds;
        }
    }

    #[test]
    fn index_consistent() {
        let mut d = Date::from(2000, Jan, 1).unwrap();
        let end = Date::from(3000, Dec, 31).unwrap();
        while d < end {
            let ds = d.next();
            let n = d.index() + 1;
            let ns = ds.index();
            if n != ns {
                panic!("date {}, successor {}, expected {} == {}", d, ds, n, ns);
            }
            d = ds;
        }
    }

    macro_rules! jday {
        ( $d1:expr, $d2:expr ) => {{
            assert_eq!($d1.jump_day(1), $d2);
            assert_eq!($d2.jump_day(-1), $d1);
        }}
    }
    macro_rules! jmonth {
        ( $d1:expr, $n:expr, <->, $d2:expr ) => {{
            assert_eq!($d1.jump_month($n), $d2);
            assert_eq!($d2.jump_month(-$n), $d1);
        }};
        ( $d1:expr, $n:expr, ->, $d2:expr ) => {{
            assert_eq!($d1.jump_month($n), $d2);
        }};
        ( $d1:expr, $n:expr, <-, $d2:expr ) => {{
            assert_eq!($d2.jump_month(-$n), $d1);
        }};

    }
    macro_rules! jyear {
        ( $d1:expr, $n:expr, <->, $d2:expr ) => {{
            assert_eq!($d1.jump_year($n), $d2);
            assert_eq!($d2.jump_year(-$n), $d1);
        }};
        ( $d1:expr, $n:expr, ->, $d2:expr ) => {{
            assert_eq!($d1.jump_year($n), $d2);
        }};
        ( $d1:expr, $n:expr, <-, $d2:expr ) => {{
            assert_eq!($d2.jump_year(-$n), $d1);
        }};
    }

    #[test]
    fn jump_day() {
        jday!(dt!(2020-Jan-1), dt!(2020-Jan-2));
        jday!(dt!(2020-Jan-15), dt!(2020-Jan-16));
        jday!(dt!(2020-Jan-30), dt!(2020-Jan-31));
        jday!(dt!(2020-Jan-31), dt!(2020-Feb-1));
        jday!(dt!(2020-Feb-28), dt!(2020-Feb-29));
        jday!(dt!(2021-Feb-28), dt!(2021-Mar-1));
        jday!(dt!(2020-Apr-30), dt!(2020-May-1));
        jday!(dt!(2020-Dec-30), dt!(2020-Dec-31));
        jday!(dt!(2020-Dec-31), dt!(2021-Jan-1));
    }

    #[test]
    fn big_jump_day() {
        assert_eq!(dt!(2000-Jan-1).jump_day(365242), dt!(2999-Dec-31));
    }

    #[test]
    fn jump_month() {
        jmonth!(dt!(2020-Jan-1), 2, <->, dt!(2020-Mar-1));
        jmonth!(dt!(2020-Dec-1), 1, <->, dt!(2021-Jan-1));
        jmonth!(dt!(2020-Dec-30), 1, <->, dt!(2021-Jan-30));
        jmonth!(dt!(2020-Mar-31), 1, ->, dt!(2020-Apr-30));
        jmonth!(dt!(2019-Dec-31), 2, ->, dt!(2020-Feb-29));
        jmonth!(dt!(2021-Jan-31), 1, ->, dt!(2021-Feb-28));
        jmonth!(dt!(2021-Feb-1), 1, <-, dt!(2021-Mar-1));
        jmonth!(dt!(2021-Feb-28), 1, <-, dt!(2021-Mar-28));
        jmonth!(dt!(2020-Jan-15), 25, <->, dt!(2022-Feb-15));
    }

    #[test]
    fn jump_year() {
        jyear!(dt!(2020-Jan-1), 1, <->, dt!(2021-Jan-1));
        jyear!(dt!(2020-Feb-28), 5, <->, dt!(2025-Feb-28));
        jyear!(dt!(2020-Feb-29), 5, ->, dt!(2025-Feb-28));
        jyear!(dt!(2019-Feb-28), 1, <-, dt!(2020-Feb-29));
    }

    #[test]
    fn time_boundaries() {
        assert_eq!(dt!(2020-Mar-26).start_of_month(), dt!(2020-Mar-1));
        assert_eq!(dt!(2020-Feb-12).end_of_month(), dt!(2020-Feb-29));
        assert_eq!(dt!(2020-Dec-31).start_of_year(), dt!(2020-Jan-1));
        assert_eq!(dt!(2020-Dec-5).end_of_year(), dt!(2020-Dec-31));
        for i in 1..15 {
            day!(dt!(2020-Jan-i).start_of_week() => Mon);
            day!(dt!(2020-Jan-i).end_of_week() => Sun);
        }
        day!(dt!(2000-Jan-5) => Wed);
        assert_eq!(dt!(2000-Jan-5).start_of_week(), dt!(2000-Jan-3));
        assert_eq!(dt!(2000-Jan-5).end_of_week(), dt!(2000-Jan-9));
    }
}
