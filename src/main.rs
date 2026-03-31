mod lib;

use lib::{ClockSession, TimeProvider};
use std::{fs, str::FromStr};
use clap::{Parser, Subcommand};
use time::{Duration, UtcOffset, macros::format_description};
use anyhow::{Context, Result, anyhow};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    operation: Operation,
}

#[derive(Subcommand, Debug)]
enum Operation {
    #[command(visible_alias = "b")]
    Begin {
        expected_duration: Option<String>,
        #[arg(short, long)]
        label: Option<String>,
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    #[command(visible_alias = "e")]
    End {
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    #[command(visible_alias = "p")]
    Pause,
    #[command(visible_alias = "r")]
    Resume,
    #[command(visible_alias = "s")]
    Section {
        #[arg(short, long)]
        label: Option<String>,
    },
    #[command(visible_alias = "d")]
    Display {
        #[arg(short, long, default_value_t = false)]
        pauses: bool,
    },
}

const CURRENT_SESSION_FILENAME: &str = "session_current.json";

macro_rules! format_duration {
    ($duration:expr) => {
        format!("{}", Duration::seconds($duration.whole_seconds()))
    };
}

macro_rules! format_date {
    ($date:expr, $offset:expr) => {
        format!("{}", $date.to_offset($offset).format(&format_description!(version = 2, "[day]/[month]/[year] [hour]:[minute]:[second]"))?)
    };
}

fn time_duration_from_humantime(duration_str: &str) -> Result<Duration> {
    let duration_secs = humantime::Duration::from_str(duration_str).context(
        format!("Failed to parse duration string: {}", duration_str)
    )?.as_secs_f32();
    Ok(Duration::seconds_f32(duration_secs))
}

fn save_current_session(clock_session: &ClockSession) -> Result<()> {
    let writer = fs::File::create(CURRENT_SESSION_FILENAME).context("Failed to create current session file")?;
    serde_json::to_writer(writer, clock_session).context("Failed to write current session JSON")?;

    if clock_session.is_finished() {
        let filename = format!("session_{}.json", clock_session.start().unix_timestamp_nanos());
        fs::rename(CURRENT_SESSION_FILENAME, filename).context("Failed to rename current session file")?;
    }
    Ok(())
}   

fn load_current_session() -> Result<Option<ClockSession>> {
    let reader = fs::File::open(CURRENT_SESSION_FILENAME);
    match reader {
        Ok(file) => {
            let clock_session = serde_json::from_reader(file).context("Failed to parse current session JSON")?;
            Ok(Some(clock_session))
        },
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(e).context("Failed to open current session file"),
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut current_session = load_current_session()?;
    if let Some(ref mut current_session) = current_session {
        current_session.update_time_provider();
    }

    match &args.operation {
        Operation::Begin { expected_duration, label, force } => {
            if !force && current_session.is_some() {
                return Err(anyhow!("A session is already in progress. Please end it before starting a new one"));
            }

            let expected_duration = if let Some(expected_duration_str) = expected_duration {
                Some(time_duration_from_humantime(expected_duration_str)?)
            } else {
                None
            };
            let clock_session: ClockSession = ClockSession::new(TimeProvider::new(), expected_duration, label.clone());
            save_current_session(&clock_session)?;
        },
        Operation::End { force } => {
            if current_session.is_none() {
                return Err(anyhow!("No session in progress. Please start a session before ending it"));
            }
            let mut clock_session = current_session.unwrap();
            clock_session.end(*force).map_err(|e| anyhow!("Failed to end current session: {}", e))?;
            save_current_session(&clock_session)?;
        },
        Operation::Pause => {
            if current_session.is_none() {
                return Err(anyhow!("No session in progress. Please start a session before pausing it"));
            }
            let mut clock_session = current_session.unwrap();
            clock_session.pause().map_err(|e| anyhow!("Failed to pause current session: {}", e))?;
            save_current_session(&clock_session)?;
        },
        Operation::Resume => {
            if current_session.is_none() {
                return Err(anyhow!("No session in progress. Please start a session before resuming it"));
            }
            let mut clock_session = current_session.unwrap();
            clock_session.resume().map_err(|e| anyhow!("Failed to resume current session: {}", e))?;
            save_current_session(&clock_session)?;
        },
        Operation::Section { label } => {
            if current_session.is_none() {
                return Err(anyhow!("No session in progress. Please start a session before adding a section"));
            }

            let mut clock_session = current_session.unwrap();
            clock_session.new_section(label.clone());
        }
        Operation::Display { pauses } => {
            if current_session.is_none() {
                println!("No session in progress");
                return Ok(());
            }

            let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);

            let clock_session = current_session.unwrap();
            println!("Session started at: {}", format_date!(clock_session.start(), local_offset));

            if let Some(expected_duration) = clock_session.expected_duration() {
                println!("Expected duration: {}", format_duration!(expected_duration));
                println!("Expected end time: {}", format_date!(clock_session.start() + expected_duration + clock_session.total_paused_duration(), local_offset));
            }
            println!("Effective duration: {}", format_duration!(clock_session.effective_duration()));
            if clock_session.is_paused() {
                println!("Session is currently paused");
            } else {
                println!("Session is currently running");
            }

            if *pauses {
                if clock_session.pauses().is_empty() {
                    println!("No pauses recorded");
                } else {
                    println!("Total paused duration: {}", format_duration!(clock_session.total_paused_duration()));
                    println!("Pauses:");
                    for (i, pause) in clock_session.pauses().iter().enumerate() {
                        let pause_start = format_date!(pause.start(), local_offset);
                        let pause_duration = format_duration!(pause.duration(clock_session.time_provider()));
                        println!("  {}. Started at: {}, Duration: {}", i + 1, pause_start, pause_duration);
                    }
                }
            }
        }
    };
    
     Ok(())
}
