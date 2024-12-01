use std::{fs::{File, OpenOptions}, io::Write};

pub struct Logger {
  file: File,
}

impl Logger {
  pub fn new<'a>(file_path: &'a str) -> Logger {
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(file_path)
      .expect("Unable to open log file");

    Logger { file }
  }

  pub fn log(&mut self, message: &[u8]) {
    let _ = self.file.write_all(message);
  }
}
