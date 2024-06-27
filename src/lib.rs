use std::error::Error;
use std::fs;

use chrono::Duration;
use chrono::prelude::*;
use clap::Parser;
use serde::Deserialize;

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

const LUNCH_BREAK_DURATION_MINUTES: i64 = 90;
const EVENING_BREAK_DURATION_MINUTES: i64 = 30;

fn calculate_work_hours(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Duration {
  let tz = start_time.offset();

  let start_of_day = DateTime::<Utc>::from_naive_utc_and_offset(
    start_time.date_naive().and_hms_opt(8, 30, 0).unwrap(),
    *tz,
  );
  let end_of_day = DateTime::<Utc>::from_naive_utc_and_offset(
    start_time.date_naive().and_hms_opt(23, 00, 0).unwrap(),
    *tz,
  );
  let lunch_break_start = DateTime::<Utc>::from_naive_utc_and_offset(
    start_time.date_naive().and_hms_opt(12, 0, 0).unwrap(),
    *tz,
  );
  let lunch_break_end = DateTime::<Utc>::from_naive_utc_and_offset(
    start_time.date_naive().and_hms_opt(13, 30, 0).unwrap(),
    *tz,
  );
  let evening_break_start = DateTime::<Utc>::from_naive_utc_and_offset(
    start_time.date_naive().and_hms_opt(18, 0, 0).unwrap(),
    *tz,
  );
  let evening_break_end = DateTime::<Utc>::from_naive_utc_and_offset(
    start_time.date_naive().and_hms_opt(18, 30, 0).unwrap(),
    *tz,
  );

  let mut work_duration = end_time.signed_duration_since(start_time);

  if start_time < start_of_day {
    work_duration -= start_of_day.signed_duration_since(start_time);
  }

  if end_time > end_of_day {
    work_duration -= end_time.signed_duration_since(end_of_day);
  }

  if start_time < lunch_break_start && end_time > lunch_break_end {
    work_duration -= Duration::minutes(LUNCH_BREAK_DURATION_MINUTES);
  }

  if end_time > evening_break_end {
    work_duration -= Duration::minutes(EVENING_BREAK_DURATION_MINUTES);
  }

  if evening_break_start < end_time && end_time < evening_break_end {
    work_duration -= end_time.signed_duration_since(evening_break_start);
  }

  work_duration
}

pub fn calculate_average_work_hours(work_days: Vec<WorkDay>) -> f64 {
  let thirty_days_ago = Utc::now() - Duration::days(30);
  let mut total_work_hours = Duration::seconds(0);
  let mut days = 0;

  for work_day in work_days {
    if work_day.start_time < thirty_days_ago {
      continue;
    }

    days += 1;
    total_work_hours += calculate_work_hours(work_day.start_time, work_day.end_time);
  }

  total_work_hours.num_minutes() as f64 / 60.0 / days as f64
}

pub fn run() -> Result<(), Box<dyn Error>> {
  let config_path = Args::parse().config;
  let config_str = fs::read_to_string(config_path)?;
  let parsed_work_days: Vec<WorkDayUpdated> = serde_json::from_str(&config_str)?;

  let work_days = parsed_work_days
      .iter()
      .map(|wd| {
        let start = NaiveDateTime::parse_from_str(&wd.start_time, "%Y-%m-%d %H:%M:%S").unwrap();
        let end = NaiveDateTime::parse_from_str(&wd.end_time, "%Y-%m-%d %H:%M:%S").unwrap();

        WorkDay {
          start_time: DateTime::from_naive_utc_and_offset(start, Utc),
          end_time: DateTime::from_naive_utc_and_offset(end, Utc),
        }
      })
      .collect();

  let avg = calculate_average_work_hours(work_days);

  println!("平均工时：{:.2}小时", avg);

  Ok(())
}
