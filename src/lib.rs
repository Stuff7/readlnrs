#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::read_key;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::read_key;

use std::io::{self, Write};

const CLEAR: &str = "\x1b[2K";

#[derive(Debug, Clone, Copy)]
pub enum Key {
  Special,
  Char(char),
  Enter,
  Backspace,
  ArrowRight,
  ArrowLeft,
  CtrlArrowRight,
  CtrlArrowLeft,
}

pub fn readln(buf: &mut String) -> io::Result<()> {
  let mut pos = 0;

  loop {
    match read_key()? {
      Key::Char(ch) => {
        buf.insert(pos, ch);
        pos += 1;
      }
      Key::Enter => {
        println!();
        return Ok(());
      }
      Key::Backspace => {
        if pos > 0 {
          pos -= 1;
          buf.remove(pos);
        }
      }
      Key::ArrowLeft => {
        pos = pos.saturating_sub(1);
      }
      Key::ArrowRight => {
        if pos < buf.len() {
          pos += 1;
        }
      }
      Key::CtrlArrowLeft => {
        while pos > 0 {
          pos -= 1;
          if buf.chars().nth(pos).is_some_and(|c| c == ' ') {
            break;
          }
        }
      }
      Key::CtrlArrowRight => {
        while pos < buf.len() {
          pos += 1;
          if buf.chars().nth(pos).is_some_and(|c| c == ' ') {
            break;
          }
        }
      }
      Key::Special => (),
    }

    print!("{}\r{}\r", CLEAR, buf);
    if pos > 0 {
      print!("\x1b[{}C", pos);
    }
    io::stdout().flush()?;
  }
}
