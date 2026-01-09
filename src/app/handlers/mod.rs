//! logic handler stuff
use {super::interrupt::InterruptHandler, crate::ui::E6Ui};

pub mod main_menu;
pub mod search;

/// logic handlers
pub struct Handlers {
    /// the UI handler
    pub(crate) ui: E6Ui,
    /// the interruption handler
    pub(crate) interrupt: InterruptHandler,
}

impl Handlers {
    /// make a new set of handlers
    ///
    /// # Arguments
    ///
    /// * `interrupt` - the interruption handler to use
    pub const fn new(ui: E6Ui, interrupt: InterruptHandler) -> Self {
        Self { ui, interrupt }
    }

    /// see [`InterruptHandler::check_and_reset`]
    pub fn was_interrupted(&self) -> bool {
        self.interrupt.check_and_reset()
    }
}
