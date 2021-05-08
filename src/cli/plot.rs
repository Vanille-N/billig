use crate::lib::{
    date::{Date, Period},
    entry::{Amount, Category},
    summary::Summary,
};

pub struct Plotter<'d> {
    data: &'d [Summary],
}

impl<'d> Plotter<'d> {
    pub fn from(data: &'d [Summary]) -> Self {
        Self { data }
    }

    pub fn print_cumulative_plot(&self) {
        println!("{:?}", self.cumulative_plot().to_range_group_drawer().render("img.svg"));
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
        Self { data: Vec::new() }
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
where
    Y: std::ops::AddAssign + Clone,
{
    fn cumul(mut points: Vec<Y>) -> Self {
        for i in 1..points.len() {
            let prev = points[i - 1].clone();
            points[i] += prev;
        }
        Self { points }
    }
}

pub trait Scalar {
    fn to_scalar(&self) -> i64;
}
pub trait ScalarRange {
    fn to_range(&self) -> (i64, i64);
}
pub trait ScalarGroup {
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

impl<T> ScalarRange for (T, T)
where T: Scalar {
    fn to_range(&self) -> (i64, i64) {
        (self.0.to_scalar(), self.1.to_scalar())
    }
}

impl<Y> ScalarGroup for CumulativeEntry<Y>
where
    Y: Scalar,
{
    fn to_group(&self) -> Vec<i64> {
        self.points
            .iter()
            .map(|p| p.to_scalar())
            .collect::<Vec<_>>()
    }
}

impl<X, Y> Plot<X, Y>
where
    X: ScalarRange,
    Y: ScalarGroup,
{
    fn to_range_group_drawer(&self) -> RangeGroupDrawer {
        RangeGroupDrawer {
            points: self.data.iter()
            .map(|(x, y)| (x.to_range(), y.to_group()))
            .collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug)]
struct RangeGroupDrawer {
    points: Vec<((i64, i64), Vec<i64>)>,
}

use svg::{
    Document,
    node::element::{Path, Line, path::Data},
};

impl RangeGroupDrawer {
    fn render(&self, file: &str) {
        let (xmin, ymin, width, height) = {
            let mut xmin = i64::MAX;
            let mut ymin = i64::MAX;
            let mut xmax = i64::MIN;
            let mut ymax = i64::MIN;
            for ((start, end), points) in &self.points {
                xmin = xmin.min(*start).min(*end);
                xmax = xmax.max(*start).max(*end);
                for pt in points {
                    ymin = ymin.min(*pt);
                    ymax = ymax.max(*pt);
                }
            }
            (xmin, ymin, xmax - xmin, ymax - ymin)
        };
        let fheight = 700.0;
        let fwidth = 1000.0;
        let stroke_width = 2.0;
        let margin = 20.0;
        let resize_x = |x| {
            (x - xmin) as f64 / width as f64 * fwidth
        };
        let resize_y = |y| {
            (height - (y - ymin)) as f64 / height as f64 * fheight
        };
        let mut groups = Vec::new();
        let group_size = self.points[0].1.len();
        for i in 0..group_size-1 {
            groups.push(Data::new().move_to((resize_x(self.points[0].0.0), resize_y(self.points[0].1[i]))));
        }
        let groups = self.points.iter()
            .fold(groups, |gr, ((start, end), points)| {
                gr.into_iter()
                    .enumerate()
                    .map(|(i, gr)| gr.line_to((resize_x(*start), resize_y(points[i])))
                                .line_to((resize_x(*end), resize_y(points[i])))
                    )
                    .collect::<Vec<_>>()
            });
        let groups = self.points.iter().rev()
            .fold(groups, |gr, ((start, end), points)| {
                gr.into_iter()
                    .enumerate()
                    .map(|(i, gr)| gr.line_to((resize_x(*end), resize_y(points[i+1])))
                        .line_to((resize_x(*start), resize_y(points[i+1])))
                        )
                    .collect::<Vec<_>>()
            });
        let paths = groups.into_iter()
            .enumerate()
            .map(|(i, gr)| Path::new()
                .set("fill", COLORS[i])
                .set("d", gr.close()));
        let yaxis = Line::new()
            .set("x1", 0.0)
            .set("x2", 0.0)
            .set("y1", 0.0)
            .set("y2", fheight)
            .set("stroke", "black")
            .set("stroke-width", stroke_width);
        let xaxis = Line::new()
            .set("x1", 0.0)
            .set("x2", fwidth)
            .set("y1", resize_y(0))
            .set("y2", resize_y(0))
            .set("stroke", "black")
            .set("stroke-width", stroke_width);
        let document = paths.into_iter()
            .fold(Document::new(), |doc, path| {
                doc.add(path)
            })
            .add(yaxis)
            .add(xaxis)
            .set("viewBox", (-margin, -margin, fwidth + 2.0 * margin, fheight + 2.0 * margin));
        svg::save(file, &document).unwrap();
    }
}

const COLORS: &[&str] = &[
    "red",
    "green",
    "blue",
    "yellow",
    "orange",
    "purple",
    "cyan",
];
