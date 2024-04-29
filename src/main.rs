use std::process;
use calc_worktime::run;

fn main() {
  run().unwrap_or_else(|err| {
    eprintln!("计算工时失败: {}", err);
    process::exit(1);
  });
}
