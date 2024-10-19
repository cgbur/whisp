use std::path::Path;
use std::sync::LazyLock;

const COLOR_ACTIVATING: (u8, u8, u8) = (255, 223, 0);
const COLOR_ACTIVE: (u8, u8, u8) = (0, 255, 0);
const ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");

static ICON: LazyLock<tray_icon::Icon> = LazyLock::new(|| load_icon(ICON_PATH, None));
static ICON_ACTIVATING: LazyLock<tray_icon::Icon> =
    LazyLock::new(|| load_icon(ICON_PATH, Some(COLOR_ACTIVATING)));
static ICON_ACTIVE: LazyLock<tray_icon::Icon> =
    LazyLock::new(|| load_icon(ICON_PATH, Some(COLOR_ACTIVE)));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicState {
    Activating,
    Active,
    Inactive,
}

impl MicState {
    pub fn icon(&self) -> tray_icon::Icon {
        match self {
            MicState::Activating => ICON_ACTIVATING.clone(),
            MicState::Active => ICON_ACTIVE.clone(),
            MicState::Inactive => ICON.clone(),
        }
    }
}

fn load_icon(path: impl AsRef<Path>, recolor: Option<(u8, u8, u8)>) -> tray_icon::Icon {
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
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
