use std::path::Path;
use std::sync::LazyLock;

use tray_icon::Icon;

const ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");
const COLOR_WAITING: (u8, u8, u8) = (255, 223, 0);
const COLOR_ACTIVE: (u8, u8, u8) = (0, 255, 0);

static IDLE: LazyLock<Icon> = LazyLock::new(|| load_icon(ICON_PATH, None));
static WAITING: LazyLock<Icon> = LazyLock::new(|| load_icon(ICON_PATH, Some(COLOR_WAITING)));
static ACTIVE: LazyLock<Icon> = LazyLock::new(|| load_icon(ICON_PATH, Some(COLOR_ACTIVE)));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicState {
    Activating,
    Active,
    Inactive,
}

impl MicState {
    pub fn icon(&self) -> Icon {
        match self {
            MicState::Activating => WAITING.clone(),
            MicState::Active => ACTIVE.clone(),
            MicState::Inactive => IDLE.clone(),
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
