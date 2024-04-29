use std::error::Error;
use std::fs;
use chrono::Duration;
use chrono::prelude::*;
use serde::Deserialize;
use clap::Parser;

/// 计算30天平均工时
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// 工时配置文件
  #[clap(short, long)]
  pub(crate) config: String,
}

pub struct WorkDay {
  pub start_time: DateTime<Utc>,
  pub end_time: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkDayUpdated {
  pub start_time: String,
  pub end_time: String,
}

pub fn calculate_average_work_hours(work_days: Vec<WorkDay>) -> f64 {
  let thirty_days_ago = Utc::now() - Duration::days(30);
  let mut total_work_hours = Duration::seconds(0);

  let mut days = 0;

  for work_day in work_days {
    if work_day.start_time < thirty_days_ago {
      continue;
    }

    days += 1; // counting the work days

    // Work start and end
    let start_time = work_day.start_time;
    let end_time = work_day.end_time;

    // Break start and end
    let lunch_break_start = work_day.start_time.date().and_hms(12, 0, 0);
    let lunch_break_end = work_day.start_time.date().and_hms(13, 30, 0);
    let evening_break_start = work_day.start_time.date().and_hms(18, 0, 0);
    let evening_break_end = work_day.start_time.date().and_hms(18, 30, 0);

    let start = work_day.start_time.date().and_hms(8, 30, 0);
    let end = work_day.start_time.date().and_hms(23, 00, 0);

    let mut work_duration = end_time.signed_duration_since(start_time);

    if start_time < start {
      work_duration = work_duration - start.signed_duration_since(start_time)
    }

    if end_time > end {
      work_duration = work_duration - end_time.signed_duration_since(end)
    }

    if start_time < lunch_break_start && end_time > lunch_break_end {
      work_duration = work_duration - Duration::minutes(90);
    }

    if end_time > evening_break_end {
      work_duration = work_duration - Duration::minutes(30); // deduct 30 minutes for evening break
    }

    if evening_break_start < end_time && end_time < evening_break_end {
      work_duration = work_duration - end_time.signed_duration_since(evening_break_start)
    }
    total_work_hours = total_work_hours + work_duration;
  }

  total_work_hours.num_minutes() as f64 / 60.0 / days as f64 // convert minutes to hours and average over days
}

pub fn run() -> Result<(), Box<dyn Error>> {
  let config_path = Args::parse().config;
  let config_str = fs::read_to_string(config_path)?;
  let parsed_data: Vec<WorkDayUpdated> = serde_json::from_str(&config_str)?;
  let mut work_days: Vec<WorkDay> = Vec::new();

  for wd in parsed_data {
    let start = NaiveDateTime::parse_from_str(&wd.start_time, "%Y-%m-%d %H:%M:%S").unwrap();
    let end = NaiveDateTime::parse_from_str(&wd.end_time, "%Y-%m-%d %H:%M:%S").unwrap();

    let work_day = WorkDay {
      start_time: DateTime::from_naive_utc_and_offset(start, Utc),
      end_time: DateTime::from_naive_utc_and_offset(end, Utc),
    };

    work_days.push(work_day);
  }
  let avg = calculate_average_work_hours(work_days);
  println!("平均工时：{:.2}小时", avg);
  Ok(())
}