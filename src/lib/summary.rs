use std::ops;

use crate::lib::{
    date::{Date, Period},
    entry::{Category, Amount, Entry, CATEGORY_COUNT},
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


/// A collection of summaries
#[derive(Debug)]
pub struct Calendar {
    /// 
    /// Assumption : any two adjacent summaries can be compared in the sense
    /// ```
    /// let lhs = items[idx].period;
    /// let rhs = items[idx+1].period;
    /// assert!(lhs.0 <= rhs.0);
    /// assert!(lhs.1 <= rhs.1);
    /// ```
    /// In practice this is guaranteed by the fact that no constructor
    /// provides a way of creating a `Calendar` for which the summaries
    /// are not of disjoint periods.
    items: Vec<Summary>,
}

impl Calendar {
    /// Construct from an _increasing_ iterator of dates
    pub fn from_iter<I>(mut splits: I) -> Self
    where I: Iterator<Item = Date> {
        let mut items = Vec::new();
        let mut start = splits.next();
        while let Some(a) = start {
            let end = splits.next();
            assert!(start <= end);
            if let Some(b) = end {
                items.push(Summary::new_period((a, b)));
            }
            start = end;
        }
        Self {
            items,
        }
    }

    pub fn from_step<F>(mut start: Date, step: F) -> Self
    where F: Fn(Date) -> Option<Date> {
        let mut items = Vec::new();
        while let Some(end) = step(start) {
            assert!(start <= end);
            items.push(Summary::new_period((start, end)));
            start = end;
        }
        Self {
            items,
        }
    }
}
        

