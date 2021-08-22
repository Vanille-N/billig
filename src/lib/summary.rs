use std::ops;

use crate::lib::{
    date::{Date, Between},
    entry::{Amount, Category, Duration, Entry},
};

#[derive(Debug, Clone)]
pub struct Summary {
    /// Period of relevance added entries are to be intersected with
    period: Between<Date>,
    /// Cached total
    total: Amount,
    /// Subtotals per expense kind
    categories: [Amount; Category::COUNT],
}

impl Summary {
    /// Initialize blank
    pub fn from_period(period: Between<Date>) -> Self {
        Self {
            period,
            total: Amount(0),
            categories: [Amount(0); Category::COUNT],
        }
    }

    /// Initialize blank for a single day
    pub fn from_date(date: Date) -> Self {
        Self::from_period(Between(date, date))
    }

    /// Read subtotal for an expense kind
    pub fn query(&self, cat: Category) -> Amount {
        self.categories[cat as usize]
    }

    /// Read all subtotals
    pub fn amounts(&self) -> &[Amount] {
        &self.categories[..]
    }

    pub fn period(&self) -> Between<Date> {
        self.period
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
    /// Periods[d1, d2, d3, ..., dn] -> Calendar[d1..d2, d2..d3, ..., dn-1..dn]
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
                items.push(Summary::from_period(Between(a, b.prev())));
            }
            start = end;
        }
        Self { items }
    }

    /// Construct from a starting point and an _increasing_ step function
    pub fn from_step<F>(mut start: Date, step: F) -> Self
    where
        F: Fn(Date) -> Option<Date>,
    {
        let mut items = Vec::new();
        while let Some(end) = step(start) {
            assert!(start < end);
            items.push(Summary::from_period(Between(start, end.prev())));
            start = end;
        }
        Self { items }
    }

    /// Construct from a standardized span step generator
    pub fn from_spacing(period: Between<Date>, duration: Duration, count: usize) -> Self {
        Self::from_step(period.0, |date| {
            if period.1 <= date {
                return None;
            }
            let next = match duration {
                Duration::Day => date.jump_day(count as isize),
                Duration::Week => date.jump_day(count as isize * 7),
                Duration::Month => date.jump_month(count as isize),
                Duration::Year => date.jump_year(count as isize),
            };
            Some(period.1.min(next))
        })
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

    fn dichotomy_idx(&self, period: Between<Date>) -> Option<(usize, usize)> {
        if self.items.len() == 0 {
            return None;
        }
        let start = self.dichotomy_aux(period.0, 0, self.items.len());
        let end = self.dichotomy_aux(period.1, 0, self.items.len());
        if start <= end
            && self.items[end].period.0 <= period.1
            && self.items[end].period.1 >= period.0
        {
            Some((start, end))
        } else {
            None
        }
    }

    fn dichotomy(&self, period: Between<Date>) -> Option<&[Summary]> {
        let (start, end) = self.dichotomy_idx(period)?;
        Some(&self.items[start..=end])
    }

    fn dichotomy_mut(&mut self, period: Between<Date>) -> Option<&mut [Summary]> {
        let (start, end) = self.dichotomy_idx(period)?;
        Some(&mut self.items[start..=end])
    }

    /// Add all entries to the summary
    pub fn register(&mut self, items: &[Entry]) {
        for item in items {
            if let Some(range) = self.dichotomy_mut(item.period()) {
                for summary in range {
                    *summary += item;
                }
            } else {
                println!("Empty range for {}", item.period());
            }
        }
    }

    pub fn contents(&self) -> &[Summary] {
        &self.items
    }
}

#[rustfmt::skip]
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
        let cal = Calendar::from_spacing(
            Between(dt!(2020-Jan-1), dt!(2020-Dec-31)),
            Duration::Week,
            1
        );
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
        // extremities
        let (_, idx, _) = query!(cal, dt!(2020-Jan-1));
        assert_eq!(idx, 0);
        let (_, idx, _) = query!(cal, dt!(2019-Dec-31));
        assert_eq!(idx, 0);
        let (_, idx, _) = query!(cal, dt!(2020-Dec-31));
        assert_eq!(idx, cal.items.len() - 1);
        let (_, idx, _) = query!(cal, dt!(2021-Jan-1));
        assert_eq!(idx, cal.items.len() - 1);
        // period
        let ans = cal.dichotomy(Between(dt!(2019-Jun-10), dt!(2019-Jun-15)));
        assert!(ans.is_none());
        let ans = cal.dichotomy(Between(dt!(2021-Jun-10), dt!(2021-Jun-15)));
        assert!(ans.is_none());
        let ans = cal.dichotomy(Between(dt!(2019-Jun-10), dt!(2021-Jun-15)));
        assert_eq!(ans.unwrap().len(), cal.items.len());
        let ans = cal.dichotomy(Between(dt!(2020-Jan-20), dt!(2020-Mar-18))).unwrap();
        assert!(ans[0].period.0 <= dt!(2020-Jan-20));
        assert!(ans[0].period.1 >= dt!(2020-Jan-20));
        assert!(ans[ans.len() - 1].period.0 <= dt!(2020-Mar-18));
        assert!(ans[ans.len() - 1].period.1 >= dt!(2020-Mar-18));
    }
}
