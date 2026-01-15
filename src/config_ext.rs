//! App-specific configuration extensions.
//!
//! This module provides hotkey support on top of the core Config.

use std::sync::Arc;

use std::sync::RwLock;

use global_hotkey::hotkey::{HotKey, Modifiers};

use crate::Config;

/// Default hotkey: Meta+Shift+Semicolon
pub fn default_hotkey() -> HotKey {
    HotKey::new(
        Some(Modifiers::META | Modifiers::SHIFT),
        global_hotkey::hotkey::Code::Semicolon,
    )
}

/// Extension trait for Config to handle hotkeys.
pub trait ConfigExt {
    /// Get the hotkey, parsing from config or using default.
    fn hotkey(&self) -> HotKey;
}

impl ConfigExt for Config {
    fn hotkey(&self) -> HotKey {
        // For now, always use default hotkey
        // TODO: Parse from config.hotkey string if present
        default_hotkey()
    }
}

impl ConfigExt for Arc<RwLock<Config>> {
    fn hotkey(&self) -> HotKey {
        self.read().unwrap().hotkey()
    }
}
