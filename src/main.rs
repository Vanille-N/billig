mod cli;
mod lib;
mod load;

use cli::{plot::Plotter, table::Table};
use lib::{
    date::{Date, Duration, Month, Period},
    summary::Calendar,
};

fn main() {
    let filename = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../expenses.bil".to_string());

    let (entries, errs, mut period) = load::read_entries(&filename);
    println!("{}", errs);
    if let Some(lst) = entries {
        period.intersect(Period(
            Date::from(2020, Month::Sep, 1).unwrap(),
            Date::from(2021, Month::Sep, 1).unwrap(),
        ));
        let mut cal_day = Calendar::from_spacing(period, Duration::Day, 1);
        let mut cal_week = Calendar::from_spacing(period, Duration::Week, 1);
        let mut cal_month = Calendar::from_spacing(period, Duration::Month, 1);
        let mut cal_year = Calendar::from_spacing(period, Duration::Year, 1);
        cal_day.register(&lst);
        cal_week.register(&lst);
        cal_month.register(&lst);
        cal_year.register(&lst);
        let table_week = Table::from(cal_week.contents()).with_title("Weekly");
        let table_month = Table::from(cal_month.contents()).with_title("Monthly");
        let table_year = Table::from(cal_year.contents()).with_title("Yearly");
        println!("{}", table_week);
        println!("{}", table_month);
        println!("{}", table_year);
        Plotter::from(cal_day.contents()).print_cumulative_plot();
    }
}
