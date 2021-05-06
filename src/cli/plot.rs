use crate::lib::{
    date::{Period, Date},
    entry::{Amount, Category},
    summary::Summary,
};

pub struct Plotter<'d> {
    data: &'d [Summary],
}

impl<'d> Plotter<'d> {
    pub fn from(data: &'d [Summary]) -> Self {
        Self {
            data,
        }
    }

    pub fn print_cumulative_plot(&self) {
        println!("{:?}", self.cumulative_plot());
    }

    fn cumulative_plot(&self) -> Plot<Period, CumulativeEntry<Amount>> {
        let mut plot = Plot::new();
        for sum in self.data {
            plot.push(sum.period(), CumulativeEntry::cumul(sum.amounts().to_vec()));
        }
        plot
    }
}

#[derive(Debug)]
pub struct Plot<X, Y> {
    data: Vec<(X, Y)>,
}

impl<X, Y> Plot<X, Y> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    fn push(&mut self, x: X, y: Y) {
        self.data.push((x, y));
    }
}

#[derive(Debug)]
struct CumulativeEntry<Y> {
    points: Vec<Y>,
}

impl<Y> CumulativeEntry<Y>
where Y: std::ops::AddAssign + Clone {
    fn cumul(mut points: Vec<Y>) -> Self {
        for i in 1..points.len() {
            let prev = points[i - 1].clone();
            points[i] += prev;
        }
        Self { points }
    }
}

trait Scalar {
    fn to_scalar(&self) -> i64;
}
trait ScalarRange {
    fn to_range(&self) -> (i64, i64);
}
trait ScalarGroup {
    fn to_group(&self) -> Vec<i64>;
}

impl Scalar for Amount {
    fn to_scalar(&self) -> i64 {
        self.0 as i64
    }
}

impl Scalar for Date {
    fn to_scalar(&self) -> i64 {
        self.index() as i64
    }
}

impl ScalarRange for Period {
    fn to_range(&self) -> (i64, i64) {
        (self.0.to_scalar(), self.1.to_scalar())
    }
}

impl<Y> ScalarGroup for CumulativeEntry<Y>
where Y: Scalar {
    fn to_group(&self) -> Vec<i64> {
        self.points.iter()
            .map(|p| p.to_scalar())
            .collect::<Vec<_>>()
    }
}
