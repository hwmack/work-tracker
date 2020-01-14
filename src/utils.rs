use chrono::prelude::*;
use chrono::{Duration, DateTime, Date, Local, Weekday};
use colored::Colorize;

pub fn get_week(week: i64, absolute: bool) -> Date<Local> {
    // Find the week specified by the int
    let mut week_day = if absolute {
        // Get the start of the year + the week specified
        Local.ymd(Local::now().year(), 1, 1) + Duration::weeks(week)
    } else {
        // Take the weeks from the current time
        (Local::now() - Duration::weeks(week)).date()
    };

    // Find the Sunday of the week
    while week_day.weekday() != Weekday::Sun {
        week_day = week_day.pred();
    }

    week_day
}

pub fn format_time(dur: Duration) -> String {
    let h = dur.num_hours();
    let m = dur.num_minutes() - (h * 60);
    let s = dur.num_seconds() - (m * 60) - (h * 60 * 60);

    let hours = if h < 10 {
        format!("0{}", h)
    } else {
        format!("{}", h)
    };

    let minutes = if m < 10 {
        format!("0{}", m)
    } else {
        format!("{}", m)
    };

    let seconds = if s < 10 {
        format!("0{}", s)
    } else {
        format!("{}", s)
    };

    format!("{}:{}:{}", hours.green(), minutes.cyan(), seconds.magenta())
}

///Format date time so it doesn't need to be done everywhere
pub fn format_date_time(date: DateTime<Local>) -> String {
    date.format("%a, %e %B %Y %T").to_string()
}

pub fn format_date(date: Date<Local>) -> String {
    date.format("%-e %b %Y").to_string()
}