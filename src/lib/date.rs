use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;

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

    pub fn next(self) -> Self {
        Self::from_isize((self as isize + 1) % 12).unwrap()
    }
    pub fn prev(self) -> Self {
        Self::from_isize((self as isize + 11) % 12).unwrap()
    }
    pub fn count(self, year: u16) -> u8 {
        use Month::*;
        match self {
            Jan | Mar | May | Jul | Aug | Oct | Dec => 31,
            Apr | Jun | Sep | Nov => 30,
            Feb => if is_bissextile(year) { 29 } else { 28 },
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
    pub fn next(self) -> Self {
        Self::from_isize((self as isize + 1) % 7).unwrap()
    }
    pub fn prev(self) -> Self {
        Self::from_isize((self as isize + 6) % 7).unwrap()
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateError {
    UnsupportedYear(usize),
    NotBissextile(usize),
    MonthTooShort(Month, usize),
    InvalidDay(usize),
}

impl Date {
    pub fn from(year: usize, month: Month, day: usize) -> Result<Self, DateError> {
        if !(2000..=4000).contains(&year) {
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

    pub fn day(&self) -> u8 {
        self.day
    }

    pub fn month(&self) -> Month {
        self.month
    }

    pub fn year(&self) -> u16 {
        self.year
    }

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
        for m in 0..self.month as usize {
            n += [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m];
        }
        n += leaps; // each leap year adds one day
        n
    }

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

    pub fn jump_year(self, count: isize) -> Self {
        let year = (self.year as isize + count) as u16;
        if self.month == Month::Feb && self.day == 29 && !is_bissextile(year) {
            Self { year, day: 28, ..self }
        } else {
            Self { year, ..self }
        }
    }

    pub fn start_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    pub fn end_of_month(self) -> Self {
        Self { day: self.month.count(self.year), ..self }
    }

    pub fn start_of_year(self) -> Self {
        Self { day: 1, month: Month::Jan, ..self }
    }

    pub fn end_of_year(self) -> Self {
        Self { day: 31, month: Month::Dec, ..self }
    }
    
    pub fn start_of_week(self) -> Self {
        self.jump_day(-(self.weekday() as isize))
    }

    pub fn end_of_week(self) -> Self {
        self.jump_day(6 - self.weekday() as isize)
    }    
}

fn is_bissextile(year: usize) -> bool {
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
            UnsupportedYear(y) => writeln!(f, "{} is outside of the supported range for years", y),
            NotBissextile(y) => writeln!(f, "{} is not bissextile, so Feb 29 does not exist", y),
            MonthTooShort(y, m, d) => writeln!(
                f,
                "In {}: {:?} is a short month, it does not have a {}th day",
                y, m, d,
            ),
            InvalidDay(y, m, d) => writeln!(f, "In {} {:?}: {} is not a valid day", y, m, d),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        *,
        Month::*,
    };

    #[test]
    fn bissextile_check() {
        macro_rules! yes {
            ( $y:expr ) => { assert!(is_bissextile($y)); }
        }
        macro_rules! no {
            ( $y:expr ) => { assert!(!is_bissextile($y)); }
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
            assert_eq!(Date::from($y, $m, $d), Err(DateError::MonthTooShort($y, $m, $d)));
        }
    }
    macro_rules! nbiss {
        ( $y:tt - $m:tt - $d:tt ) => {
            assert_eq!(Date::from($y, $m, $d), Err(DateError::NotBissextile($y)));
        }
    }
    macro_rules! invalid {
        ( $y:tt - $m:tt - $d:tt ) => {
            assert_eq!(Date::from($y, $m, $d), Err(DateError::InvalidDay($y, $m, $d)));
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
}
