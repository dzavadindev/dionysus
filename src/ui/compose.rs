use dionysus::core::Launcher;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

#[derive(Debug)]
pub struct DModel {
    launcher: Launcher,
    visible: bool,
}

#[derive(Debug)]
pub enum DInputMessages {
    ToggleRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for DModel {
    type Init = Launcher;

    type Input = DInputMessages;
    type Output = ();

    view! {
        #[root]
        gtk4::Window {
            set_default_width: 700,
            set_default_height: 400,
            #[watch]
            set_visible: model.visible,
        }
    }

    fn init(
        launcher: Self::Init,
        _root: gtk4::Window,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let application = gtk4::Application::default();
        let activation_sender = sender.clone();

        application.connect_activate(move |_| {
            activation_sender.input(DInputMessages::ToggleRequested);
        });

        let model = DModel {
            launcher,
            visible: false,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            DInputMessages::ToggleRequested => {
                self.visible = !self.visible;
            }
        }
    }
}
