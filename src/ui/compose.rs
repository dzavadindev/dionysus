use dionysus::core::{desktop, freq_store, launcher};
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

#[derive(Debug)]
struct DModel {
    launcher: launcher::Launcher,
    visible: bool,
}

struct DModelInit {
    launcher: launcher::Launcher,
    visible: bool,
}

#[derive(Debug)]
enum DInputMessages {
    ShowLauncher,
}

#[derive(Debug)]
enum DOutputMessages {
    ShowLauncher,
}

#[derive(Debug)]
struct DWidgets {
    window: gtk4::Window,
}

#[relm4::component]
impl SimpleComponent for DModel {
    type Init = launcher::Launcher;

    type Input = DInputMessages;
    type Output = DOutputMessages;

    view! {
        #[root]
        gtk4::Window {
            set_default_width: 700,
            set_default_height: 400,
        }
    }

    fn init(
        init: Self::Init,
        root: gtk4::Window,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Init the model
        let freq = freq_store::load_freq().unwrap_or_default();
        let model = DModel {
            launcher: launcher::Launcher::new(desktop::load_app_entries(), freq, 10),
            visible: false,
        };
        // Generate the widgets
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            DInputMessages::ShowLauncher => {
                println!("Hello");
            }
        }
    }
}
