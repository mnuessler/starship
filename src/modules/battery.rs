use super::{Context, Module, RootModuleConfig};
use crate::configs::battery::BatteryConfig;

/// Creates a module for the battery percentage and charging state
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    // TODO: Update when v1.0 printing refactor is implemented to only
    // print escapes in a prompt context.
    let shell = std::env::var("STARSHIP_SHELL").unwrap_or_default();
    let percentage_char = match shell.as_str() {
        "zsh" => "%%", // % is an escape in zsh, see PROMPT in `man zshmisc`
        _ => "%",
    };

    let battery_status = get_battery_status()?;
    let BatteryStatus { state, percentage } = battery_status;

    let mut module = context.new_module("battery");
    let battery_config: BatteryConfig = BatteryConfig::try_load(module.config);

    // Parse config under `display`
    let display_styles = &battery_config.display;
    let display_style = display_styles
        .iter()
        .find(|display_style| percentage <= display_style.threshold as f32);

    if let Some(display_style) = display_style {
        // Set style based on percentage
        module.set_style(display_style.style);
        module.get_prefix().set_value("");

        match state {
            battery::State::Full => {
                module.create_segment("full_symbol", &battery_config.full_symbol);
            }
            battery::State::Charging => {
                module.create_segment("charging_symbol", &battery_config.charging_symbol);
            }
            battery::State::Discharging => {
                module.create_segment("discharging_symbol", &battery_config.discharging_symbol);
            }
            battery::State::Unknown => {
                log::debug!("Unknown detected");
                if let Some(unknown_symbol) = battery_config.unknown_symbol {
                    module.create_segment("unknown_symbol", &unknown_symbol);
                }
            }
            battery::State::Empty => {
                if let Some(empty_symbol) = battery_config.empty_symbol {
                    module.create_segment("empty_symbol", &empty_symbol);
                }
            }
            _ => {
                log::debug!("Unhandled battery state `{}`", state);
                return None;
            }
        }

        let mut percent_string = Vec::<String>::with_capacity(2);
        // Round the percentage to a whole number
        percent_string.push(percentage.round().to_string());
        percent_string.push(percentage_char.to_string());
        module.create_segment(
            "percentage",
            &battery_config
                .percentage
                .with_value(percent_string.join("").as_ref()),
        );

        Some(module)
    } else {
        None
    }
}

fn get_battery_status() -> Option<BatteryStatus> {
    let battery_manager = battery::Manager::new().ok()?;
    match battery_manager.batteries().ok()?.next() {
        Some(Ok(battery)) => {
            log::debug!("Battery found: {:?}", battery);
            let battery_status = BatteryStatus {
                percentage: battery.state_of_charge().value * 100.0,
                state: battery.state(),
            };

            Some(battery_status)
        }
        Some(Err(e)) => {
            log::debug!("Unable to access battery information:\n{}", &e);
            None
        }
        None => {
            log::debug!("No batteries found");
            None
        }
    }
}

struct BatteryStatus {
    percentage: f32,
    state: battery::State,
}
