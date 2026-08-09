pub mod command;
pub mod parser;

use std::io::{self, BufRead};

use crate::{
    cdh::command::{Command, Target},
    kernel::{event::Event, event_bus::EventBus},
};

/// Map a parsed command to the event it triggers. Unknown -> None.
pub fn command_to_event(command: Command) -> Option<Event> {
    match command {
        Command::On(Target::Stabilization) => Some(Event::StabilizationEnabled),
        Command::Off(Target::Stabilization) => Some(Event::StabilizationDisabled),
        Command::On(Target::Damping) => Some(Event::DampingEnabled),
        Command::Off(Target::Damping) => Some(Event::DampingDisabled),
        _ => None,
    }
}

/// Parse one input line and publish the resulting event to the bus.
/// Returns true if the input produced a published event.
pub fn publish_command(input: &str, bus: &EventBus) -> bool {
    match parser::parse(input) {
        Some(command) => match command_to_event(command) {
            Some(event) => {
                bus.publish(event);
                true
            }
            None => false,
        },
        None => false,
    }
}

/// Read commands from stdin line by line until EOF, parsing each line and
/// publishing the resulting event to the bus. Blocking: run on a dedicated
/// thread (see `main`). Blank lines are skipped.
pub fn read_stdin(bus: &EventBus) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if publish_command(trimmed, bus) {
            println!("CMD OK: {trimmed}");
        } else {
            println!("CMD UNKNOWN: {trimmed}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_to_event_maps_stab_on_off() {
        assert_eq!(
            command_to_event(Command::On(Target::Stabilization)),
            Some(Event::StabilizationEnabled)
        );
        assert_eq!(
            command_to_event(Command::Off(Target::Stabilization)),
            Some(Event::StabilizationDisabled)
        );
        assert_eq!(
            command_to_event(Command::On(Target::Damping)),
            Some(Event::DampingEnabled)
        );
        assert_eq!(
            command_to_event(Command::Off(Target::Damping)),
            Some(Event::DampingDisabled)
        );
    }

    #[test]
    fn on_damp_publishes_damping_enabled() {
        let bus = EventBus::new();
        assert!(publish_command("on damp", &bus));
        assert_eq!(bus.drain(), vec![Event::DampingEnabled]);
    }

    #[test]
    fn off_damp_publishes_damping_disabled() {
        let bus = EventBus::new();
        assert!(publish_command("off damp", &bus));
        assert_eq!(bus.drain(), vec![Event::DampingDisabled]);
    }

    #[test]
    fn on_stab_publishes_enabled_event() {
        let bus = EventBus::new();
        assert!(publish_command("on stab", &bus));
        assert_eq!(bus.drain(), vec![Event::StabilizationEnabled]);
    }

    #[test]
    fn off_stab_publishes_disabled_event() {
        let bus = EventBus::new();
        assert!(publish_command("off stab", &bus));
        assert_eq!(bus.drain(), vec![Event::StabilizationDisabled]);
    }

    #[test]
    fn unknown_input_publishes_nothing() {
        let bus = EventBus::new();
        assert!(!publish_command("do a barrel roll", &bus));
        assert!(bus.drain().is_empty());
    }

    #[test]
    fn garbage_input_publishes_nothing() {
        let bus = EventBus::new();
        // split_whitespace collapses extra spaces, so use real garbage
        assert!(!publish_command("on stabb", &bus));
        assert!(!publish_command("en stab", &bus));
        assert!(!publish_command("", &bus));
        assert!(bus.drain().is_empty());
    }
}