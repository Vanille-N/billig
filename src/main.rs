mod cli;
mod lib;
mod load;

use cli::{plot::Plotter, table::Table};
use lib::{
    date::{Date, Duration, Month, Interval},
    summary::Calendar,
};
use std::collections::{BTreeSet, HashMap};

use clap::{App, Arg};

fn main() {
    let matches = App::new("Billig")
        .version("0.2")
        .author("Vanille N. <neven.villani@gmail.com>")
        .about("Command-line DSL-powered budget manager")
        .arg(
            Arg::with_name("source")
                .default_value("expenses.bil")
                .value_name("FILE")
                .help("Source file"),
        )
        .arg(
            Arg::with_name("table")
                .short("t")
                .long("table")
                .value_name("TABLE,...")
                .help("Choose tables to print (day, week, month, year)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("plot")
                .short("p")
                .long("plot")
                .value_name("PLOT,...")
                .help("Choose plots to print (day, week, month, year)")
                .takes_value(true),
        )
        .get_matches();
    let filename = matches.value_of("source").unwrap();
    let (entries, errs, mut timeframe) = load::read_entries(&filename);
    println!("{}", errs);
    if let Some(lst) = entries {
        timeframe = timeframe.intersect(Interval::Between(
            Date::from(2020, Month::Sep, 1).unwrap(),
            Date::from(2022, Month::Jan, 1).unwrap(),
        ));
        let tables = durations(&matches, "table");
        let plots = durations(&matches, "plot");
        dbg!(&tables, &plots);
        let mut calendars: HashMap<Duration, Calendar> = tables
            .union(&plots)
            .map(|&k| (k, Calendar::from_spacing(timeframe.as_between(), k, 1)))
            .collect();
        for (_, cal) in calendars.iter_mut() {
            cal.register(&lst);
        }
        for t in tables {
            let tbl = Table::from(calendars[&t].contents()).with_title(t.text_frequency());
            println!("{}", tbl);
        }
        for p in plots {
            Plotter::from(calendars[&p].contents()).print_cumulative_plot(p.text_frequency());
        }
    }
}

fn durations(matches: &clap::ArgMatches, label: &str) -> BTreeSet<Duration> {
    if let Some(s) = matches.value_of(label) {
        s.split(',')
            .filter_map(|v| {
                Some(match v {
                    "day" | "d" => Duration::Day,
                    "week" | "w" => Duration::Week,
                    "month" | "m" => Duration::Month,
                    "year" | "y" => Duration::Year,
                    other => {
                        eprintln!("'{}' is not a valid duration", other);
                        eprintln!("Expected one of 'day','week','month','year' or 'd','w','m','y'");
                        return None;
                    }
                })
            })
            .collect()
    } else {
        BTreeSet::new()
    }
}
