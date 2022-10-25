mod cli;
mod util;
mod load;

use cli::{plot::Plotter, table::Table};
use util::{
    date::{Date, Duration, Interval, Month},
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
        .arg(
            Arg::with_name("period")
                .short("D")
                .long("period")
                .value_name("YY-MM-DD..YY-MM-DD")
                .help("Choose range of dates to analyze")
                .takes_value(true),
        )
        .get_matches();
    let mut errs = load::error::Record::new();
    // Get the period right now: we want these errors before we start parsing the file
    let arg_timeframe = match parse_arg_timeframe(&matches, &mut errs) {
        Some(timeframe) => timeframe,
        None => {
            println!("{}", errs);
            return
        }
    };
    let filename = matches.value_of("source").unwrap();
    let (entries, mut timeframe) = load::read_entries(filename, &mut errs);
    println!("{}", errs);
    if let Some(lst) = entries {
        timeframe = timeframe.intersect(arg_timeframe);
        let tables = durations(&matches, "table");
        let plots = durations(&matches, "plot");
        let mut calendars: HashMap<Duration, Calendar> = tables
            .union(&plots)
            .map(|&k| (k, Calendar::from_spacing(timeframe.into_between(), k, 1)))
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

fn parse_arg_timeframe(args: &clap::ArgMatches, errs: &mut load::error::Record) -> Option<Interval<Date>> {
    let value = match args.value_of("period") {
        Some(arg) => arg,
        None => return Some(Interval::Unbounded),
    };
    let pseudo_span = pest::Span::new(value, 0, value.len()).unwrap();
    let pseudo_path = "cmdline";
    let pseudo_loc = &(pseudo_path, pseudo_span);
    let partial_interval = Interval::parse(pseudo_path, errs, value)?;
    let interval = partial_interval.make(errs, pseudo_loc, Date::today())?;
    Some(interval)
}
