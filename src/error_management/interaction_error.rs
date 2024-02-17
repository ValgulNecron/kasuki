use std::fmt;
use crate::error_management::autocomplete_error::AutocompleteError;
use crate::error_management::command_error::CommandError;
use crate::error_management::component_error::ComponentError;
use crate::error_management::differed_command_error::DifferedCommandError;

#[derive(Debug, Clone)]
pub enum InteractionError {
    Command(CommandError),
    DifferedCommand(DifferedCommandError),
    Component(ComponentError),
    Autocomplete(AutocompleteError),
}

impl fmt::Display for InteractionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InteractionError::Command(command_error) => write!(f, "Command error: {}", command_error),
            InteractionError::DifferedCommand(differed_command_error) => write!(f, "Differed command error: {}", differed_command_error),
            InteractionError::Component(component_error) => write!(f, "Component error: {}", component_error),
            InteractionError::Autocomplete(autocomplete_error) => write!(f, "Autocomplete error: {}", autocomplete_error),
        }
    }
}