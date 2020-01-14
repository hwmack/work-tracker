use std::io::{stdin, stdout, Write};

use chrono::prelude::*;
use chrono::Duration;
use colored::Colorize;

use crate::format::*;
use crate::utils::*;

///Edit some dates in the file
///Should not be used but provided because accidents happen
pub fn edit(week: i64, absolute: bool, data: &mut FileFormat) {
    // Find the week that is specified as an argument
    let mut week = get_week(week, absolute);
    let (index, data_index) = {
        let mut shifts = Vec::new();

        // Get every shift for this week
        loop {
            for (index, past_block) in data.past.iter().enumerate() {
                let parsed_date: DateTime<Local> = DateTime::from(past_block.date);
                if parsed_date.date() == week {
                    shifts.push((index, past_block));
                }
            }

            week = week.succ();
            if week.weekday() == Weekday::Sun {
                break;
            }
        }

        for (index, block) in shifts.iter().enumerate() {
            println!(
                "{}: {}\n   duration={}\n   comment={}\n",
                index,
                format_date_time(DateTime::from(block.1.date)).blue(),
                format_time(Duration::seconds(block.1.seconds)),
                block.1.comment.magenta()
            );
        }

        let shift_len = shifts.len() as i32;
        let index: Option<i32> =
            get_user_input("Which would you like to edit or exit(e)", &|value| {
                if value == "e" {
                    return Some(None);
                }
                let index = value.parse().unwrap_or(-1);
                if index >= 0 && index < shift_len {
                    Some(Some(index))
                } else {
                    None
                }
            });

        if let Some(index) = index {
            (index, shifts[index as usize].0)
        } else {
            return;
        }
    };

    println!("Editing {}", index);

    loop {
        let choice = get_user_input(
            "Would you like to edit duration(e), comment(c), delete(d), or exit(x)?",
            &|value| {
                if value == "e" || value == "c" || value == "d" || value == "x" {
                    Some(String::from(value))
                } else {
                    None
                }
            },
        );

        match choice.as_ref() {
            "e" => {
                data.past[data_index].seconds = get_user_input("new hours:", &|value| {
                    if let Ok(value) = value.trim().parse::<i64>() {
                        Some(value * 60 * 60)
                    } else {
                        None
                    }
                }) + get_user_input("new minutes:", &|value| {
                    if let Ok(value) = value.trim().parse::<i64>() {
                        Some(value * 60)
                    } else {
                        None
                    }
                }) + get_user_input("new seconds:", &|value| {
                    if let Ok(value) = value.trim().parse::<i64>() {
                        Some(value)
                    } else {
                        None
                    }
                });
            }
            "c" => {
                data.past[data_index].comment =
                    get_user_input("new comment:", &|value| Some(String::from(value)));
            }
            "d" => {
                println!("{}", "deleting shift..".red());
                data.past.remove(data_index);
                break;
            }
            "x" => break,
            _ => continue,
        }
    }
}

fn get_user_input<T>(output: &str, is_valid: &dyn Fn(&str) -> Option<T>) -> T {
    let mut input = String::new();
    loop {
        // Remove any character in the heap
        input.clear();

        // Write and flush stdout
        print!("{} ", output);
        stdout().flush().unwrap();

        // Read line from stdin
        stdin()
            .read_line(&mut input)
            .expect("Could not read user input");

        // Remove extra characters in input
        if let Some('\n') = input.chars().next_back() {
            input.pop();
        }
        if let Some('\r') = input.chars().next_back() {
            input.pop();
        }

        // Check if it is valid
        if let Some(value) = is_valid(&input) {
            return value;
        }
    }
}
