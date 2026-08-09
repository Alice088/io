use crate::cdh::command::{Command, Target};

pub fn parse(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    match parts.as_slice() {
        ["on", "stab"] => {
            Some(Command::On(Target::Stabilization))
        }

        ["off", "stab"] => {
            Some(Command::Off(Target::Stabilization))
        }

        ["on", "damp"] => {
            Some(Command::On(Target::Damping))
        }

        ["off", "damp"] => {
            Some(Command::Off(Target::Damping))
        }

        _ => None,
    }
}