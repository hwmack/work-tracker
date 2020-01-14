use colored::Colorize;

use crate::format::*;

///Temporarily pause tracking time
///
///Updates the state to paused and sets the end of the current time
pub fn pause(data: &mut FileFormat) {
    if data.state == TrackingState::Stopped {
        eprintln!("{}", "Not tracking any time to pause".red());
        return;
    }

    update_last_block(&mut data.times);

    data.state = TrackingState::Paused;
}
