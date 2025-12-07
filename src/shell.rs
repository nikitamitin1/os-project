use alloc::format;
use alloc::string::ToString;

pub struct Shell;

struct Command {
    name: &'static str,
    description: &'static str,
}

enum CommandToExecute {
    Greet { name: &'static str },
    Sum { a: i32, b: i32 },
    Diff { a: i32, b: i32 },
    Min { a: i32, b: i32 },
    Max { a: i32, b: i32 },
    Exit,
}

enum CommandError {
    UnknownCommand,
    ExecutionFailed,
}

enum CommandResult {
    Success,
    Error(CommandError),
}

fn print_string(s: &str, color_code: crate::vga_buffer::ColorCode) {
    for byte in s.bytes() {
        crate::vga_buffer::write_byte(byte, color_code);
    }
}

fn print_os_version(os_version: &str) {
    use crate::vga_buffer::{get_color_code, Color};
    let color_code = get_color_code(Color::LightCyan, Color::Black);
    print_string(os_version, color_code);
}

fn print_hello() {
    use crate::vga_buffer::{get_color_code, Color};
    let color_code = get_color_code(Color::LightGreen, Color::Black);
    print_string("Hello, from Shell!\n", color_code);
}

fn print_on_entry(os_version: &str) {
    print_hello();
    print_os_version(os_version);
}

fn print_prompt() {
    use crate::vga_buffer::{get_color_code, Color};
    let color_code = get_color_code(Color::Yellow, Color::Black);
    print_string("shell> ", color_code);
}

pub fn print_command_output(output: &str) {
    use crate::vga_buffer::{get_color_code, Color};
    let color_code = get_color_code(Color::White, Color::Black);
    print_string(output, color_code);
}

impl Shell {
    fn new() -> Self {
        Shell
    }

    fn execute_command(&self, command: CommandToExecute) -> CommandResult {
        match command {
            CommandToExecute::Greet { name } => {
                let output = format!("Hello, {}!\n", name);
                print_command_output(&output);
                CommandResult::Success
            }
            CommandToExecute::Sum { a, b } => {
                let result = a + b;
                let output = result;
                print_command_output(&output);
                CommandResult::Success
            }
            CommandToExecute::Diff { a, b } => {
                let result = a - b;
                let output = format!("Difference: {}\n", result);
                print_command_output(&output);
                CommandResult::Success
            }
            CommandToExecute::Min { a, b } => {
                let result = if a < b { a } else { b };
                let output = format!("Minimum: {}\n", result);
                print_command_output(&output);
                CommandResult::Success
            }
            CommandToExecute::Max { a, b } => {
                let result = if a > b { a } else { b };
                let output = format!("Maximum: {}\n", result);
                print_command_output(&output);
                CommandResult::Success
            }
            CommandToExecute::Exit => {
                print_command_output("Exiting shell...\n");
                CommandResult::Success
            }
        }
    }


}


