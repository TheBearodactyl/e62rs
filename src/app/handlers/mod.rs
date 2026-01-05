//! logic handler stuff
mod main_menu;
mod search;

use {super::interrupt::InterruptHandler, crate::ui::E6Ui};

/// logic handlers
pub struct Handlers {
    /// the UI handler
    pub(crate) ui: E6Ui,
    /// the interruption handler
    pub(crate) interrupt: InterruptHandler,
}

impl Handlers {
    /// make a new set of handlers
    pub fn new(ui: E6Ui, interrupt: InterruptHandler) -> Self {
        Self { ui, interrupt }
    }

    /// see [`InterruptHandler::check_and_reset`]
    pub fn was_interrupted(&self) -> bool {
        self.interrupt.check_and_reset()
    }
}
