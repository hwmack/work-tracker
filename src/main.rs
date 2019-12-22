use std::fs::{ File, OpenOptions };
use std::io::{ Read, Write };

use structopt::StructOpt;
use dirs::home_dir;
use chrono::prelude::*;
use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};
use rmp_serde::Serializer;

///Arguments for the command line application
#[derive(Debug, StructOpt)]
#[structopt(name = "work", about = "Track time spent working")]
enum Args {
    #[structopt(name = "start", about = "Start/Resume tracking time")]
    Start,

    #[structopt(name = "stop", about = "stop tracking, and finish the current shift")]
    Stop,

    #[structopt(name = "pause", about = "pause tracking")]
    Pause,

    #[structopt(name = "status", about = "display the current state")]
    Status,

    #[structopt(name = "display", about = "display the hours for the current week")]
    Display {
        ///0 == the current week
        ///1-52 == the week of the current year
        ///-1-52 == the week relative of the current week
        #[structopt(default_value = "0")]
        week: i8
    },

    #[structopt(name = "edit", about = "edit raw time details if needed")]
    Edit,
}

///Possible time tracking states we could be in
#[derive(Serialize, Deserialize, Debug)]
enum TrackingState {
    Tracking,
    Stopped,
    Paused
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
    hours: f32,
    comment: String,
}

///Format of the file to read to and from RMP
#[derive(Serialize, Deserialize, Debug)]
struct FileFormat {
    version: String,
    state: TrackingState,
    times: Vec<TimeBlock>,
    past: Vec<PastTimeBlock>
}

///Open the file that we will use for writing the timesheets
fn open_file() -> File {
    let home = home_dir().expect("Could not find home directory");
    let path = home.join(".config/work/");

    // Create all of the dirs if they dont exist already
    std::fs::create_dir_all(&path).unwrap_or_else(|why| {
         println!("! {:?}", why.kind());
    });

    let path = path.join("timesheets.json");
    match OpenOptions::new().create(true).read(true).write(true).open(&path) {
        Ok(file) => file,
        Err(why) => panic!("Could not open {}: {}", path.display(), why)
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
        Err(why) => {
            println!("Empty file: {}", why);
            FileFormat {
                version: env!("CARGO_PKG_VERSION").into(),
                state: TrackingState::Stopped,
                times: Vec::new(),
                past: Vec::new(),
            }
        }
    }
}

///Writes the data to the file
fn write_file(mut file: File, data: FileFormat) {
    let mut buffer = Vec::new();
    data.serialize(&mut Serializer::new(&mut buffer))
        .expect("Could not serialize data");
    file.write_all(&buffer)
        .expect("Failed to write data to file");
}

///Start tracking time for this shift
///
///Puts a new start time into the structure and set the current state
///to tracking
///
///NOTE: If the state is current resumed, it will just continue
fn start(data: &mut FileFormat) {
    println!("Start shift");
}

///Stop tracking time
///
///Sets the current state to stopped
///
///Sums all the times together and inserts it into the past vec
fn stop(data: &mut FileFormat) {
    println!("Stop shift");
    todo!();
}

///Temporarily pause tracking time
///
///Updates the state to paused and sets the end of the current time
fn pause(data: &mut FileFormat) {
    println!("pause shift");
    todo!();
}

fn status() {
    todo!();
}

///Displays the hours for a given week
///
///Valid week values are:
/// 0: current week
/// 1-52: absolute week of the current year
/// -(1-52): relative week of the current week
fn display(week: i8, data: &mut FileFormat) {
    todo!();
}

fn edit() {
    todo!();
}

///Main entry point of the cli
fn main() {
    let opts = Args::from_args();
    let mut file = open_file();

    // After opening the file, we need to read it into the FileFormat struct
    let mut data = read_file(&mut file);

    println!("{:?}", data);

    // Execute the method related to each argument
    match opts {
        Args::Start => start(&mut data),
        Args::Stop => stop(&mut data),
        Args::Pause => pause(&mut data),
        Args::Status => status(),
        Args::Display{ week } => display(week, &mut data),
        Args::Edit => edit(),
    }

    // After we manipulate the data structure -> write it back to the file
    write_file(file, data);
}
