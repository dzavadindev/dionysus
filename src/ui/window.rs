use dionysus::core::Launcher;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
    gtk::{self, gdk, gio, glib, prelude::*},
};

#[derive(Debug)]
pub struct WindowModel {
    _launcher: Launcher,
    window: gtk::Window,
}

#[derive(Debug)]
pub enum WindowInput {
    ToggleRequested,
    HideRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for WindowModel {
    type Init = Launcher;

    type Input = WindowInput;
    type Output = ();

    view! {
        #[root]
        gtk::Window {
            set_default_width: 700,
            set_default_height: 400,
        }
    }

    fn init(
        launcher: Self::Init,
        root: gtk::Window,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        init_layer_shell(&root);
        init_key_handler(&root, &sender);

        let toggle_action = gio::SimpleAction::new("toggle", None);
        toggle_action.connect_activate({
            let sender = sender.clone();
            move |_, _| sender.input(WindowInput::ToggleRequested)
        });
        relm4::main_application().add_action(&toggle_action);

        let model = WindowModel {
            _launcher: launcher,
            window: root.clone(),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            WindowInput::ToggleRequested => {
                if self.window.is_visible() {
                    self.window.hide();
                } else {
                    self.window.present();
                }
            }
            WindowInput::HideRequested => self.window.hide(),
        }
    }
}

fn init_layer_shell(window: &gtk::Window) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("dionysus"));
    window.set_keyboard_mode(KeyboardMode::Exclusive);
}

fn init_key_handler(window: &gtk::Window, sender: &ComponentSender<WindowModel>) {
    let controller = gtk::EventControllerKey::new();
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    controller.connect_key_pressed({
        let sender = sender.clone();
        move |_, key, _, _| {
            if key == gdk::Key::Escape {
                sender.input(WindowInput::HideRequested);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        }
    });
    window.add_controller(controller);
}
