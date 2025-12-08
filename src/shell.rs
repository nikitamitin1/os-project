use crate::{
    keyboard,
    parser::{int_to_str_buf, parse_int_from_str, ParseError},
    vga_buffer::{self, get_color_code, Color},
};
use core::hint::spin_loop;
use crate::vga_buffer::write_byte;

const INPUT_BUFFER_LEN: usize = 128;

pub struct Shell {
    buffer: [u8; INPUT_BUFFER_LEN],
    len: usize,
}

enum CommandToExecute<'a> {
    Greet { name: &'a str },
    Sum { a: i64, b: i64 },
    Diff { a: i64, b: i64 },
    Min { a: i64, b: i64 },
    Max { a: i64, b: i64 },
    Exit,
}

enum CommandError {
    UnknownCommand,
    InvalidArguments,
    ExecutionFailed,
    Parse(ParseError),
}

enum CommandResult {
    Success,
    Exit,
    Error(CommandError),
}

fn command_parser<'a>(input: &'a str) -> Result<CommandToExecute<'a>, CommandError> {
    let mut parts = input.trim().split_whitespace();
    let cmd = parts.next().ok_or(CommandError::UnknownCommand)?;

    match cmd {
        "greet" => {
            let name = parts.next().unwrap_or("stranger");
            Ok(CommandToExecute::Greet { name })
        }
        "sum" => {
            let a_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let b_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let a = parse_int_from_str(a_str).map_err(CommandError::Parse)?;
            let b = parse_int_from_str(b_str).map_err(CommandError::Parse)?;
            Ok(CommandToExecute::Sum { a, b })
        }
        "diff" => {
            let a_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let b_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let a = parse_int_from_str(a_str).map_err(CommandError::Parse)?;
            let b = parse_int_from_str(b_str).map_err(CommandError::Parse)?;
            Ok(CommandToExecute::Diff { a, b })
        }
        "min" => {
            let a_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let b_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let a = parse_int_from_str(a_str).map_err(CommandError::Parse)?;
            let b = parse_int_from_str(b_str).map_err(CommandError::Parse)?;
            Ok(CommandToExecute::Min { a, b })
        }
        "max" => {
            let a_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let b_str = parts.next().ok_or(CommandError::InvalidArguments)?;
            let a = parse_int_from_str(a_str).map_err(CommandError::Parse)?;
            let b = parse_int_from_str(b_str).map_err(CommandError::Parse)?;
            Ok(CommandToExecute::Max { a, b })
        }
        "exit" => Ok(CommandToExecute::Exit),
        _ => Err(CommandError::UnknownCommand),
    }
}

fn print_string(s: &str, color_code: vga_buffer::ColorCode) {
    for byte in s.bytes() {
        vga_buffer::write_byte(byte, color_code);
    }
}

fn print_os_version(os_version: &str) {
    let color_code = get_color_code(Color::LightCyan, Color::Black);
    print_string(os_version, color_code);
}

fn print_hello() {
    let color_code = get_color_code(Color::LightGreen, Color::Black);
    print_string("Hello, from Shell!\n", color_code);
}

fn print_prompt() {
    let color_code = get_color_code(Color::Yellow, Color::Black);
    print_string("shell> ", color_code);
}

pub fn print_command_output(output: &str) {
    let color_code = get_color_code(Color::White, Color::Black);
    print_string(output, color_code);
}

impl Shell {
    fn new() -> Self {
        Shell {
            buffer: [0; INPUT_BUFFER_LEN],
            len: 0,
        }
    }

    fn run(&mut self) -> ! {
        print_prompt();
        loop {
            let scancode = keyboard::read_scancode_safe();
            if let Some(ascii) = vga_buffer::scancode_to_ascii(scancode) {
                self.handle_input_byte(ascii);
            }
        }
    }

    fn handle_input_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.echo_byte(byte);
                self.process_buffer();
                self.clear_buffer();
                print_prompt();
            }
            0x08 => {
                self.handle_backspace();
            }
            _ => {
                if self.len < self.buffer.len() {
                    self.buffer[self.len] = byte;
                    self.len += 1;
                    self.echo_byte(byte);
                }
            }
        }
    }

    fn echo_byte(&self, byte: u8) {
        let color_code = get_color_code(Color::White, Color::Black);
        vga_buffer::write_byte(byte, color_code);
    }

    fn clear_buffer(&mut self) {
        self.len = 0;
    }

    fn handle_backspace(&mut self) {
        if self.len > 0 {
            self.len -= 1;
            self.erase_last_char();
        }
    }

    fn erase_last_char(&self) {
        let color_code = get_color_code(Color::White, Color::Black);
        vga_buffer::backspace(color_code);
    }

    fn process_buffer(&mut self) {
        if self.len == 0 {
            return;
        }

        let slice = &self.buffer[..self.len];
        let line = match core::str::from_utf8(slice) {
            Ok(line) => line.trim(),
            Err(_) => {
                self.print_error("Input is not valid UTF-8\n");
                return;
            }
        };

        if line.is_empty() {
            return;
        }

        match command_parser(line) {
            Ok(command) => match self.execute_command(command) {
                CommandResult::Success => {}
                CommandResult::Exit => self.shutdown(),
                CommandResult::Error(err) => self.report_error(err),
            },
            Err(err) => self.report_error(err),
        }
    }

    fn execute_command<'a>(&self, command: CommandToExecute<'a>) -> CommandResult {
        match command {
            CommandToExecute::Greet { name } => {
                let mut buf = [0u8; 64];
                let mut len = 0;

                for &byte in b"Hello, " {
                    buf[len] = byte;
                    len += 1;
                }

                for &byte in name.as_bytes() {
                    if len >= buf.len() {
                        break;
                    }
                    buf[len] = byte;
                    len += 1;
                }

                if len + 2 <= buf.len() {
                    buf[len] = b'!';
                    len += 1;
                    buf[len] = b'\n';
                    len += 1;
                }

                if let Ok(msg) = core::str::from_utf8(&buf[..len]) {
                    print_command_output(msg);
                }
                CommandResult::Success
            }
            CommandToExecute::Sum { a, b } => {
                let mut tmp_buf = [0u8; 32];
                match int_to_str_buf(a + b, &mut tmp_buf) {
                    Ok(output) => print_command_output(output),
                    Err(error) => self.print_error(error.as_str()),
                }
                print_command_output("\n");
                CommandResult::Success
            }
            CommandToExecute::Diff { a, b } => {
                let mut tmp_buf = [0u8; 32];
                match int_to_str_buf(a - b, &mut tmp_buf) {
                    Ok(output) => print_command_output(output),
                    Err(error) => self.print_error(error.as_str()),
                }
                print_command_output("\n");
                CommandResult::Success
            }
            CommandToExecute::Min { a, b } => {
                let mut tmp_buf = [0u8; 32];
                match int_to_str_buf(core::cmp::min(a, b), &mut tmp_buf) {
                    Ok(output) => print_command_output(output),
                    Err(error) => self.print_error(error.as_str()),
                }
                print_command_output("\n");
                CommandResult::Success
            }
            CommandToExecute::Max { a, b } => {
                let mut tmp_buf = [0u8; 32];
                match int_to_str_buf(core::cmp::max(a, b), &mut tmp_buf) {
                    Ok(output) => print_command_output(output),
                    Err(error) => self.print_error(error.as_str()),
                }
                print_command_output("\n");
                CommandResult::Success
            }
            CommandToExecute::Exit => {
                print_command_output("Exiting shell...\n");
                CommandResult::Exit
            }
        }
    }

    fn report_error(&self, error: CommandError) {
        match error {
            CommandError::UnknownCommand => self.print_error("Unknown command\n"),
            CommandError::InvalidArguments => {
                self.print_error("Invalid arguments for command\n");
            }
            CommandError::ExecutionFailed => self.print_error("Command failed to execute\n"),
            CommandError::Parse(parse_error) => {
                self.print_error(parse_error.as_str());
                self.print_error("\n");
            }
        }
    }

    fn print_error(&self, message: &str) {
        let color_code = get_color_code(Color::LightRed, Color::Black);
        print_string(message, color_code);
    }

    fn shutdown(&self) -> ! {
        loop {
            spin_loop();
        }
    }
}

pub fn bootstrap(os_version: &str) -> ! {
    print_hello();
    print_os_version(os_version);
    print_string("\n", get_color_code(Color::White, Color::Black));
    let mut shell = Shell::new();
    shell.run()
}
