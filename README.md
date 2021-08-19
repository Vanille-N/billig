# Billig

#### A command-line DSL budget manager


# Philosophy

The central feature of Billig is that expenses are meant to be spread over a
relevant period. A budget manager that registers expenses and incomes only
for the day they occur cannot provide accurate insights due to high variability.

Billig aims to remove this limitation by having all transactions be registered
with the timeframe during which they are relevant.

Billig is a rewrite from Python to Rust of another project of mine; feature parity
has been reached since Billig now provides graphical representations and tabulated
summaries. The original project used Python dictionaries to store data, Billig aims
to offer a better experience by using a specifically-designed DSL.


# Usage

*WIP: Billig's command-line interface is subject to change*

Run `billig -h` for help.

```
Complete form:
$ billig --plot day,week --table week,month,year expenses.bil
         ^               ^                       ^--- source file
         |               |
         |               '--- which tables to show: weekly + monthly + yearly
         '--- which plots to show : daily + weekly

Shortened:
$ billig -pd,w -tw,m,y
         ^     ^         ^--- default source is expenses.bil
         |     |
         |     '--- -t short for --table, m for month, y for year
         '--- -p short for --plot, d for day, w for week
```
Tables are printed in the terminal in color, plots are generated as `.svg`



# Syntax

Data is stored in `.bil` files, which are parsed using [pest.rs](https://pest.rs)

The following example gives an overview of the available constructs.

```java
2020:
    Sep:
        01: val -300, type Mov, span Year<Post> 1, tag "Train pass";
            // the above expense will be registered for one year from
            // 2020-Sep-01 to 2021-Sep-01
            val -3.5, type Food, span Day, tag "Sandwich";
            // several entries can be registered for a single day
            // this one will last only a day, Day is the contracted form
            // of Day<Curr> 1
        02: -40, Food, period ..Oct-15, "Misc";
            // labels 'val', 'type', 'span', 'tag' can be omitted
            // the 'period' construct allows for more fine-grained control
            // over timeframes

// this is a template
!food_supplies value { // it takes a single positional argument
    val @Neg *value, // expands to an amount
    type Food,
    span Month<Post>,
    tag @Concat "Food " @Year "-" @Month, // date is passed to the template as
                                          // an implicit argument
}

!restaurant value tip=0 { // tip is an optional argument
    val @Neg @Sum *value *tip, // total value is the sum of the two
    type Food,
    span Day,
    tag @Concat "Restaurant " @Weekday ". " @Date " at " *place,
                                            // this forces place to be 
                                            // passed as a named argument
}

2020:
    Dec:
        15: !food_supplies 69.42;
            // expands to:
            //   val -69.42, type Food, span Month<Post>,
            //   tag "Food 2020-Dec";
        20: !restaurant 30 place="Foo";
            // expands to:
            //   val -30, type Food, span Day,
            //   tag "Restaurant Sun. 2020-Dec-20 at Foo";
        25: !restaurant 50 place="Bar" tip=5;
            // expands to:
            //   val -55, type Food, span Day,
            //   tag "Restaurant Fri. 2020-Dec-25 at Bar";

import ../other.bil
// either relative or absolute path, parses the contents of the
// imported file in the context of the current one: local template definitions
// are available in other.bil, but definitions from other.bil do not pollute the
// current namespace
```
