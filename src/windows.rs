use crate::Key;
use std::io;
use windows_sys::Win32::{
  System::Console::{self, GetStdHandle, ReadConsoleInputA, INPUT_RECORD, STD_INPUT_HANDLE},
  UI::Input::KeyboardAndMouse as Wk,
};

const KEY_EVENT: u16 = Console::KEY_EVENT as u16;

pub fn read_key() -> io::Result<Key> {
  let mut dw_events_read: u32 = 0;

  unsafe {
    let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
    let mut ir_input_record: INPUT_RECORD = std::mem::zeroed();

    while ReadConsoleInputA(h_stdin, &mut ir_input_record, 1, &mut dw_events_read) != 0 {
      if ir_input_record.EventType == KEY_EVENT && ir_input_record.Event.KeyEvent.bKeyDown != 0 {
        let ctrl = ir_input_record.Event.KeyEvent.dwControlKeyState & 0x0008 != 0;
        return Ok(match ir_input_record.Event.KeyEvent.wVirtualKeyCode {
          Wk::VK_W | Wk::VK_BACK if ctrl => Key::CtrlBackspace,
          Wk::VK_LEFT if ctrl => Key::CtrlArrowLeft,
          Wk::VK_RIGHT if ctrl => Key::CtrlArrowRight,
          Wk::VK_RETURN => Key::Enter,
          Wk::VK_BACK => Key::Backspace,
          Wk::VK_UP => Key::ArrowUp,
          Wk::VK_DOWN => Key::ArrowDown,
          Wk::VK_LEFT => Key::ArrowLeft,
          Wk::VK_RIGHT => Key::ArrowRight,
          _ if !ctrl && ir_input_record.Event.KeyEvent.uChar.AsciiChar != 0 => Key::Char(ir_input_record.Event.KeyEvent.uChar.AsciiChar as char),
          _ => Key::NA,
        });
      }
    }
  }

  if dw_events_read == 0 {
    return Err(io::Error::last_os_error());
  }

  Ok(Key::NA)
}
