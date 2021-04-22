use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: u16,
    month: Month,
    day: u8,
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

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
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
    MonthTooShort(usize, Month, usize),
    InvalidDay(usize, Month, usize),
}

impl Date {
    pub fn from(year: usize, month: Month, day: usize) -> Result<Self, DateError> {
        use Month::*;
        if !(2000..=4000).contains(&year) {
            Err(DateError::UnsupportedYear(year))
        } else if day > 31 || day == 0 {
            Err(DateError::InvalidDay(year, month, day))
        } else if day == 31 {
            match month {
                Jan | Mar | May | Jul | Aug | Oct | Dec => Ok(Self {
                    year: year as u16,
                    month,
                    day: day as u8,
                }),
                _ => Err(DateError::MonthTooShort(year, month, day)),
            }
        } else if day == 30 {
            if month == Feb {
                Err(DateError::MonthTooShort(year, month, day))
            } else {
                Ok(Self {
                    year: year as u16,
                    month,
                    day: day as u8,
                })
            }
        } else if day == 29 {
            if month == Feb && !is_bissextile(year) {
                Err(DateError::NotBissextile(year))
            } else {
                Ok(Self {
                    year: year as u16,
                    month,
                    day: day as u8,
                })
            }
        } else {
            Ok(Self {
                year: year as u16,
                month,
                day: day as u8,
            })
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

    pub fn weekday(&self) -> Weekday {
        Weekday::Mon
        // TODO
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{:02}", self.year, self.month, self.day)
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
