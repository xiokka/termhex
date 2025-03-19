use crossterm::{ 
	event::{KeyCode, KeyEvent, read, Event}, 
	terminal::{ClearType, Clear, disable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen, size, SetSize},
	style::{Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color},
	cursor::{Hide, Show, MoveTo},
	execute, 
	queue
};

use std::{
	io::{stdout, self, Write},
	fs, env,
	fmt::write
};
use std::fs::File;

const CELLS_PER_HEX:u16 = 2;
const MENU_HEIGHT:u16 = 1;
const PERCENTAGE_POSITION:u16 = 55;

trait Colors {
	fn get_color_by_category(&self) -> Color;
}

impl Colors for u8 {
        // COLOR PATTERN
        fn get_color_by_category(&self) -> Color {
                match self {
                        _ if self.is_ascii_graphic() => Color::Cyan,
                        _ if self.is_ascii_whitespace() => Color::Red,
                        _ if self.is_ascii_control() => Color::Reset,
                        _ if !self.is_ascii() => Color::Yellow,
                        _ => Color::Green
                }
        }
}

trait Render{
	fn render_as_hex(&self, start: usize, size: (u16, u16) );
	fn render_as_char(&self, start: usize, size: (u16, u16) );
}

impl Render for Vec<u8> {
	fn render_as_hex(&self, start: usize, size: (u16, u16)) {
		let mut current_pos = start;
		let mut stdout = stdout();
                queue!(stdout, MoveTo(0, 0));
                for i in 0..size.1-MENU_HEIGHT {
                        for j in 0..16 {
                                if current_pos >= self.len() {
                                        let formatted = format!("{: >width$}", this_char = ' ', width = 2);
                                        queue!(stdout, MoveTo(j * (2 * CELLS_PER_HEX), i), Print(formatted) );
                                        continue;
                                } else {
					let hex_string = format!("{:#x}", self[current_pos]);
                                        let without_prefix = hex_string.trim_start_matches("0x");
                                        queue!(stdout, MoveTo(j * (2 * CELLS_PER_HEX), i), SetForegroundColor(self[current_pos].get_color_by_category() ), Print( format!("{:0>width$}", without_prefix, width = 2)    ));
				}
				current_pos += 1;
			};
		}
                let bottom_line = format!("[0x{:0>width$x}, 0x{:0>width$x}] / 0x{:0>width$x}", start, current_pos, self.len(), width = self.len().to_string().len() );
                queue!(stdout, MoveTo(0, size.1), SetBackgroundColor(Color::White), SetForegroundColor(Color::Black), Print(bottom_line), ResetColor );
		let menu_percentage:String = format!("{: >6.2}%", (current_pos as f64 / self.len() as f64) * 100.0);
		queue!(stdout, MoveTo(55, size.1),SetBackgroundColor(Color::White), SetForegroundColor(Color::Black), Print(menu_percentage), ResetColor);
//size.0-menu_percentage.len()
		stdout.flush();
	}

	fn render_as_char(&self, start:usize, size: (u16, u16) ) {
                let mut current_pos = start;
		let mut stdout = stdout();
                queue!(stdout, MoveTo(0, 0));
                for i in 0..size.1-MENU_HEIGHT {
                        for j in 0..16 {
                                if current_pos >= self.len() {
                                        let formatted = format!("{: >width$}", this_char = ' ', width = 2);
                                        queue!(stdout, MoveTo(j * (2 * CELLS_PER_HEX), i), Print(formatted) );
                                        continue;
                                }
                                else {
                                        let this_char = self[current_pos] as char;
                                        let this_char = if this_char.is_ascii_graphic() { this_char } else { ' ' };
                                        let without_prefix = format!("{: >width$}", this_char, width = 2);
                                        queue!(stdout, MoveTo(j * (2 * CELLS_PER_HEX), i), Print(without_prefix) );
                                }
                                current_pos += 1;
                        }
                }
		let bottom_line = format!("[0x{:0>width$x}, 0x{:0>width$x}] / 0x{:0>width$x}", start, current_pos, self.len(), width = self.len().to_string().len() );
		queue!(stdout, MoveTo(0, size.1), SetBackgroundColor(Color::White), SetForegroundColor(Color::Black), Print(bottom_line), ResetColor );
		queue!(stdout, MoveTo(55, size.1), SetBackgroundColor(Color::White), SetForegroundColor(Color::Black),Print(format!("{: >6.2}%", (current_pos as f64 / self.len() as f64) * 100.0)), ResetColor);
		stdout.flush();
	}
}

fn main() -> io::Result<()> {
        let args: Vec<String> = env::args().collect();
        if args.len() <= 1 {
                println!("No arguments provided. Usage: termhex [file]");
                return Ok(());
        }
        let binding = args[1].to_string();
        let image_path = std::path::Path::new(&binding); // Store the first argument in the image_path variable
	let data: Vec<u8> = fs::read(image_path)?;
	execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        crossterm::terminal::enable_raw_mode().expect("Failed to enable raw mode");
	io::stdout().write_all(b"\n").unwrap();
	let mut stdout = stdout();
	
	// Set starting position to 0 and render the file in hex mode
	let mut print_as_char:bool = false;
	let mut user_position: usize = 0;
	let mut current_size:(u16,u16) = size().unwrap();
	loop {
		match print_as_char {
			true => data.render_as_char(user_position, current_size),
			_ => data.render_as_hex(user_position, current_size)
		}
                if let Ok(event) = read() {
                        match event {
                                Event::Resize(_, _) => {
                                        execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
					current_size = size().unwrap();
                                }
                                Event::Key(KeyEvent { code, .. }) => {
                                        match code {
                                                KeyCode::Up => {
							if (user_position as i32 - 16 >= 0 ) {
                                                        	user_position -= 16;
							}
                                                }
                                                KeyCode::Down => {
							if (user_position as i32 + 16 < data.len() as i32) {
                                                        	user_position += 16;
							}
                                                }

                                                KeyCode::PageDown => {
							for i in 0..current_size.1 {
	                                                        if (user_position as i32 + 16 < data.len() as i32) {
        	                                                        user_position += 16;
        	                                                }
							}
                                                }
                                                KeyCode::PageUp => {
							for i in 0..current_size.1 {
	                                                        if (user_position as i32 - 16 >= 0 ) {
	                                                                user_position -= 16;
	                                                        }
							}
                                                }
                                                KeyCode::Home => {
                                                                user_position = 0;
                                                }

                                                KeyCode::End => {
								let total_rows = data.len() / 16;
                                                                user_position = 16 * total_rows;
                                                }

                                                KeyCode::Tab => {
                                                        print_as_char = !print_as_char;
							execute!(stdout, Clear(ClearType::All));
                                                }
						KeyCode::Char('e') | KeyCode::Char('E') => {
							let vector_of_strings = extract_strings(&data);
							let mut output_file = format!("{}_export.txt", image_path.display() );
							let mut file = File::create(&output_file).expect("Failed to create file");
							file.write_all(vector_of_strings.join("\n").as_bytes()).expect("Failed to write to file");

						}
                                                KeyCode::Char('q') | KeyCode::Char('Q') => {
                                                        disable_raw_mode().expect("Failed to disable raw mode");
                                                        let stdout = io::stdout();
                                                        let mut handle = stdout.lock();
                                                        handle.write_all(b"\n").unwrap();
                                                        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
                                                        std::process::exit(0);
                                                }
                                                _ => {}
                                        }
                                }
                                _ => {}
                        }
                }


	}
	return Ok(());

}


fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut current_string = Vec::new();

    for &byte in data {
	if byte.is_ascii_control() { 
            if !current_string.is_empty() {
                if let Ok(s) = String::from_utf8(current_string.clone()) {
                    strings.push(s);
                    current_string.clear();
                } else {
                    // Handle invalid UTF-8 sequence if needed
                    current_string.clear();
                }
            }
        } else {
             if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
		current_string.push(byte);
		}
        }
    }

    // If there's any remaining data in the buffer after the last delimiter,
    // treat it as a separate string
    if !current_string.is_empty() {
        if let Ok(s) = String::from_utf8(current_string) {
            strings.push(s);
        }
    }

    strings
}

