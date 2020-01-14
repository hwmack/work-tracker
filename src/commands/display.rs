use chrono::prelude::*;
use chrono::{DateTime, Duration, Local, Weekday};
use colored::Colorize;

use crate::format::*;
use crate::utils::*;

///Displays the hours for a given week
///
///Valid week values are:
/// 0: current week
/// 1-52: absolute week of the current year
/// -(1-52): relative week of the current week
pub fn display(week: i64, absolute: bool, data: &mut FileFormat) {
    // find the week specified
    let mut week = get_week(week, absolute);

    println!("Week starting: {}", format_date(week).blue());
    println!();

    // Attempt to find data for each day of the week
    // Stopping once the next day will be the next Sunday
    loop {
        let mut shifts = Vec::new();

        // Find any matching blocks and add to shifts
        for past_block in &data.past {
            let parsed_date: DateTime<Local> = DateTime::from(past_block.date);
            if parsed_date.date() == week {
                shifts.push(past_block);
            }
        }

        // Get the total hours for the day (if there are multiple shifts)
        let sum = shifts.iter().fold(Duration::seconds(0), |acc, shift| {
            acc + Duration::seconds(shift.seconds)
        });

        // Display the results of the search
        println!("{}:", week.format("%A - %F"));

        if shifts.len() == 0 {
            println!("  {}\n", "No shifts".red());
        } else {
            for shift in shifts {
                println!(
                    "  {}: {}",
                    format_time(Duration::seconds(shift.seconds)).blue(),
                    shift.comment
                );
            }
            println!(
                "{}\n",
                format!("{:2}(total={})", "", format_time(sum).blue())
            );
        }

        week = week.succ();
        if week.weekday() == Weekday::Sun {
            break;
        }
    }
}
