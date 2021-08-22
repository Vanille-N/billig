use num_traits::FromPrimitive;
use std::fmt;

use crate::lib::{
    date::{Between, Date},
    entry::{Amount, Category},
    summary::Summary,
};

pub struct Table<'d> {
    title: String,
    data: &'d [Summary],
}

struct BoxFmt {
    width: usize,
    text: String,
    color: Option<Color>,
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
        Self {
            title: String::new(),
            data,
        }
    }

    pub fn with_title<S>(mut self, s: S) -> Self
    where
        S: ToString,
    {
        self.title = s.to_string();
        self
    }

    fn to_formatter(&self) -> GridFmt {
        let columns = (0..Category::COUNT)
            .map(|i| Category::from_usize(i).unwrap())
            .collect::<Vec<_>>();
        let cols = columns
            .iter()
            .map(|c| BoxFmt::category(*c))
            .chain(std::iter::once(BoxFmt::from("Total")))
            .map(ColFmt::with_label)
            .collect::<Vec<_>>();
        let mut shaders = (0..Category::COUNT)
            .map(|_| Statistics::new())
            .collect::<Vec<_>>();
        let mut shader_total = Statistics::new();
        for sum in self.data {
            for (i, data) in sum.amounts().iter().enumerate() {
                shaders[i].register(data.0 as f64);
            }
            shader_total.register(sum.total().0 as f64);
        }
        let shaders = shaders
            .into_iter()
            .map(Statistics::make_shader)
            .collect::<Vec<_>>();
        let shader_total = shader_total.make_shader();
        let mut grid = GridFmt::with_columns(BoxFmt::from(&self.title), cols);
        for sum in self.data {
            grid.push_line(
                BoxFmt::period(sum.period()),
                sum.amounts()
                    .iter()
                    .enumerate()
                    .map(|(i, f)| BoxFmt::amount(*f).with_shade(shaders[i].generate(f.0 as f64)))
                    .chain(std::iter::once(
                        BoxFmt::amount(sum.total())
                            .with_shade(shader_total.generate(sum.total().0 as f64)),
                    ))
                    .collect::<Vec<_>>(),
            );
        }
        grid
    }
}

impl BoxFmt {
    fn from<S>(text: S) -> Self
    where
        S: ToString,
    {
        let text = text.to_string();
        let width = text.len();
        Self {
            text,
            width,
            color: None,
        }
    }

    fn amount(a: Amount) -> Self {
        if a != Amount(0) {
            let text = format!("{}", a);
            let width = text.len() - 2;
            Self {
                text,
                width,
                color: None,
            }
        } else {
            Self::from(String::new())
        }
    }

    fn period(p: Between<Date>) -> Self {
        Self::from(format!("{}", p))
    }

    fn category(c: Category) -> Self {
        Self::from(format!("{:?}", c))
    }

    fn with_shade(mut self, shade: Color) -> Self {
        self.color = Some(shade);
        self
    }
}

impl ColFmt {
    fn with_label(label: BoxFmt) -> Self {
        Self {
            width: label.width,
            label,
            boxes: Vec::new(),
        }
    }

    fn push(&mut self, b: BoxFmt) {
        self.width = self.width.max(b.width);
        self.boxes.push(b);
    }
}

impl GridFmt {
    fn with_columns(title: BoxFmt, columns: Vec<ColFmt>) -> Self {
        Self {
            labels: ColFmt::with_label(title),
            columns,
        }
    }

    fn push_line(&mut self, label: BoxFmt, boxes: Vec<BoxFmt>) {
        self.labels.push(label);
        for (i, b) in boxes.into_iter().enumerate() {
            self.columns[i].push(b);
        }
    }
}

impl fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_formatter())
    }
}

impl fmt::Display for GridFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // upper border
        write!(f, "{}", ULCORNER)?;
        self.labels.hline(f)?;
        for c in &self.columns {
            write!(f, "{}", LOJOIN)?;
            c.hline(f)?;
        }
        writeln!(f, "{}", URCORNER)?;
        // title line
        write!(f, "{}", VLINE)?;
        self.labels.write_label(f, false)?;
        for c in &self.columns {
            write!(f, "{}", VLINE)?;
            c.write_label(f, true)?;
        }
        writeln!(f, "{}", VLINE)?;
        // separator
        write!(f, "{}", RTJOIN)?;
        self.labels.hline(f)?;
        for c in &self.columns {
            write!(f, "{}", CROSS)?;
            c.hline(f)?;
        }
        writeln!(f, "{}", LTJOIN)?;

        // main block
        for idx in 0..self.labels.len() {
            write!(f, "{}", VLINE)?;
            self.labels.write_item(f, idx, false)?;
            for c in &self.columns {
                write!(f, "{}", VLINE)?;
                c.write_item(f, idx, true)?;
            }
            writeln!(f, "{}", VLINE)?;
        }
        // lower border
        write!(f, "{}", DLCORNER)?;
        self.labels.hline(f)?;
        for c in &self.columns {
            write!(f, "{}", HIJOIN)?;
            c.hline(f)?;
        }
        writeln!(f, "{}", DRCORNER)?;
        Ok(())
    }
}

impl ColFmt {
    fn write_label(&self, f: &mut fmt::Formatter, right: bool) -> fmt::Result {
        self.label.write(f, self.width, right)
    }

    fn write_item(&self, f: &mut fmt::Formatter, idx: usize, right: bool) -> fmt::Result {
        self.boxes[idx].write(f, self.width, right)
    }

    fn len(&self) -> usize {
        self.boxes.len()
    }

    fn hline(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &HLINE[..(self.width + 2 + MARGIN) * 3])
    }
}

const PADDING: &str = "                                         ";
const HLINE: &str = "─────────────────────────────────────────";
const VLINE: &str = "│";
const ULCORNER: &str = "┌";
const URCORNER: &str = "┐";
const DLCORNER: &str = "└";
const DRCORNER: &str = "┘";
const LTJOIN: &str = "┤";
const RTJOIN: &str = "├";
const HIJOIN: &str = "┴";
const LOJOIN: &str = "┬";
const CROSS: &str = "┼";
const MARGIN: usize = 1;
impl BoxFmt {
    fn write(&self, f: &mut fmt::Formatter, width: usize, right: bool) -> fmt::Result {
        if let Some(c) = self.color {
            write!(f, "{}", c)?;
        }
        if right {
            write!(
                f,
                " {}{} ",
                &PADDING[..(width + MARGIN).saturating_sub(self.width)],
                self.text,
            )?;
        } else {
            write!(
                f,
                " {}{} ",
                self.text,
                &PADDING[..(width + MARGIN).saturating_sub(self.width)],
            )?;
        }
        write!(f, "{}", Color::BLANK)
    }
}

#[derive(Default)]
pub struct Statistics {
    positive: Vec<f64>,
    negative: Vec<f64>,
}

impl Statistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, data: f64) {
        if data >= 0.0 {
            self.positive.push(data);
        } else {
            self.negative.push(data);
        }
    }

    pub fn make_shader(mut self) -> Shader {
        let make_deciles = |v: &mut Vec<f64>, reverse: bool| {
            v.sort_by(|a, b| {
                if reverse {
                    a.partial_cmp(b)
                } else {
                    b.partial_cmp(a)
                }
                .unwrap_or(std::cmp::Ordering::Less)
            });
            (0..=10)
                .map(|i| *v.get(v.len().saturating_sub(1) * i / 10).unwrap_or(&0.0))
                .collect::<Vec<_>>()
        };
        Shader::with_steps(
            make_deciles(&mut self.negative, true),
            make_deciles(&mut self.positive, false),
        )
    }
}

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);

pub struct Shader {
    positive: Vec<(f64, Color)>,
    negative: Vec<(f64, Color)>,
}

impl Shader {
    const RED_YLW: &'static [Color] = &[
        Color(255, 0, 0),
        Color(255, 18, 0),
        Color(255, 37, 0),
        Color(255, 55, 0),
        Color(255, 74, 0),
        Color(255, 92, 0),
        Color(255, 110, 0),
        Color(255, 129, 0),
        Color(255, 147, 0),
        Color(255, 166, 0),
        Color(255, 184, 0),
        Color(255, 203, 0),
        Color(255, 221, 0),
    ];

    const GRN_BLU: &'static [Color] = &[
        Color(0, 255, 255),
        Color(0, 255, 213),
        Color(0, 255, 170),
        Color(0, 255, 128),
        Color(0, 255, 85),
        Color(0, 255, 43),
        Color(0, 255, 0),
        Color(43, 255, 0),
        Color(85, 255, 0),
        Color(128, 255, 0),
        Color(170, 255, 0),
        Color(213, 255, 0),
        Color(255, 255, 0),
    ];

    fn with_steps(steps_neg: Vec<f64>, steps_pos: Vec<f64>) -> Self {
        let make_steps = |v: Vec<f64>, shades: &[Color]| {
            let nb = v.len();
            let max = shades.len();
            let indexer = |i| (max - 1) * i / (nb - 1);
            let mut arr = v
                .into_iter()
                .enumerate()
                .map(|(i, f)| (f, shades[indexer(i)]))
                .collect::<Vec<_>>();
            let delta = arr.get(0).map(|(f, _)| *f).unwrap_or(0.0)
                - arr.last().map(|(f, _)| *f).unwrap_or(0.0);
            if let Some(f) = arr.get_mut(0) {
                f.0 += delta;
            }
            if let Some(f) = arr.last_mut() {
                f.0 -= delta;
            }
            arr
        };
        Self {
            positive: make_steps(steps_pos, Self::GRN_BLU),
            negative: make_steps(steps_neg, Self::RED_YLW),
        }
    }

    pub fn generate(&self, data: f64) -> Color {
        let chooser = if data >= 0.0 {
            &self.positive
        } else {
            &self.negative
        };
        let contains = |b, (lo, hi)| (lo < b && b <= hi) || (hi < b && b <= lo);
        for w in chooser.windows(2) {
            if contains(data, (w[0].0, w[1].0)) {
                return w[0].1;
            }
        }
        chooser[0].1
    }
}

impl Color {
    pub const BLANK: &'static str = "\x1b[0m";
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1b[38;2;{};{};{}m", self.0, self.1, self.2)
    }
}
