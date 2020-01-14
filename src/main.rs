use structopt::StructOpt;

mod utils;
mod format;
mod commands;

// Include functions inside the commands directory
use commands::start::*;
use commands::pause::*;
use commands::stop::*;
use commands::status::*;
use commands::display::*;
use commands::edit::*;

///Arguments for the command line application
#[derive(Debug, StructOpt)]
#[structopt(name = "work", about = "Track time spent working")]
enum Args {
    #[structopt(name = "start", about = "Start/Resume tracking time")]
    Start,

    #[structopt(name = "stop", about = "Stop tracking, and finish the current shift")]
    Stop {
        #[structopt(default_value = "")]
        comment: String
    },

    #[structopt(name = "pause", about = "Pause tracking")]
    Pause,

    #[structopt(name = "status", about = "Display the current state")]
    Status,

    #[structopt(name = "display", about = "Display the hours for the current week")]
    Display {
        #[structopt(default_value = "0")]
        week: i64,

        #[structopt(short, long)]
        absolute: bool,
    },

    #[structopt(name = "edit", about = "Edit raw time details if needed")]
    Edit {
        #[structopt(default_value = "0")]
        week: i64,

        #[structopt(short, long)]
        absolute: bool,
    },
}

///Main entry point of the cli
fn main() {
    let opts = Args::from_args();
    let mut file = format::open_file();

    // After opening the file, we need to read it into the FileFormat struct
    let mut data = format::read_file(&mut file);

    // Execute the method related to each argument
    match opts {
        Args::Start => start(&mut data),
        Args::Stop { comment } => stop(&mut data, comment),
        Args::Pause => pause(&mut data),
        Args::Status => status(&mut data),
        Args::Display { week, absolute } => display(week, absolute, &mut data),
        Args::Edit { week, absolute } => edit(week, absolute, &mut data),
    }

    // After we manipulate the data structure -> write it back to the file
    format::write_file(file, data);
}

