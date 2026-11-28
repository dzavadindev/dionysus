mod animation;
pub mod window;

pub fn load_styles() {
    relm4::set_global_css(include_str!("style.css"));
}
