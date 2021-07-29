mod cli;
mod lib;
mod load;

use cli::{plot::Plotter, table::Table};
use lib::{
    date::{Date, Duration, Month, TimeFrame},
    summary::Calendar,
};

fn main() {
    let filename = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "expenses.bil".to_string());

    let (entries, errs, mut timeframe) = load::read_entries(&filename);
    println!("{}", errs);
    if let Some(lst) = entries {
        timeframe = timeframe.intersect(TimeFrame::Between(
            Date::from(2020, Month::Sep, 1).unwrap(),
            Date::from(2021, Month::Sep, 1).unwrap(),
        ));
        println!("{:?}", timeframe);
        let mut cal_day = Calendar::from_spacing(timeframe.as_period(), Duration::Day, 1);
        let mut cal_week = Calendar::from_spacing(timeframe.as_period(), Duration::Week, 1);
        let mut cal_month = Calendar::from_spacing(timeframe.as_period(), Duration::Month, 1);
        let mut cal_year = Calendar::from_spacing(timeframe.as_period(), Duration::Year, 1);
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
