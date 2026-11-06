//! Core types for assembling the Artemisia desktop shell.
//!
//! This crate intentionally does not depend on GTK or MangoWC. It translates
//! the loaded desktop configuration into a UI-independent shell plan. The app
//! crate can later pass that plan to the GTK UI layer and compositor adapter.

use config::Config;
pub use config::{Position, StatusItem, Widget};

/// The horizontal region in which a panel module should be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Start,
    Center,
    End,
}

/// A configured module and the panel region where it belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModulePlacement {
    pub widget: Widget,
    pub slot: Slot,
}

/// UI-independent description of the shell's top-level panel.
///
/// This is an intermediate representation: it keeps configuration parsing
/// separate from GTK widget construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellPlan {
    pub position: Position,
    pub modules: Vec<ModulePlacement>,
    pub clock_format: String,
    pub status_items: Vec<StatusItem>,
}

impl From<&Config> for ShellPlan {
    fn from(config: &Config) -> Self {
        let mut modules = Vec::with_capacity(
            config.bar.left.len() + config.bar.center.len() + config.bar.right.len(),
        );

        modules.extend(
            config
                .bar
                .left
                .iter()
                .copied()
                .map(|widget| ModulePlacement {
                    widget,
                    slot: Slot::Start,
                }),
        );
        modules.extend(
            config
                .bar
                .center
                .iter()
                .copied()
                .map(|widget| ModulePlacement {
                    widget,
                    slot: Slot::Center,
                }),
        );
        modules.extend(
            config
                .bar
                .right
                .iter()
                .copied()
                .map(|widget| ModulePlacement {
                    widget,
                    slot: Slot::End,
                }),
        );

        Self {
            position: config.bar.position,
            modules,
            clock_format: config.clock.format.clone(),
            status_items: config.status.items.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Bar, Clock, Status};

    #[test]
    fn creates_shell_plan_from_config() {
        let config = Config {
            bar: Bar {
                position: Position::Top,
                left: vec![Widget::Launcher, Widget::Tags],
                center: vec![Widget::Clock],
                right: vec![Widget::Status],
            },
            clock: Clock {
                format: "%H:%M".to_owned(),
            },
            status: Status {
                items: vec![StatusItem::Network, StatusItem::Volume],
            },
        };

        let plan = ShellPlan::from(&config);

        assert_eq!(plan.position, Position::Top);
        assert_eq!(plan.modules.len(), 4);
        assert_eq!(plan.modules[0].slot, Slot::Start);
        assert_eq!(plan.modules[2].slot, Slot::Center);
        assert_eq!(plan.modules[3].slot, Slot::End);
        assert_eq!(plan.clock_format, "%H:%M");
        assert_eq!(
            plan.status_items,
            vec![StatusItem::Network, StatusItem::Volume]
        );
    }
}
