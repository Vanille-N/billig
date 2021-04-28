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


