use colored::Colorize;

use crate::format::*;
use crate::utils::format_time;

///Check the current status
pub fn status(data: &mut FileFormat) {
    println!("status: {}", data.state);

    // Display the current shift length if not stopped
    if data.state != TrackingState::Stopped {
        // If we are tracking time, we can safely update the block
        // without destroying data
        if data.state == TrackingState::Tracking {
            update_last_block(&mut data.times);
        }

        let shift_time = calculate_shift_length(&data.times);
        println!("timer: {}", format_time(shift_time).blue());
    }
}
