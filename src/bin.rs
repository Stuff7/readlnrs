use std::io;

fn main() -> io::Result<()> {
  let mut buf = String::new();
  readln::readln(&mut buf)?;
  println!("OUT: {buf}");
  Ok(())
}
