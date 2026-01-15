//! Tray icon management.

use std::path::Path;
use std::sync::LazyLock;

use tray_icon::Icon;

use crate::color::{self, Color};
use crate::MicState;

pub const ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");

fn load_color(color: Color) -> Icon {
    const CHOOSE_FN: fn(Color) -> (u8, u8, u8) = |color| color.accessible_dark;
    load_icon(ICON_PATH, Some(CHOOSE_FN(color)))
}

static IDLE: LazyLock<Icon> = LazyLock::new(|| load_color(color::WHITE));
static WAITING: LazyLock<Icon> = LazyLock::new(|| load_color(color::YELLOW));
static ACTIVE: LazyLock<Icon> = LazyLock::new(|| load_color(color::GREEN));
static WORKING: LazyLock<Icon> = LazyLock::new(|| load_color(color::YELLOW));

/// Extension trait to get icons for MicState.
pub trait MicStateIcon {
    /// Get the tray icon for this state.
    fn icon(&self) -> Icon;
}

impl MicStateIcon for MicState {
    fn icon(&self) -> Icon {
        match self {
            MicState::Activating => WAITING.clone(),
            MicState::Active => ACTIVE.clone(),
            MicState::Idle => IDLE.clone(),
            MicState::Processing => WORKING.clone(),
        }
    }
}

fn load_icon(path: impl AsRef<Path>, recolor: Option<(u8, u8, u8)>) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let mut image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();

        if let Some((r, g, b)) = recolor {
            for pixel in image.pixels_mut() {
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
            }
        }

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
