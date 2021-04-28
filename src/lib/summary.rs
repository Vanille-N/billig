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
