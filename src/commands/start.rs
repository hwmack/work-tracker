use chrono::{DateTime, Local, Utc};
use colored::Colorize;

use crate::format;
use crate::utils;

///Start tracking time for this shift
///
///Puts a new start time into the structure and set the current state
///to tracking
///
///NOTE: If the state is current resumed, it will continue
pub fn start(data: &mut format::FileFormat) {
    if data.state == format::TrackingState::Tracking {
        eprintln!("{}", "Already tracking time".red());
        return;
    }

    let current_block = format::TimeBlock {
        start: Utc::now(),
        end: Utc::now(),
        finished_tasks: vec![],
    };

    let local: DateTime<Local> = DateTime::from(current_block.start);
    println!("Started timer at {}", utils::format_date_time(local).blue());

    data.times.push(current_block);

    // Lastly set the state to tracking
    data.state = format::TrackingState::Tracking;
}
