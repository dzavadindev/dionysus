mod animation;
mod style;
pub mod window;

pub fn load_styles(config: &dionysus::config::Config) {
    relm4::set_global_css(&style::css(config));
}
