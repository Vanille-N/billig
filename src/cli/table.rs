use num_traits::FromPrimitive;
use std::fmt;

use crate::lib::{
    summary::Summary,
    entry::Category,
};

pub struct Table<'d> {
    data: &'d [Summary],
}

impl<'d> Table<'d> {
    pub fn from(data: &'d [Summary]) -> Self {
        Self { data }
    }
}


impl fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let columns = (0..Category::COUNT)
            .map(|i| Category::from_usize(i).unwrap())
            .collect::<Vec<_>>();
        for sum in self.data {
            let pp = format!("{}", sum.period());
            write!(f, "| {:>15} |", pp)?;
            for cat in &columns {
                let amount = sum.query(*cat);
                let pp = if amount.nonzero() {
                    format!("{}", sum.query(*cat))
                } else {
                    String::new()
                };
                write!(f, " {:>10} |", pp)?;
            }
            writeln!(f);
        }
        Ok(())
    }
}
