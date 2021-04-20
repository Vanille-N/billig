#[derive(Debug, Clone)]
pub struct Date {
    year: usize,
    month: Month,
    day: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Month {
    Jan,
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
}

#[derive(Clone, Debug)]
pub enum DateError {
    UnsupportedYear(usize),
    NotBissextile(usize),
    MonthTooShort(usize, Month, usize),
    InvalidDay(usize, Month, usize),
}

impl Date {
    pub fn from(year: usize, month: Month, day: usize) -> Result<Self, DateError> {
        use Month::*;
        if day > 31 || day == 0 {
            return Err(DateError::InvalidDay(year, month, day));
        } else if day == 31 {
            match month {
                Jan | Mar | May | Jul | Aug | Oct | Dec => Ok(Self { year, month, day }),
                _ => Err(DateError::MonthTooShort(year, month, day)),
            }
        } else if day == 30 {
            if month == Feb {
                Err(DateError::MonthTooShort(year, month, day))
            } else {
                Ok(Self { year, month, day })
            }
        } else if day == 29 {
            if month == Feb && !is_bissextile(year) {
                Err(DateError::NotBissextile(year))
            } else {
                Ok(Self { year, month, day })
            }
        } else {
            Ok(Self { year, month, day })
        }
    }
}

fn is_bissextile(year: usize) -> bool {
    if year % 400 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 4 != 0 {
        false
    } else {
        true
    }
}
    

use std::fmt;
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
