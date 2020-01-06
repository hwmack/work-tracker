use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::fmt;

use colored::Colorize;
use chrono::prelude::*;
use chrono::serde::ts_seconds;
use chrono::Duration;
use dirs::home_dir;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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

///Possible time tracking states we could be in
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[repr(u8)]
enum TrackingState {
    Tracking = 1,
    Stopped = 2,
    Paused = 3,
}

///Implement the display trait for the TrackingState enum
impl fmt::Display for TrackingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = match self {
            TrackingState::Tracking => "tracking",
            TrackingState::Stopped => "stopped",
            TrackingState::Paused => "paused"
        };
        write!(f, "{}", result)
    }
}

///Represents a block of time (possibly unfinished)
#[derive(Serialize, Deserialize, Debug)]
struct TimeBlock {
    #[serde(with = "ts_seconds")]
    start: DateTime<Utc>,

    #[serde(with = "ts_seconds")]
    end: DateTime<Utc>,
}

///Where all TimeBlocks are stored after running the `stop` command
#[derive(Serialize, Deserialize, Debug)]
struct PastTimeBlock {
    #[serde(with = "ts_seconds")]
    date: DateTime<Utc>,
    seconds: i64,
    comment: String,
}

///Format of the file to read to and from RMP
#[derive(Serialize, Deserialize, Debug)]
struct FileFormat {
    version: String,
    state: TrackingState,
    times: Vec<TimeBlock>,
    past: Vec<PastTimeBlock>,
}

///Open the file that we will use for writing the timesheets
fn open_file() -> File {
    let home = home_dir().expect("Could not find home directory");
    let path = home.join(".config/work/");

    // Create all of the dirs if they dont exist already
    std::fs::create_dir_all(&path).expect("Failed to create directory");

    let path = path.join("timesheets");
    match OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .append(false)
        .open(&path)
    {
        Ok(file) => file,
        Err(why) => panic!("Could not open file: {}", why),
    }
}

///Reads the file into the file format and returns it
///or creates an empty value if the format is incorrect
fn read_file(file: &mut File) -> FileFormat {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read timesheets file");

    match rmp_serde::from_read_ref(&buffer) {
        Ok(result) => result,
        Err(_) => FileFormat {
            version: env!("CARGO_PKG_VERSION").into(),
            state: TrackingState::Stopped,
            times: Vec::new(),
            past: Vec::new(),
        },
    }
}

///Writes the data to the file
fn write_file(mut file: File, data: FileFormat) {
    let mut buffer = Vec::new();
    data.serialize(&mut Serializer::new(&mut buffer))
        .expect("Could not serialize data");

    // Truncate the file and seek to the start
    file.set_len(0).expect("Could not clear file for writing");
    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");

    let len = file.write(&buffer).expect("Failed to write data to file");

    // Assert expected amount was written to file
    assert!(len == buffer.len())
}

///Start tracking time for this shift
///
///Puts a new start time into the structure and set the current state
///to tracking
///
///NOTE: If the state is current resumed, it will continue
fn start(data: &mut FileFormat) {
    if data.state == TrackingState::Tracking {
        panic!("Already started tracking time");
    }

    let current_block = TimeBlock {
        start: Utc::now(),
        end: Utc::now(),
    };

    println!("Started timer at {}", current_block.start.to_rfc2822().blue());

    data.times.push(current_block);

    // Lastly set the state to tracking
    data.state = TrackingState::Tracking;
}

///Stop tracking time
///
///Sets the current state to stopped
///
///Sums all the times together and inserts it into the past vec
fn stop(data: &mut FileFormat, comment: String) {
    if data.state == TrackingState::Stopped {
        panic!("Not tracking any time to stop");
    }

    // Set the final block to the current time
    if data.state != TrackingState::Paused {
        update_last_block(&mut data.times);
    }

    let shift_time = calculate_shift_length(&data.times);

    data.past.push(PastTimeBlock {
        date: Utc::now(),
        seconds: shift_time.num_seconds(),
        comment: comment,
    });

    // Reset the times vec
    data.times = Vec::new();
    data.state = TrackingState::Stopped;

    println!("Finished shift after {}", format_time(shift_time).blue());
}

///Temporarily pause tracking time
///
///Updates the state to paused and sets the end of the current time
fn pause(data: &mut FileFormat) {
    if data.state == TrackingState::Stopped {
        panic!("Not tracking any time to pause");
    }

    update_last_block(&mut data.times);

    data.state = TrackingState::Paused;
}

///Check the current status
fn status(data: &mut FileFormat) {
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

///Displays the hours for a given week
///
///Valid week values are:
/// 0: current week
/// 1-52: absolute week of the current year
/// -(1-52): relative week of the current week
fn display(week: i64, absolute: bool, data: &mut FileFormat) {
    // find the week specified
    let mut week = get_week(week, absolute);

    println!("Week starting at {})", week);
    println!();

    // Attempt to find data for each day of the week
    // Stopping once the next day will be the next Sunday
    while week.succ().weekday() != Weekday::Sun {
        let mut shifts = Vec::new();

        // Find any matching blocks and add to shifts
        for past_block in &data.past {
            if past_block.date.date() == week {
                shifts.push(past_block);
            }
        }

        // Get the total hours for the day (if there are multiple shifts)
        let sum = shifts.iter().fold(Duration::seconds(0), |acc, shift| {
            acc + Duration::seconds(shift.seconds)
        });

        // Display the results of the search
        println!("{:?}:", week.weekday());

        week = week.succ();

        if shifts.len() == 0 {
            println!("  {}\n", "No shifts".red());
            continue;
        }

        for shift in shifts {
            println!("  {}: {}", 
                format_time(Duration::seconds(shift.seconds)).blue(),
                shift.comment
                );
        }
        println!("{}\n", format!("{:2}(total={})", "", format_time(sum).blue()));
    }
}

///Edit some dates in the file
///Should not be used but provided because accidents happen
fn edit(_week: i64, _absolute: bool) {
    // Find the week that is specified as an argument

    // Give each shift of each day a id

    // Allow the user to specify
    // Then the user can edit the comment or length of time
    // Or delete it

    eprintln!("{}", "Not implemented yet".red());
    std::process::exit(1);
}

fn get_week(week: i64, absolute: bool) -> Date<Utc> {
    // Find the week specified by the int
    let mut week_day = if absolute {
        // Get the start of the year + the week specified
        Utc.ymd(Utc::now().year(), 1, 1) + Duration::weeks(week)
    } else {
        // Take the weeks from the current time
        (Utc::now() - Duration::weeks(week)).date()
    };

    // Find the Sunday of the week
    while week_day.weekday() != Weekday::Sun {
        week_day = week_day.pred();
    }

    week_day
}

///Update the most recent time block to the current time
fn update_last_block(times: &mut Vec<TimeBlock>) {
    if let Some((last, _)) = times.split_last_mut() {
        last.end = Utc::now();
    }
}

///Get the duration of the times in the timeblock vec
fn calculate_shift_length(times: &Vec<TimeBlock>) -> Duration {
    // Add up all the times
    let mut time_sum = Duration::seconds(0);
    for block in times {
        time_sum = time_sum +
            Duration::seconds(block.end.timestamp() - block.start.timestamp());
    }

    // Return the duration
    time_sum
}

fn format_time(dur: Duration) -> String {
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

///Main entry point of the cli
fn main() {
    let opts = Args::from_args();
    let mut file = open_file();

    // After opening the file, we need to read it into the FileFormat struct
    let mut data = read_file(&mut file);

    // Execute the method related to each argument
    match opts {
        Args::Start => start(&mut data),
        Args::Stop { comment } => stop(&mut data, comment),
        Args::Pause => pause(&mut data),
        Args::Status => status(&mut data),
        Args::Display { week, absolute } => display(week, absolute, &mut data),
        Args::Edit { week, absolute } => edit(week, absolute),
    }

    // After we manipulate the data structure -> write it back to the file
    write_file(file, data);
}

