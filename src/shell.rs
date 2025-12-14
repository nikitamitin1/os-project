use crate::{
    history::InputHistory,
    keyboard,
    parser::{int_to_str_buf, parse_int_from_str, ParseError},
    simple_string::FixedString,
    vga_buffer::{self, get_color_code, Color},
};
use core::hint::spin_loop;

const INPUT_BUFFER_LEN: usize = 128;

pub struct Shell {
    buffer: [u8; INPUT_BUFFER_LEN],
    len: usize,
    extended_prefix: bool,
    history: InputHistory,
    saved_line: FixedString<INPUT_BUFFER_LEN>,
    saved_line_active: bool,
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

enum HistoryKey {
    Up,
    Down,
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
            extended_prefix: false,
            history: InputHistory::new(),
            saved_line: FixedString::new(),
            saved_line_active: false,
        }
    }

    fn run(&mut self) -> ! {
        print_prompt();
        loop {
            if let Some(scancode) = keyboard::pop_scancode() {
                self.handle_scancode(scancode);
            } else {
                x86_64::instructions::hlt();
            }
        }
    }

    fn handle_scancode(&mut self, scancode: u8) {
        if self.extended_prefix {
            self.extended_prefix = false;
            match scancode {
                0x48 => self.handle_history_navigation(HistoryKey::Up),
                0x50 => self.handle_history_navigation(HistoryKey::Down),
                _ => {}
            }
            return;
        }

        if scancode == 0xE0 {
            self.extended_prefix = true;
            return;
        }

        if let Some(byte) = vga_buffer::scancode_to_ascii(scancode) {
            self.handle_input_byte(byte);
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
                    self.reset_history_tracking();
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
            self.reset_history_tracking();
        }
    }

    fn handle_history_navigation(&mut self, key: HistoryKey) {
        if self.history.is_empty() {
            return;
        }

        if self.history.is_at_current() && !self.saved_line_active {
            self.save_current_line();
        }

        match key {
            HistoryKey::Up => {
                if let Some(line) = self
                    .history
                    .previous()
                    .map(|line| Self::own_line(line))
                {
                    self.replace_buffer_with(line.as_str());
                } else if let Some(line) = self.history.latest().map(Self::own_line) {
                    self.replace_buffer_with(line.as_str());
                }
            }
            HistoryKey::Down => {
                if let Some(line) = self.history.next().map(Self::own_line) {
                    self.replace_buffer_with(line.as_str());
                } else {
                    self.restore_saved_line();
                }
            }
        }
    }

    fn erase_last_char(&self) {
        let color_code = get_color_code(Color::White, Color::Black);
        vga_buffer::backspace(color_code);
    }

    fn save_current_line(&mut self) {
        let owned = {
            let current = self.current_line();
            Self::own_line(current)
        };
        self.saved_line = owned;
        self.saved_line_active = true;
    }

    fn restore_saved_line(&mut self) {
        if self.saved_line_active {
            let owned = {
                let saved = self.saved_line.as_str();
                Self::own_line(saved)
            };
            self.replace_buffer_with(owned.as_str());
        } else {
            self.replace_buffer_with("");
        }
        self.saved_line.clear();
        self.saved_line_active = false;
        self.history.reset_navigation();
    }

    fn replace_buffer_with(&mut self, line: &str) {
        self.clear_displayed_buffer();
        for &byte in line.as_bytes() {
            if self.len < self.buffer.len() {
                self.buffer[self.len] = byte;
                self.len += 1;
                self.echo_byte(byte);
            }
        }
    }

    fn clear_displayed_buffer(&mut self) {
        while self.len > 0 {
            self.len -= 1;
            self.erase_last_char();
        }
    }

    fn current_line(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("")
    }

    fn reset_history_tracking(&mut self) {
        self.history.reset_navigation();
        self.saved_line.clear();
        self.saved_line_active = false;
    }

    fn own_line(line: &str) -> FixedString<INPUT_BUFFER_LEN> {
        let mut owned = FixedString::<INPUT_BUFFER_LEN>::new();
        let _ = owned.push_str(line);
        owned
    }

    fn process_buffer(&mut self) {
        if self.len == 0 {
            return;
        }

        let owned_line = {
            let slice = &self.buffer[..self.len];
            let raw = match core::str::from_utf8(slice) {
                Ok(line) => line,
                Err(_) => {
                    self.print_error("Input is not valid UTF-8\n");
                    return;
                }
            };

            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return;
            }

            Self::own_line(trimmed)
        };

        let line = owned_line.as_str();
        self.history.push(line);
        self.reset_history_tracking();

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
                let mut msg = FixedString::<64>::new();
                let _ = msg.push_str("Hello, ");
                let _ = msg.push_str(name);
                let _ = msg.push_str("!\n");
                print_command_output(msg.as_str());
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
