use colored::Colorize;

use crate::format::*;
use crate::utils;

///Stop tracking time
///
///Sets the current state to stopped
///
///Sums all the times together and inserts it into the past vec
pub fn stop(data: &mut FileFormat, comment: String) {
    if data.state == TrackingState::Stopped {
        eprintln!("{}", "Not tracking any time to stop".red());
        return;
    }

    // Set the final block to the current time
    if data.state != TrackingState::Paused {
        update_last_block(&mut data.times);
    }

    let shift_time = calculate_shift_length(&data.times);
    let time_block = data.times.get(0).expect("There is no time data found");

    data.past.push(PastTimeBlock {
        date: time_block.start, // Set the date to when the shift was started
        seconds: shift_time.num_seconds(),
        comment: comment,
    });

    // Reset the times vec
    data.times = Vec::new();
    data.state = TrackingState::Stopped;

    println!(
        "Finished shift after {}",
        utils::format_time(shift_time).blue()
    );
}
