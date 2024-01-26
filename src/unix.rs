use super::Key;
use std::io::{self};
use std::os::raw::{c_char, c_int};

pub fn read_key() -> io::Result<Key> {
  let mut ch = getch()?;

  match ch {
    27 => (),
    10 => return Ok(Key::Enter),
    127 => return Ok(Key::Backspace),
    _ => return Ok(Key::Char(ch as char)),
  }

  let mut pos = 0;
  while pos < ESC_SEQ_LEN + 1 {
    ch = getch()?;

    for ref seq in ESC_SEQ_LIST {
      if pos == seq.value.len() {
        continue;
      }

      if seq.value.chars().nth(pos).map(|c| c as u8).is_some_and(|c| c == ch) && seq.value.len() - 1 == pos {
        return Ok(seq.key);
      }
    }

    pos += 1;
  }

  Ok(Key::Special)
}

const ESC_SEQ_LEN: usize = 4;
const ESC_SEQ_LIST: [EscapeSequence; ESC_SEQ_LEN] = [
  EscapeSequence {
    key: Key::ArrowRight,
    value: "[C",
  },
  EscapeSequence {
    key: Key::ArrowLeft,
    value: "[D",
  },
  EscapeSequence {
    key: Key::CtrlArrowRight,
    value: "[1;5C",
  },
  EscapeSequence {
    key: Key::CtrlArrowLeft,
    value: "[1;5D",
  },
];

struct EscapeSequence {
  key: Key,
  value: &'static str,
}

extern "C" {
  fn tcgetattr(fd: c_int, termios_p: *mut libc::termios) -> c_int;
  fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const libc::termios) -> c_int;
  fn fflush(stream: *mut libc::FILE) -> c_int;
  fn read(fd: c_int, buf: *mut c_char, count: usize) -> isize;
}

const STDIN_FILENO: c_int = 0;
const TCSANOW: c_int = 0;
const TCSADRAIN: c_int = 1;

fn getch() -> io::Result<u8> {
  let mut buf: c_char = 0;
  let mut old: libc::termios = unsafe { std::mem::zeroed() };

  unsafe {
    if fflush(std::ptr::null_mut()) < 0 {
      return Err(io::Error::last_os_error());
    }

    if tcgetattr(STDIN_FILENO, &mut old) < 0 {
      return Err(io::Error::last_os_error());
    }

    old.c_lflag &= !libc::ICANON;
    old.c_lflag &= !libc::ECHO;
    old.c_cc[libc::VMIN] = 1;
    old.c_cc[libc::VTIME] = 0;

    if tcsetattr(STDIN_FILENO, TCSANOW, &old) < 0 {
      return Err(io::Error::last_os_error());
    }

    if read(STDIN_FILENO, &mut buf, 1) < 0 {
      return Err(io::Error::last_os_error());
    }

    old.c_lflag |= libc::ICANON;
    old.c_lflag |= libc::ECHO;

    if tcsetattr(STDIN_FILENO, TCSADRAIN, &old) < 0 {
      return Err(io::Error::last_os_error());
    }
  }

  Ok(buf as u8)
}
