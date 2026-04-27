use std::io::{self, Write};

pub fn cmd_clear() -> Result<(String, String), Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush()?;
    Ok((String::new(), String::new()))
}