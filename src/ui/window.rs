use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerKey};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

pub fn build_launcher_window(app: &Application) -> ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);

    window.set_decorated(false);
    window.set_resizable(false);
    window.set_default_size(700, 420);

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("dionysus"));

    let controller = EventControllerKey::new();
    controller.connect_key_pressed({
        let window = window.clone();
        move |_, key, _keycode, _state| {
            if key == gdk::Key::Escape {
                window.close();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    });
    window.add_controller(controller);

    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.present();

    window
}
