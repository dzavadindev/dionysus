#[path = "../src/ui/style.rs"]
mod style;

use dionysus::config::{ColorsConfig, Config, Rgb};
use style::{Palette, UiMetrics};

fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb { red, green, blue }
}

#[test]
fn default_palette_preserves_spotlight_colors() {
    let palette = Palette::from_config(&ColorsConfig::default());
    assert_eq!(palette.primary, rgb(30, 33, 42));
    assert_eq!(palette.secondary, rgb(242, 243, 247));
    assert_eq!(palette.tertiary, rgb(70, 90, 129));
    assert_eq!(palette.selected_text, palette.secondary);
    let css = style::css(&Config::default());
    assert!(css.contains("@define-color dy-panel rgba(30, 33, 42, 0.97)"));
    assert!(css.contains("@define-color dy-focus #465a81"));
    assert!(css.contains("entry.dionysus-prompt { min-height: 48px;"));
    assert!(css.contains("list.dionysus-list row { min-height: 36px;"));
}

#[test]
fn derived_text_contrasts_with_light_and_dark_primary() {
    let light = Palette::from_config(&ColorsConfig {
        primary: Some(rgb(240, 240, 240)),
        ..ColorsConfig::default()
    });
    assert_eq!(light.secondary, rgb(28, 31, 38));
    assert!(light.tertiary.red < light.primary.red);

    let dark = Palette::from_config(&ColorsConfig {
        primary: Some(rgb(16, 16, 16)),
        ..ColorsConfig::default()
    });
    assert_eq!(dark.secondary, rgb(242, 243, 247));
    assert!(dark.tertiary.red > dark.primary.red);
}

#[test]
fn explicit_colors_win_and_focus_text_stays_readable() {
    let palette = Palette::from_config(&ColorsConfig {
        primary: Some(rgb(10, 20, 30)),
        secondary: Some(rgb(250, 250, 250)),
        tertiary: Some(rgb(255, 255, 255)),
    });
    assert_eq!(palette.primary, rgb(10, 20, 30));
    assert_eq!(palette.secondary, rgb(250, 250, 250));
    assert_eq!(palette.tertiary, rgb(255, 255, 255));
    assert_eq!(palette.selected_text, rgb(28, 31, 38));

    let derived = Palette::from_config(&ColorsConfig {
        primary: Some(rgb(10, 20, 30)),
        secondary: Some(rgb(250, 250, 250)),
        tertiary: None,
    });
    assert_eq!(derived.tertiary, rgb(70, 78, 85));
}

#[test]
fn geometry_and_type_scales_are_independent() {
    let mut config = Config::default();
    config.ui.h_scale = 2.0;
    config.ui.v_scale = 0.5;
    config.ui.item_scale = 1.5;
    config.ui.font_scale = 3.0;
    let metrics = UiMetrics::from_config(&config.ui);
    assert_eq!(metrics.window_width, 1400);
    assert_eq!(metrics.window_height, 200);
    assert_eq!(metrics.panel_vertical_space, 35);
    let css = style::css(&config);
    assert!(css.contains("entry.dionysus-prompt { min-height: 72px;"));
    assert!(css.contains("font-size: 60px;"));
    assert!(css.contains("list.dionysus-list row { min-height: 54px;"));
    assert!(css.contains("font-size: 45px;"));
    assert!(css.contains("label.dionysus-mode-indicator { margin-right: 21px;"));
}

#[test]
fn parsed_ini_values_feed_palette_and_layout() {
    let config = dionysus::config::parse(
        "[ui]\nh_scale=1.5\nv_scale=0.75\nitem_scale=2\nfont_scale=1.25\n[colors]\nprimary=#f0f0f0",
    )
    .unwrap();
    let metrics = UiMetrics::from_config(&config.ui);
    assert_eq!(metrics.window_width, 1050);
    assert_eq!(metrics.window_height, 300);
    let css = style::css(&config);
    assert!(css.contains("@define-color dy-panel rgba(240, 240, 240, 0.97)"));
    assert!(css.contains("@define-color dy-text #1c1f26"));
    assert!(css.contains("entry.dionysus-prompt { min-height: 96px;"));
    assert!(css.contains("font-size: 25px;"));
}

#[test]
fn extreme_finite_scales_do_not_overflow_gtk_lengths() {
    let mut config = Config::default();
    config.ui.h_scale = f64::MAX;
    config.ui.v_scale = f64::MAX;
    config.ui.item_scale = f64::MAX;
    config.ui.font_scale = f64::MAX;
    let metrics = UiMetrics::from_config(&config.ui);
    assert_eq!(metrics.window_width, 8192);
    assert_eq!(metrics.window_height, 8192);
    assert!(style::css(&config).contains("font-size: 256px;"));
}
