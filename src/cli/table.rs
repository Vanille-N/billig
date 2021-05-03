use num_traits::FromPrimitive;
use std::fmt;

use crate::lib::{
    date::Period,
    entry::{Amount, Category},
    summary::Summary,
};

pub struct Table<'d> {
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
        Self { data }
    }

    fn to_formatter(&self) -> GridFmt {
        let columns = (0..Category::COUNT)
            .map(|i| Category::from_usize(i).unwrap())
            .collect::<Vec<_>>();
        let cols = columns
            .iter()
            .map(|c| BoxFmt::category(*c))
            .chain(std::iter::once(BoxFmt::from(String::from("Total"))))
            .map(|b| ColFmt::with_label(b))
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
        let mut grid = GridFmt::with_columns(cols);
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
    fn from(text: String) -> Self {
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

    fn period(p: Period) -> Self {
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
            width: label.width + 3,
            label,
            boxes: Vec::new(),
        }
    }

    fn push(&mut self, b: BoxFmt) {
        self.width = self.width.max(b.width + 3);
        self.boxes.push(b);
    }
}

impl GridFmt {
    fn with_columns(columns: Vec<ColFmt>) -> Self {
        Self {
            labels: ColFmt::with_label(BoxFmt::from(String::new())),
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
        self.labels.write_label(f)?;
        for c in &self.columns {
            write!(f, "{}", VLINE)?;
            c.write_label(f)?;
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
    fn write_label(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.label.write(f, self.width, true)
    }

    fn write_item(&self, f: &mut fmt::Formatter, idx: usize, right: bool) -> fmt::Result {
        self.boxes[idx].write(f, self.width, right)
    }

    fn len(&self) -> usize {
        self.boxes.len()
    }

    fn hline(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &HLINE[..(self.width + 2) * 3])
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
impl BoxFmt {
    fn write(&self, f: &mut fmt::Formatter, width: usize, right: bool) -> fmt::Result {
        if let Some(c) = self.color {
            write!(f, "{}", c)?;
        }
        if right {
            write!(
                f,
                " {}{} ",
                &PADDING[..width.saturating_sub(self.width)],
                self.text,
            )?;
        } else {
            write!(
                f,
                " {}{} ",
                self.text,
                &PADDING[..width.saturating_sub(self.width)],
            )?;
        }
        write!(f, "{}", Color::BLANK)
    }
}

pub struct Statistics(Vec<f64>);

impl Statistics {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn register(&mut self, data: f64) {
        self.0.push(data);
    }

    pub fn make_shader(mut self) -> Shader {
        self.0
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less));
        let deciles = (0..=10)
            .map(|i| self.0[(self.0.len() - 1) * i / 10])
            .collect::<Vec<_>>();
        Shader::with_steps(deciles)
    }
}

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);

pub struct Shader {
    steps: Vec<(f64, Color)>,
}

impl Shader {
    const STEPS: &'static [Color] = &[
        Color(255, 0, 0),
        Color(255, 43, 0),
        Color(255, 85, 0),
        Color(255, 128, 0),
        Color(255, 170, 0),
        Color(255, 213, 0),
        Color(255, 255, 0),
        Color(213, 255, 0),
        Color(170, 255, 0),
        Color(128, 255, 0),
        Color(85, 255, 0),
        Color(43, 255, 0),
        Color(0, 255, 0),
    ];

    fn with_steps(steps: Vec<f64>) -> Self {
        let nb = steps.len();
        let max = Self::STEPS.len();
        let indexer = |i| (max - 1) * i / (nb - 1);
        Self {
            steps: steps
                .into_iter()
                .enumerate()
                .map(|(i, f)| (f, Self::STEPS[indexer(i)]))
                .collect::<Vec<_>>(),
        }
    }

    pub fn generate(&self, data: f64) -> Color {
        for w in self.steps.windows(2) {
            if w[0].0 < data && data <= w[1].0 {
                return w[0].1;
            }
        }
        self.steps[0].1
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
