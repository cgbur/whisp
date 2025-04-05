//! These are the MacOS system colors from the apple website. Dillegently copied
//! by an intern here for easy access.
//!
//! https://developer.apple.com/design/human-interface-guidelines/color#macOS-system-colors

#![allow(unused)]

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub default_light: (u8, u8, u8),
    pub default_dark: (u8, u8, u8),
    pub accessible_light: (u8, u8, u8),
    pub accessible_dark: (u8, u8, u8),
}

pub static WHITE: Color = Color {
    default_light: (255, 255, 255),
    default_dark: (255, 255, 255),
    accessible_light: (255, 255, 255),
    accessible_dark: (255, 255, 255),
};

pub static RED: Color = Color {
    default_light: (255, 59, 48),
    default_dark: (255, 69, 58),
    accessible_light: (215, 0, 21),
    accessible_dark: (255, 105, 97),
};

pub static ORANGE: Color = Color {
    default_light: (255, 149, 0),
    default_dark: (255, 159, 10),
    accessible_light: (201, 52, 0),
    accessible_dark: (255, 179, 64),
};

pub static YELLOW: Color = Color {
    default_light: (255, 204, 0),
    default_dark: (255, 214, 10),
    accessible_light: (160, 90, 0),
    accessible_dark: (255, 212, 38),
};

pub static GREEN: Color = Color {
    default_light: (40, 205, 65),
    default_dark: (50, 215, 75),
    accessible_light: (0, 125, 27),
    accessible_dark: (49, 222, 75),
};

pub static MINT: Color = Color {
    default_light: (0, 199, 190),
    default_dark: (102, 212, 207),
    accessible_light: (12, 129, 123),
    accessible_dark: (102, 212, 207),
};

pub static TEAL: Color = Color {
    default_light: (89, 173, 196),
    default_dark: (106, 196, 220),
    accessible_light: (0, 130, 153),
    accessible_dark: (93, 230, 255),
};

pub static CYAN: Color = Color {
    default_light: (85, 190, 240),
    default_dark: (90, 200, 245),
    accessible_light: (0, 113, 164),
    accessible_dark: (112, 215, 255),
};

pub static BLUE: Color = Color {
    default_light: (0, 122, 255),
    default_dark: (10, 132, 255),
    accessible_light: (0, 64, 221),
    accessible_dark: (64, 156, 255),
};

pub static INDIGO: Color = Color {
    default_light: (88, 86, 214),
    default_dark: (94, 92, 230),
    accessible_light: (54, 52, 163),
    accessible_dark: (125, 122, 255),
};

pub static PURPLE: Color = Color {
    default_light: (175, 82, 222),
    default_dark: (191, 90, 242),
    accessible_light: (137, 68, 171),
    accessible_dark: (218, 143, 255),
};

pub static PINK: Color = Color {
    default_light: (255, 45, 85),
    default_dark: (255, 55, 95),
    accessible_light: (211, 15, 69),
    accessible_dark: (255, 100, 130),
};

pub static BROWN: Color = Color {
    default_light: (162, 132, 94),
    default_dark: (172, 142, 104),
    accessible_light: (127, 101, 69),
    accessible_dark: (181, 148, 105),
};

pub static GRAY: Color = Color {
    default_light: (142, 142, 147),
    default_dark: (152, 152, 157),
    accessible_light: (105, 105, 110),
    accessible_dark: (152, 152, 157),
};
