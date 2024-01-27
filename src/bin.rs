use std::io;

fn main() -> io::Result<()> {
  let mut history = Vec::new();

  loop {
    let cmd = readln::pushln("> ", &mut history)?;

    println!("OUT: {cmd:?}");
    if cmd == "q" {
      break;
    }
  }

  Ok(())
}
