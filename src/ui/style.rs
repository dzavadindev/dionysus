//! Default palette and CSS overrides generated from validated launcher settings.

use dionysus::config::{ColorsConfig, Config, Rgb, UiConfig};

const DEFAULT_PRIMARY: Rgb = Rgb {
    red: 30,
    green: 33,
    blue: 42,
};
const DEFAULT_SECONDARY: Rgb = Rgb {
    red: 242,
    green: 243,
    blue: 247,
};
const DEFAULT_TERTIARY: Rgb = Rgb {
    red: 70,
    green: 90,
    blue: 129,
};
const LIGHT_TEXT: Rgb = DEFAULT_SECONDARY;
const DARK_TEXT: Rgb = Rgb {
    red: 28,
    green: 31,
    blue: 38,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UiMetrics {
    pub window_width: i32,
    pub window_height: i32,
    pub panel_vertical_space: i32,
}

impl UiMetrics {
    pub fn from_config(ui: &UiConfig) -> Self {
        let item = ui.item_scale;
        // Panel padding and borders plus the results separator's top margin/border.
        let panel_vertical_space = 2 * item_px(8, item) + 2 + item_px(5, item) + 1;
        Self {
            window_width: scaled_px(700, ui.h_scale, 8192),
            window_height: scaled_px(400, ui.v_scale, 8192),
            panel_vertical_space,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Palette {
    pub primary: Rgb,
    pub secondary: Rgb,
    pub tertiary: Rgb,
    pub selected_text: Rgb,
}

impl Palette {
    pub fn from_config(colors: &ColorsConfig) -> Self {
        let primary = colors.primary.unwrap_or(DEFAULT_PRIMARY);
        let secondary = colors.secondary.unwrap_or_else(|| {
            if colors.primary.is_none() {
                DEFAULT_SECONDARY
            } else {
                readable_on(primary)
            }
        });
        let tertiary = colors.tertiary.unwrap_or_else(|| {
            if colors.primary.is_none() {
                DEFAULT_TERTIARY
            } else {
                blend(primary, secondary, 0.25)
            }
        });
        let selected_text = if contrast(tertiary, secondary) >= 4.5 {
            secondary
        } else {
            readable_on(tertiary)
        };
        Self {
            primary,
            secondary,
            tertiary,
            selected_text,
        }
    }
}

// -------------------------------------------------------
// scale

fn scaled_px(base: i32, scale: f64, maximum: i32) -> i32 {
    (f64::from(base) * scale)
        .round()
        .clamp(1.0, f64::from(maximum)) as i32
}

fn item_px(base: i32, scale: f64) -> i32 {
    scaled_px(base, scale, 2048)
}

fn font_px(base: i32, scale: f64) -> i32 {
    scaled_px(base, scale, 256)
}

// -------------------------------------------------------
// color

fn luminance(color: Rgb) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.red) + 0.7152 * channel(color.green) + 0.0722 * channel(color.blue)
}

fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (lighter, darker) = (luminance(a), luminance(b));
    (lighter.max(darker) + 0.05) / (lighter.min(darker) + 0.05)
}

fn readable_on(background: Rgb) -> Rgb {
    if contrast(background, LIGHT_TEXT) >= contrast(background, DARK_TEXT) {
        LIGHT_TEXT
    } else {
        DARK_TEXT
    }
}

fn blend(a: Rgb, b: Rgb, toward_b: f64) -> Rgb {
    let mix = |start: u8, end: u8| {
        (f64::from(start) * (1.0 - toward_b) + f64::from(end) * toward_b).round() as u8
    };
    Rgb {
        red: mix(a.red, b.red),
        green: mix(a.green, b.green),
        blue: mix(a.blue, b.blue),
    }
}

fn rgba(color: Rgb, alpha: f64) -> String {
    format!(
        "rgba({}, {}, {}, {alpha})",
        color.red, color.green, color.blue
    )
}

// -------------------------------------------------------
// api

/// Combines the global stylesheet with computed palette tokens and size overrides.
/// CSS names remain in `style.css`; this fragment supplies values.
pub(crate) fn css(config: &Config) -> String {
    let colors = Palette::from_config(&config.colors);
    let item = config.ui.item_scale;
    let font = config.ui.font_scale;
    let indicator_font = font_px(11, font);
    let badge_end = item_px(14, item);
    let badge_pad = item_px(8, item);
    let prompt_end = item_px(88, item).max(indicator_font * 5 + badge_pad * 2 + badge_end);

    format!(
        "@define-color dy-panel {panel};\n\
         @define-color dy-text {text};\n\
         @define-color dy-border {border};\n\
         @define-color dy-placeholder {placeholder};\n\
         @define-color dy-indicator-bg {indicator_bg};\n\
         @define-color dy-indicator-text {indicator_text};\n\
         @define-color dy-separator {separator};\n\
         @define-color dy-row-text {row_text};\n\
         @define-color dy-hover {hover};\n\
         @define-color dy-focus {focus};\n\
         @define-color dy-focus-text {focus_text};\n\
         @define-color dy-scrollbar {scrollbar};\n\n\
         {base}\n\n\
         box.dionysus-panel {{ padding: {panel_padding}px; border-radius: {panel_radius}px; }}\n\
         entry.dionysus-prompt {{ min-height: {prompt_height}px; padding: 0 {prompt_end}px 0 {prompt_start}px; border-radius: {prompt_radius}px; font-size: {prompt_font}px; }}\n\
         label.dionysus-mode-indicator {{ margin-right: {badge_end}px; padding: {badge_pad_y}px {badge_pad}px; border-radius: {badge_radius}px; font-size: {indicator_font}px; }}\n\
         scrolledwindow.dionysus-results {{ margin-top: {separator_margin}px; }}\n\
         list.dionysus-list {{ padding-top: {list_padding}px; }}\n\
         list.dionysus-list row {{ min-height: {row_height}px; margin: {row_margin_y}px {row_margin_x}px; padding: 0 {row_padding}px; border-radius: {row_radius}px; font-size: {row_font}px; }}\n\
         scrolledwindow.dionysus-results scrollbar slider {{ min-width: {slider_width}px; border-radius: {slider_radius}px; }}\n",
        panel = rgba(colors.primary, 0.97),
        text = colors.secondary,
        border = rgba(colors.secondary, 0.16),
        placeholder = rgba(colors.secondary, 0.70),
        indicator_bg = rgba(colors.secondary, 0.10),
        indicator_text = rgba(colors.secondary, 0.85),
        separator = rgba(colors.secondary, 0.12),
        row_text = rgba(colors.secondary, 0.94),
        hover = rgba(colors.secondary, 0.07),
        focus = colors.tertiary,
        focus_text = colors.selected_text,
        scrollbar = rgba(colors.secondary, 0.30),
        base = include_str!("style.css"),
        panel_padding = item_px(8, item),
        panel_radius = item_px(18, item),
        prompt_height = item_px(48, item),
        prompt_start = item_px(18, item),
        prompt_radius = item_px(10, item),
        prompt_font = font_px(20, font),
        badge_pad_y = item_px(5, item),
        badge_radius = item_px(6, item),
        separator_margin = item_px(5, item),
        list_padding = item_px(4, item),
        row_height = item_px(36, item),
        row_margin_y = item_px(2, item),
        row_margin_x = item_px(4, item),
        row_padding = item_px(14, item),
        row_radius = item_px(9, item),
        row_font = font_px(15, font),
        slider_width = item_px(4, item),
        slider_radius = item_px(4, item),
    )
}
