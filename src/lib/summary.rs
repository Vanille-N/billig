use std::ops;

use crate::lib::{
    date::{Date, Period},
    entry::{Amount, Category, Entry, Duration, CATEGORY_COUNT},
};

#[derive(Debug, Clone)]
pub struct Summary {
    period: Period,
    total: Amount,
    categories: [Amount; CATEGORY_COUNT],
}

impl Summary {
    pub fn new_period(period: Period) -> Self {
        Self {
            period,
            total: Amount::from(0),
            categories: [Amount::from(0); CATEGORY_COUNT],
        }
    }

    pub fn new_date(date: Date) -> Self {
        Self::new_period((date, date))
    }

    pub fn query(&self, cat: Category) -> Amount {
        self.categories[cat as usize]
    }

    pub fn total(&self) -> Amount {
        self.total
    }
}

impl ops::AddAssign<&Entry> for Summary {
    fn add_assign(&mut self, entry: &Entry) {
        if let Some(entry) = entry.intersect_loss(self.period) {
            let idx = entry.category() as usize;
            let add = entry.value();
            self.categories[idx] += add;
            self.total += add;
        }
    }
}

/// A collection of disjoint ordered summaries
#[derive(Debug)]
pub struct Calendar {
    items: Vec<Summary>,
}

impl Calendar {
    /// Construct from an _increasing_ iterator of dates
    pub fn from_iter<I>(mut splits: I) -> Self
    where
        I: Iterator<Item = Date>,
    {
        let mut items = Vec::new();
        let mut start = splits.next();
        while let Some(a) = start {
            let end = splits.next();
            if let Some(b) = end {
                assert!(start < end);
                items.push(Summary::new_period((a, b.prev())));
            }
            start = end;
        }
        Self { items }
    }

    /// Construct from a starting point and a step function
    pub fn from_step<F>(mut start: Date, step: F) -> Self
    where
        F: Fn(Date) -> Option<Date>,
    {
        let mut items = Vec::new();
        while let Some(end) = step(start) {
            assert!(start < end);
            items.push(Summary::new_period((start, end.prev())));
            start = end;
        }
        Self { items }
    }

    pub fn from_spacing(period: Period, duration: Duration, count: usize) -> Self {
        Self::from_step(
            period.0,
            |date| {
                let next = match duration {
                    Duration::Day => date.jump_day(count as isize),
                    Duration::Week => date.jump_day(count as isize * 7),
                    Duration::Month => date.jump_month(count as isize),
                    Duration::Year => date.jump_year(count as isize),
                };
                if next <= period.1 {
                    Some(next)
                } else {
                    None
                }
            },
        )
    }
    
    /// Find index that contains `target`
    ///
    /// `start` is large, `end` is strict
    fn dichotomy_aux(&self, target: Date, start: usize, end: usize) -> usize {
        if start + 1 >= end {
            return start;
        }
        let mid = (start + end) / 2;
        if self.items[mid].period.0 > target {
            self.dichotomy_aux(target, start, mid)
        } else {
            self.dichotomy_aux(target, mid, end)
        }
    }

    fn dichotomy(&self, period: Period) -> &[Summary] {
        let start = self.dichotomy_aux(period.0, 0, self.items.len());
        let end = self.dichotomy_aux(period.1, 0, self.items.len());
        if start <= end && self.items[end].period.0 <= period.1 && self.items[end].period.1 >= period.0 {
            &self.items[start..=end]
        } else {
            &self.items[0..0]
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::lib::{
        date::{Date, Month::*},
        entry::Duration::*,
    };

    macro_rules! dt {
        ( $y:tt - $m:tt - $d:tt ) => {{
            Date::from($y, $m, $d).unwrap()
        }}
    }

    macro_rules! query {
        ( $cal:expr, $date:expr ) => {{
            let date = $date;
            let len = $cal.items.len();
            let idx = $cal.dichotomy_aux(date, 0, len);
            let start = &$cal.items[idx];
            (date, idx, start.period)
        }}
    }

    #[test]
    fn dichotomies() {
        let cal = Calendar::from_spacing((dt!(2020-Jan-1), dt!(2020-Dec-31)), Duration::Week, 1);
        println!("{:?}", cal);
        // middle
        let (date, _, start) = query!(cal, dt!(2020-Feb-5));
        assert!(start.0 <= date && date <= start.1);
        let (date, _, start) = query!(cal, dt!(2020-Mar-7));
        assert!(start.0 <= date && date <= start.1);
        let (date, _, start) = query!(cal, dt!(2020-Feb-4));
        assert!(start.0 <= date && date <= start.1);
        let (date, _, start) = query!(cal, dt!(2020-Nov-15));
        assert!(start.0 <= date && date <= start.1);
        let (date, _, start) = query!(cal, dt!(2020-Jan-7));
        assert!(start.0 <= date && date <= start.1);
