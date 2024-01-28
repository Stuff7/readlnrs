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
  NA,
  Char(char),
  Enter,
  Backspace,
  ArrowUp,
  ArrowDown,
  ArrowRight,
  ArrowLeft,
  CtrlBackspace,
  CtrlArrowRight,
  CtrlArrowLeft,
}

/// Reads user input in a loop with a customizable prompt and a command history.
///
/// # Blocking
///
/// This function blocks execution until the user presses Enter to submit the entered command.
/// During this blocking period, the user can navigate through the command history and perform editing
/// operations. Once the user presses Enter, the entered command is added to the history, and the function
/// returns a reference to the entered command in the history.
///
/// # Arguments
///
/// * `prompt` - The prompt to display to the user before reading input.
/// * `history` - A mutable reference to a vector representing the command history.
///   This vector is used to store and retrieve previously entered commands.
///
/// # Returns
///
/// Returns an `std::io::Result` containing a reference to the entered command string in the history.
///
/// # Examples
///
/// ```rust
/// use std::io;
/// use your_crate_name::{pushln, Key};
///
/// let mut command_history = Vec::new();
///
/// loop {
///     match pushln(">> ", &mut command_history) {
///         Ok(command) => {
///             // Handle the entered command or break the loop on a specific condition
///         }
///         Err(error) => {
///             eprintln!("Error reading user input: {}", error);
///             break;
///         }
///     }
/// }
/// ```
///
/// # Editing Operations
///
/// This function internally handles the following operations:
///
/// - `Key::Enter`: Accepts the current input and ends the input loop.
/// - `Key::ArrowUp`: Navigates to the previous input in the history.
/// - `Key::ArrowDown`: Navigates to the next input in the history.
///
/// Other editing operations are handled the same way as in the `readln` function, such as character insertion,
/// deletion, and cursor movement.
pub fn pushln<'a>(prompt: &str, history: &'a mut Vec<String>) -> io::Result<&'a str> {
  let mut local_history = Vec::new();
  let mut new_buf = String::new();
  let mut pos = 0;
  let mut hpos = history.len();
  let mut buf = &mut new_buf;
  let last_history_idx = history.len().saturating_sub(1);

  loop {
    promptln(prompt, buf, pos)?;

    match readch(buf, &mut pos)? {
      Key::Enter => break,
      Key::ArrowUp => hpos = hpos.saturating_sub(1),
      Key::ArrowDown => {
        if hpos < history.len() {
          hpos += 1
        }
      }
      _ => continue,
    }

    let local_pos = last_history_idx.wrapping_sub(hpos);
    buf = if let Some(item) = local_history.get_mut(local_pos) {
      item
    } else if let Some(item) = history.get(hpos) {
      // We want to be able to mutate the existing history items during the loop but
      // keep them the same after we return, that's why we clone them on demand here
      local_history.push(item.clone());
      &mut local_history[local_pos]
    } else {
      &mut new_buf
    };
    pos = buf.len();
  }

  println!();
  if buf.is_empty() {
    return Ok("");
  }

  let buf = buf.clone();
  history.push(buf);
  Ok(&history[history.len() - 1])
}

/// Reads user input with basic single-line navigation and editing.
///
/// # Blocking
///
/// This function blocks until the Enter key is pressed. The final input is stored in the provided buffer.
///
/// # Arguments
///
/// * `prompt` - The prompt displayed to the user before input is read.
/// * `buf` - A mutable reference to the buffer where the user input is stored.
///
/// # Returns
///
/// Returns an `std::io::Result` indicating the success or failure of the input operation.
///
/// # Examples
///
/// ```rust
/// use std::io;
///
/// let mut input_buffer = String::new();
///
/// match readln("Enter your name: ", &mut input_buffer) {
///     Ok(()) => {
///         println!("Hello, {}!", input_buffer.trim());
///     }
///     Err(error) => {
///         eprintln!("Error reading user input: {}", error);
///     }
/// }
/// ```
///
/// # Editing Operations
///
/// - `Key::Enter`: Accepts the current input and ends the input loop.
///
/// Other editing operations are handled the same way as in the `readch` function, such as character insertion,
/// deletion, and cursor movement.
pub fn readln(prompt: &str, buf: &mut String) -> io::Result<()> {
  let mut pos = buf.len();

  loop {
    promptln(prompt, buf, pos)?;
    if matches!(readch(buf, &mut pos)?, Key::Enter) {
      break;
    }
  }

  println!();
  Ok(())
}

fn promptln(prompt: &str, input: &str, mut cursor: usize) -> io::Result<()> {
  print!("{}\r{}{}\r", CLEAR, prompt, input);
  cursor += prompt.len();
  if cursor > 0 {
    print!("\x1b[{}C", cursor);
  }
  io::stdout().flush()
}

/// Reads a single byte of user input, allowing basic editing operations with a cursor.
///
/// # Blocking
///
/// This function blocks until a key event is detected and returns the corresponding `Key` enum
/// representing the user input. The function also updates the provided buffer and cursor
/// position based on the input.
///
/// # Arguments
///
/// * `buf` - A mutable reference to the buffer where user input is stored.
/// * `pos` - A mutable reference to the cursor position.
///
/// # Returns
///
/// Returns an `std::io::Result` containing the `Key` enum representing the user input.
///
/// # Examples
///
/// ```rust
/// use std::io;
/// use your_crate_name::stdin_edit;
///
/// let mut buffer = String::new();
/// let mut cursor_position = 0;
///
/// loop {
///     match readch(&mut buffer, &mut cursor_position) {
///         Ok(key) => {
///             // Handle the key or break the loop on a specific condition
///         }
///         Err(error) => {
///             eprintln!("Error reading user input: {}", error);
///             break;
///         }
///     }
/// }
/// ```
///
/// # Editing Operations
///
/// - `Key::Char(ch)`: Inserts the character `ch` at the current cursor position.
/// - `Key::Backspace`: Deletes the character before the cursor position.
/// - `Key::ArrowLeft`: Moves the cursor one position to the left.
/// - `Key::ArrowRight`: Moves the cursor one position to the right.
/// - `Key::CtrlBackspace`: Deletes the word before the cursor position.
/// - `Key::CtrlArrowLeft`: Moves the cursor to the beginning of the previous word.
/// - `Key::CtrlArrowRight`: Moves the cursor to the beginning of the next word.
pub fn readch(buf: &mut String, pos: &mut usize) -> io::Result<Key> {
  let key = read_key()?;
  match key {
    Key::Char(ch) => {
      buf.insert(*pos, ch);
      *pos += 1;
    }
    Key::Backspace => {
      if *pos > 0 {
        *pos -= 1;
        buf.remove(*pos);
      }
    }
    Key::ArrowLeft => {
      *pos = pos.saturating_sub(1);
    }
    Key::ArrowRight => {
      if *pos < buf.len() {
        *pos += 1;
      }
    }
    Key::CtrlBackspace => {
      let idx = buf[..*pos].as_bytes().iter().rposition(|c| c == &b' ').unwrap_or_default();
      buf.replace_range(idx..*pos, "");
      *pos = idx;
    }
    Key::CtrlArrowLeft => {
      *pos = buf[..*pos].as_bytes().iter().rposition(|c| c == &b' ').unwrap_or_default();
    }
    Key::CtrlArrowRight => {
      let bytes = buf.as_bytes();
      while *pos < buf.len() {
        *pos += 1;
        if bytes.get(*pos).is_some_and(|b| b == &b' ') {
          break;
        }
      }
    }
    _ => (),
  }
  Ok(key)
}
