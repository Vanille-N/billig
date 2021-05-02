use num_traits::FromPrimitive;
use std::fmt;

use crate::lib::{
    summary::Summary,
    entry::Category,
};

pub struct Table<'d> {
    data: &'d [Summary],
}

struct BoxFmt {
    width: usize,
    text: String,
}

struct ColFmt {
    width: usize,
    label: BoxFmt,
    boxes: Vec<BoxFmt>,
}

struct GridFmt {
    labels: ColFmt,
    columns: Vec<ColFmt>,
}

impl<'d> Table<'d> {
    pub fn from(data: &'d [Summary]) -> Self {
        Self { data }
    }

    fn to_formatter(&self) -> GridFmt {
        let columns = (0..Category::COUNT)
            .map(|i| Category::from_usize(i).unwrap())
            .collect::<Vec<_>>();
        let cols = columns
            .iter()
            .map(|c| BoxFmt::category(*c))
            .map(|b| ColFmt::with_label(b))
            .collect::<Vec<_>>();
        let mut grid = GridFmt::with_columns(cols);
        for sum in self.data {
            grid.push_line(BoxFmt::period(sum.period()), sum.amounts().iter().map(|f| BoxFmt::amount(*f)).collect::<Vec<_>>());
        }
        grid
    }
}


impl fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_formatter())
    }
}
            }
            writeln!(f);
        }
        Ok(())
    }
}
