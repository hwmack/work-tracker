use std::fs::{File, OpenOptions};
use std::fmt;
use std::io::{Write, Read, Seek, SeekFrom};

use serde::{Serialize, Deserialize};
use rmp_serde::Serializer;
use chrono::serde::ts_seconds;
use chrono::{Duration, DateTime, Utc};
use dirs::home_dir;

///Possible time tracking states we could be in
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[repr(u8)]
pub enum TrackingState {
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
pub struct TimeBlock {
    #[serde(with = "ts_seconds")]
    pub start: DateTime<Utc>,

    #[serde(with = "ts_seconds")]
    pub end: DateTime<Utc>,

    pub finished_tasks: Vec<String>,
}

///Where all TimeBlocks are stored after running the `stop` command
#[derive(Serialize, Deserialize, Debug)]
pub struct PastTimeBlock {
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
    pub seconds: i64,
    pub comment: String,
    pub finished_tasks: Vec<String>
}

///Format of the file to read to and from RMP
#[derive(Serialize, Deserialize, Debug)]
pub struct FileFormat {
    pub version: String,
    pub state: TrackingState,
    pub tasks: Vec<String>,
    pub times: Vec<TimeBlock>,
    pub past: Vec<PastTimeBlock>,
}

///Update the most recent time block to the current time
pub fn update_last_block(times: &mut Vec<TimeBlock>) {
    if let Some((last, _)) = times.split_last_mut() {
        last.end = Utc::now();
    }
}

///Get the duration of the times in the timeblock vec
pub fn calculate_shift_length(times: &Vec<TimeBlock>) -> Duration {
    // Add up all the times
    let mut time_sum = Duration::seconds(0);
    for block in times {
        time_sum = time_sum +
            Duration::seconds(block.end.timestamp() - block.start.timestamp());
    }

    // Return the duration
    time_sum
}

///Open the file that we will use for writing the timesheets
pub fn open_file() -> File {
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
pub fn read_file(file: &mut File) -> FileFormat {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read timesheets file");

    match rmp_serde::from_read_ref(&buffer) {
        Ok(result) => result,
        Err(_) => FileFormat {
            version: env!("CARGO_PKG_VERSION").into(),
            state: TrackingState::Stopped,
            tasks: vec![],
            times: Vec::new(),
            past: Vec::new(),
        },
    }
}

///Writes the data to the file
pub fn write_file(mut file: File, data: FileFormat) {
    let mut buffer = Vec::new();
    data.serialize(&mut Serializer::new(&mut buffer))
        .expect("Could not serialize data");

    // Truncate the file and seek to the start
    file.set_len(0).expect("Could not clear file for writing");
    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");

    let len = file.write(&buffer).expect("Failed to write data to file");

    // Assert expected amount was written to file
    assert_eq!(len, buffer.len())
}
