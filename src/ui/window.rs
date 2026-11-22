use dionysus::core::{LaunchError, Launcher, desktop, watcher};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use notify::RecommendedWatcher;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
    gtk::{self, gdk, gio, glib, prelude::*},
};

#[derive(Debug)]
pub struct WindowModel {
    launcher: Launcher,
    window: gtk::Window,
    list: gtk::ListBox,
    _watcher: Option<RecommendedWatcher>,
}

#[derive(Debug)]
pub enum WindowInput {
    ToggleRequested,
    HideRequested,
    MoveUp,
    MoveDown,
    Select(usize),
    LaunchSelected,
    CatalogChanged,
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

            gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                set_propagate_natural_height: true,
                set_max_content_height: 400,

                #[local_ref]
                list -> gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::Single,
                    connect_row_selected[sender] => move |_, row| {
                        if let Some(row) = row {
                            sender.input(WindowInput::Select(row.index() as usize));
                        }
                    },
                }
            }
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

        let watcher = watcher::init_watcher({
            let sender = sender.clone();
            move |event| match event {
                Ok(event)
                    if event
                        .paths
                        .iter()
                        .any(|path| path.extension().is_some_and(|ext| ext == "desktop")) =>
                {
                    sender.input(WindowInput::CatalogChanged);
                }
                Err(error) => eprintln!("Desktop watcher error: {error}"),
                _ => {}
            }
        })
        .map_err(|error| eprintln!("Could not watch desktop entries: {error}"))
        .ok();

        let model = WindowModel {
            launcher,
            window: root.clone(),
            list: gtk::ListBox::new(),
            _watcher: watcher,
        };
        let list = &model.list;
        let widgets = view_output!();
        model.rebuild_list();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            WindowInput::ToggleRequested => {
                if self.window.is_visible() {
                    self.window.hide();
                } else {
                    self.window.present();
                    if let Some(row) = self.list.selected_row() {
                        row.grab_focus();
                    }
                }
            }
            WindowInput::HideRequested => self.window.hide(),
            WindowInput::MoveUp => {
                self.launcher.move_selection_up();
                self.sync_selection();
            }
            WindowInput::MoveDown => {
                self.launcher.move_selection_down();
                self.sync_selection();
            }
            WindowInput::Select(index) => self.launcher.select_index(index),
            WindowInput::LaunchSelected => match self.launcher.launch_selected() {
                Ok(()) => self.hide_after_launch(),
                Err(LaunchError::Frequency(error)) => {
                    eprintln!("Application started, but could not save launch frequency: {error}");
                    self.hide_after_launch();
                }
                Err(LaunchError::NoSelection) => {}
                Err(error) => eprintln!("{error}"),
            },
            WindowInput::CatalogChanged => {
                self.launcher.replace_catalog(desktop::load_app_entries());
                self.rebuild_list();
            }
        }
    }
}

impl WindowModel {
    fn hide_after_launch(&mut self) {
        self.window.hide();
        self.launcher.reset();
        self.rebuild_list();
    }

    fn sync_selection(&self) {
        if let Some(row) = self
            .launcher
            .selected_index()
            .and_then(|index| self.list.row_at_index(index as i32))
        {
            self.list.select_row(Some(&row));
            row.grab_focus();
        }
    }

    fn rebuild_list(&self) {
        while let Some(row) = self.list.row_at_index(0) {
            self.list.remove(&row);
        }
        for app in self.launcher.results() {
            let row = gtk::ListBoxRow::new();
            let name = gtk::Label::new(Some(&app.name));
            name.set_xalign(0.0);
            name.set_ellipsize(gtk::pango::EllipsizeMode::End);
            name.set_margin_start(12);
            name.set_margin_end(12);
            name.set_margin_top(8);
            name.set_margin_bottom(8);
            row.set_child(Some(&name));
            self.list.append(&row);
        }
        if let Some(row) = self
            .launcher
            .selected_index()
            .and_then(|index| self.list.row_at_index(index as i32))
        {
            self.list.select_row(Some(&row));
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
            } else if key == gdk::Key::Up {
                sender.input(WindowInput::MoveUp);
                glib::Propagation::Stop
            } else if key == gdk::Key::Down {
                sender.input(WindowInput::MoveDown);
                glib::Propagation::Stop
            } else if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
                sender.input(WindowInput::LaunchSelected);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        }
    });
    window.add_controller(controller);
}
