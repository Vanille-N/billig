use crate::lib::{
    date::Date,
    entry::Amount,
    period::{Between, Minimax},
    summary::Summary,
};

/// In charge of the public interface to the plotting devices
pub struct Plotter<'d> {
    data: &'d [Summary],
}

/// Recommended usage:
/// ```
/// let mut cal: Calendar = unimplemented!();
/// let lst: Vec<entry> = unimplemented!();
/// cal.register(&lst);
/// Plotter::from(cal.contents()).print_cumulative_plot()
/// ```
impl<'d> Plotter<'d> {
    /// Wrap data to plot
    pub fn from(data: &'d [Summary]) -> Self {
        Self { data }
    }

    /// Launch plotting
    pub fn print_cumulative_plot(&self, title: &str) {
        self.cumulative_plot()
            .to_range_group_drawer()
            .render(&format!("{}.svg", title))
    }

    /// Accumulate contained data into cumulative plot
    fn cumulative_plot(&self) -> Plot<Between<Date>, CumulativeEntry<Amount>> {
        let mut plot = Plot::new();
        for sum in self.data {
            plot.push(sum.period(), CumulativeEntry::cumul(sum.amounts().to_vec()));
        }
        plot
    }
}

/// Holds data for bounds of data to graduate
pub struct Grads<T> {
    lower: T,
    upper: T,
}

impl<T> Grads<T>
where
    T: Minimax,
{
    /// Default impl: empty interval
    fn new() -> Self {
        Self {
            lower: T::MAX,
            upper: T::MIN,
        }
    }
}

impl<T> Grads<T>
where
    T: Ord + Copy,
{
    /// Add `data` to the current interval
    fn extend(&mut self, data: T) {
        self.lower = self.lower.min(data);
        self.upper = self.upper.max(data);
    }
}

impl<T> Grads<T>
where
    T: ToString + Scalar + Hierarchical,
{
    fn into_grads(self) -> Vec<(i64, String)> {
        T::hierarchy(self.lower, self.upper)
            .into_iter()
            .map(|x| (x.to_scalar(), x.to_string()))
            .collect::<Vec<_>>()
    }
}

pub trait GradExtend {
    type Item;
    fn extend(&self, grads: &mut Grads<Self::Item>);
}

pub trait Hierarchical: Sized {
    fn hierarchy(lo: Self, hi: Self) -> Vec<Self> {
        vec![lo, hi]
    }
}

impl Hierarchical for Amount {
    fn hierarchy(lo: Self, hi: Self) -> Vec<Self> {
        // calculate step for ~target graduations
        let step = {
            let mut step = 1;
            let diff = (hi.0 - lo.0).abs();
            let target = 7;
            while diff / step > target {
                if diff / step / 10 > target {
                    step *= 10;
                } else if diff / step / 5 > target {
                    step *= 5;
                } else if diff / step / 2 > target {
                    step *= 2;
                } else {
                    break;
                }
            }
            step
        };
        let mut v = Vec::new();
        let mut curr = 0;
        // find lower bound
        while curr < lo.0 {
            curr += step;
        }
        while curr >= lo.0 {
            curr -= step;
        }
        curr += step;
        // step to upper bound
        while curr <= hi.0 {
            v.push(Amount(curr));
            curr += step;
        }
        v
    }
}

impl Hierarchical for Date {
    fn hierarchy(lo: Self, hi: Self) -> Vec<Self> {
        let diff = hi.index() - lo.index();
        let step = {
            let mut step = 1;
            let target = 10;
            while diff / step > 10 {
                step += 1;
            }
            step as isize
        };
        let mut curr = lo;
        let mut v = Vec::new();
        while curr < hi {
            v.push(curr);
            curr = curr.jump_day(step);
        }
        v
    }
}

/// Generic plotter
#[derive(Debug)]
pub struct Plot<X, Y> {
    /// (X, Y) generic descriptor of how to display the data
    data: Vec<(X, Y)>,
}

impl<X, Y> Plot<X, Y> {
    /// Empty plotter
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Add item
    fn push(&mut self, x: X, y: Y) {
        self.data.push((x, y));
    }
}

/// Describes how to format a collection of same-abscissa points
#[derive(Debug)]
struct CumulativeEntry<Y> {
    points: Vec<Y>,
}

impl<Y> CumulativeEntry<Y>
where
    Y: std::ops::AddAssign + Clone,
{
    /// Calculate cumulative values
    fn cumul(mut points: Vec<Y>) -> Self {
        for i in 1..points.len() {
            let prev = points[i - 1].clone();
            points[i] += prev;
        }
        Self { points }
    }
}

/// A plot item that can be converted to a value
/// (e.g. an amount or a date)
pub trait Scalar {
    fn to_scalar(&self) -> i64;
}

/// A plot item that can be converted to a pair of values
/// (e.g. a period)
pub trait ScalarRange {
    fn to_range(&self) -> (i64, i64);
}

/// A plot item that can be converted to a group of values
/// (e.g. a sequence of cumulative entries)
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

impl<T> ScalarRange for Between<T>
where
    T: Scalar,
{
    fn to_range(&self) -> (i64, i64) {
        (self.0.to_scalar(), self.1.to_scalar())
    }
}

impl<T> GradExtend for Between<T>
where
    T: Ord + Copy,
{
    type Item = T;
    fn extend(&self, grad: &mut Grads<Self::Item>) {
        grad.extend(self.0);
        grad.extend(self.1);
    }
}

impl<T> ScalarRange for (T, T)
where
    T: Scalar,
{
    fn to_range(&self) -> (i64, i64) {
        (self.0.to_scalar(), self.1.to_scalar())
    }
}

impl<T> GradExtend for (T, T)
where
    T: Ord + Copy,
{
    type Item = T;
    fn extend(&self, grad: &mut Grads<Self::Item>) {
        grad.extend(self.0);
        grad.extend(self.1);
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

impl<Y> GradExtend for CumulativeEntry<Y>
where
    Y: Ord + Copy,
{
    type Item = Y;
    fn extend(&self, grad: &mut Grads<Self::Item>) {
        for item in &self.points {
            grad.extend(*item);
        }
    }
}

impl<X, Y> Plot<X, Y>
where
    X: ScalarRange + GradExtend,
    Y: ScalarGroup + GradExtend,
    <X as GradExtend>::Item: ToString + Scalar + Minimax + Hierarchical,
    <Y as GradExtend>::Item: ToString + Scalar + Minimax + Hierarchical,
{
    fn to_range_group_drawer(&self) -> RangeGroupDrawer {
        let mut points = Vec::new();
        let mut grad_x = Grads::new();
        let mut grad_y = Grads::new();
        for (x, y) in &self.data {
            x.extend(&mut grad_x);
            y.extend(&mut grad_y);
            let x = x.to_range();
            let y = y.to_group();
            points.push((x, y));
        }
        RangeGroupDrawer {
            points,
            grad_x: grad_x.into_grads(),
            grad_y: grad_y.into_grads(),
        }
    }
}

struct Dimensions {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
    delta_x: i64,
    delta_y: i64,
    view_height: f64,
    view_width: f64,
    stroke_width: f64,
    margin: f64,
    atomic_width: f64,
}

impl Dimensions {
    fn new() -> Self {
        Self {
            min_x: i64::MAX,
            min_y: i64::MAX,
            max_x: i64::MIN,
            max_y: i64::MIN,
            delta_y: 0,
            delta_x: 0,
            stroke_width: 2.0,
            margin: 20.0,
            view_height: 700.0,
            view_width: 1000.0,
            atomic_width: 0.0,
        }
        .update()
    }

    fn update(mut self) -> Self {
        self.delta_y = self.max_y.saturating_sub(self.min_y);
        self.delta_x = self.max_x.saturating_sub(self.min_x);
        self.atomic_width = 0.95 / self.delta_x.max(1) as f64 * self.view_width;
        self
    }

    fn with_data<'iter, Points, XSeq, YSeq>(mut self, data: Points) -> Self
    where
        Points: IntoIterator<Item = (XSeq, YSeq)>,
        XSeq: IntoIterator<Item = &'iter i64>,
        YSeq: IntoIterator<Item = &'iter i64>,
    {
        for (xs, ys) in data {
            for x in xs {
                self.max_x = self.max_x.max(*x);
                self.min_x = self.min_x.min(*x);
            }
            for y in ys {
                self.max_y = self.max_y.max(*y);
                self.min_y = self.min_y.min(*y);
            }
        }
        self.update()
    }

    fn resize_x(&self, x: i64) -> f64 {
        (x - self.min_x) as f64 / self.delta_x as f64 * self.view_width
    }

    fn resize_y(&self, y: i64) -> f64 {
        (self.max_y - y) as f64 / self.delta_y as f64 * self.view_height
    }
}

#[derive(Debug)]
struct RangeGroupDrawer {
    points: Vec<((i64, i64), Vec<i64>)>,
    grad_x: Vec<(i64, String)>,
    grad_y: Vec<(i64, String)>,
}

use svg::{
    node::element::{path::Data, Line, Path, Text},
    node,
    Document,
};

impl RangeGroupDrawer {
    fn render(&self, file: &str) {
        // configure dimensions with extremal values
        let dim = Dimensions::new().with_data(
            self.points
                .iter()
                .map(|((start, end), points)| ([start, end], points)),
        );
        // plot columns one by one
        if self.points.is_empty() {
            return;
        }
        let mut groups = Vec::new();
        let group_size = self.points[0].1.len();
        for i in 0..group_size - 1 {
            groups.push(Data::new().move_to((
                dim.resize_x(self.points[0].0 .0),
                dim.resize_y(self.points[0].1[i]),
            )));
        }
        // add lower data points
        let groups_inorder = self
            .points
            .iter()
            .fold(groups, |gr, ((start, end), points)| {
                gr.into_iter()
                    .enumerate()
                    .map(|(i, gr)| {
                        gr.line_to((dim.resize_x(*start), dim.resize_y(points[i])))
                            .line_to((
                                dim.resize_x(*end) + dim.atomic_width,
                                dim.resize_y(points[i]),
                            ))
                    })
                    .collect::<Vec<_>>()
            });
        // add upper data points
        let groups = self
            .points
            .iter()
            .rev()
            .fold(groups_inorder, |gr, ((start, end), points)| {
                gr.into_iter()
                    .enumerate()
                    .map(|(i, gr)| {
                        gr.line_to((
                            dim.resize_x(*end) + dim.atomic_width,
                            dim.resize_y(points[i + 1]),
                        ))
                        .line_to((dim.resize_x(*start), dim.resize_y(points[i + 1])))
                    })
                    .collect::<Vec<_>>()
            });
        // the two transformations above create
        //
        // (start,i+1) <-------- (end,i+1)
        //    |                     ^
        //    |                     |
        //    v                     |
        // (start,i)   --------> (end,i)
        let paths = groups
            .into_iter()
            .enumerate()
            .map(|(i, gr)| Path::new().set("fill", COLORS[i]).set("d", gr.close()));
        let yaxis = Line::new()
            .set("x1", dim.resize_x(dim.min_x))
            .set("x2", dim.resize_x(dim.min_x))
            .set("y1", dim.resize_y(dim.max_y) - dim.margin / 2.0)
            .set("y2", dim.resize_y(dim.min_y) + dim.margin / 2.0)
            .set("stroke", "black")
            .set("stroke-width", dim.stroke_width);
        let ygrad = self.grad_y.iter().map(|(n, txt)| {
            (
            Line::new()
                .set("x1", dim.resize_x(dim.min_x))
                .set("x2", dim.resize_x(dim.min_x) - dim.margin / 2.0)
                .set("y1", dim.resize_y(*n))
                .set("y2", dim.resize_y(*n))
                .set("stroke", "black")
                .set("stroke-width", dim.stroke_width),
            Text::new()
                .set("x", dim.resize_x(dim.min_x) - dim.margin)
                .set("y", dim.resize_y(*n) + dim.margin / 4.0)
                .set("stroke", "black")
                .set("text-anchor", "end")
                .set("stroke-width", dim.stroke_width)
                .add(node::Text::new(txt))
            )
        });
        let xaxis = Line::new()
            .set("x1", dim.resize_x(dim.min_x))
            .set("x2", dim.resize_x(dim.max_x) + dim.margin / 2.0)
            .set("y1", dim.resize_y(0))
            .set("y2", dim.resize_y(0))
            .set("stroke", "black")
            .set("stroke-width", dim.stroke_width);
        let xgrad = self.grad_x.iter().map(|(n, txt)| {
            let x = dim.resize_x(*n);
            let y = dim.resize_y(0);
            (Line::new()
                .set("x1", x)
                .set("x2", x)
                .set("y1", y)
                .set("y2", y + dim.margin / 2.0)
                .set("stroke", "black")
                .set("stroke-width", dim.stroke_width),
            Text::new()
                .set("transform", format!("rotate(40, {x}, {y}) translate({x} {y}) translate(10 20)", x = x + dim.margin / 2.0, y = y - dim.margin / 2.0))
                .set("stroke", "black")
                .set("stroke-width", dim.stroke_width)
                .add(node::Text::new(txt))
            )
        });
        let document = paths
            .into_iter()
            .fold(Document::new(), |doc, path| doc.add(path));
        let document = ygrad
            .into_iter()
            .chain(xgrad.into_iter())
            .fold(document, |doc, (path, text)| doc.add(path).add(text))
            .add(yaxis)
            .add(xaxis)
            .set(
                "viewBox",
                (
                    -2.0 * dim.margin,
                    -2.0 * dim.margin,
                    dim.view_width + 2.0 * dim.margin,
                    dim.view_height + 4.0 * dim.margin,
                ),
            );
        svg::save(file, &document).unwrap();
    }
}

const COLORS: &[&str] = &["red", "green", "blue", "yellow", "orange", "purple", "cyan"];
