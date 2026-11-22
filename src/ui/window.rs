use dionysus::core::{LaunchError, Launcher, desktop, search::SearchMode, watcher};
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
    prompt: gtk::Entry,
    mode_indicator: gtk::Label,
    scroller: gtk::ScrolledWindow,
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
    QueryChanged(String),
    ToggleSearchMode,
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

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "prompt_overlay"]
                gtk::Overlay {
                    #[local_ref]
                    prompt -> gtk::Entry {
                        add_css_class: "dionysus-prompt",
                        set_placeholder_text: Some("Search applications"),
                        connect_changed[sender] => move |entry| {
                            sender.input(WindowInput::QueryChanged(entry.text().to_string()));
                        },
                    }
                },

                #[local_ref]
                scroller -> gtk::ScrolledWindow {
                    set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                    set_propagate_natural_height: true,

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
    }

    fn init(
        launcher: Self::Init,
        root: gtk::Window,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        super::load_styles();
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
            prompt: gtk::Entry::new(),
            mode_indicator: gtk::Label::new(None),
            scroller: gtk::ScrolledWindow::new(),
            list: gtk::ListBox::new(),
            _watcher: watcher,
        };
        let list = &model.list;
        let prompt = &model.prompt;
        let scroller = &model.scroller;
        let widgets = view_output!();
        model.mode_indicator.set_halign(gtk::Align::End);
        model.mode_indicator.set_valign(gtk::Align::Center);
        model
            .mode_indicator
            .add_css_class("dionysus-mode-indicator");
        model.mode_indicator.set_can_target(false);
        widgets.prompt_overlay.add_overlay(&model.mode_indicator);
        model.update_mode_indicator();

        let (_, prompt_height, _, _) = model.prompt.measure(gtk::Orientation::Vertical, -1);
        model
            .scroller
            .set_max_content_height((400 - prompt_height).max(1));
        model.rebuild_list();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            WindowInput::ToggleRequested => {
                if self.window.is_visible() {
                    self.hide_and_reset();
                } else {
                    self.window.present();
                    self.prompt.grab_focus();
                }
            }
            WindowInput::HideRequested => self.hide_and_reset(),
            WindowInput::MoveUp => {
                self.launcher.move_selection_up();
                self.sync_selection();
            }
            WindowInput::MoveDown => {
                self.launcher.move_selection_down();
                self.sync_selection();
            }
            WindowInput::Select(index) => self.launcher.select_index(index),
            WindowInput::QueryChanged(query) => {
                if query != self.launcher.query() {
                    self.launcher.set_query(query);
                    self.rebuild_list();
                }
            }
            WindowInput::ToggleSearchMode => {
                if let Err(error) = self.launcher.toggle_mode() {
                    eprintln!("Could not save search mode: {error}");
                }
                self.update_mode_indicator();
                self.rebuild_list();
            }
            WindowInput::LaunchSelected => match self.launcher.launch_selected() {
                Ok(()) => self.hide_and_reset(),
                Err(LaunchError::Frequency(error)) => {
                    eprintln!("Application started, but could not save launch frequency: {error}");
                    self.hide_and_reset();
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
    fn hide_and_reset(&mut self) {
        self.window.hide();
        self.launcher.reset();
        self.prompt.set_text("");
        self.rebuild_list();
    }

    fn update_mode_indicator(&self) {
        self.mode_indicator.set_label(match self.launcher.mode() {
            SearchMode::Freq => "FREQ",
            SearchMode::Fuzz => "FUZZ",
        });
    }

    fn sync_selection(&self) {
        if let Some(row) = self
            .launcher
            .selected_index()
            .and_then(|index| self.list.row_at_index(index as i32))
        {
            self.list.select_row(Some(&row));
            let list = self.list.clone();
            let scroller = self.scroller.clone();
            glib::idle_add_local_once(move || {
                if let Some(bounds) = row.compute_bounds(&list) {
                    let adjustment = scroller.vadjustment();
                    let top = adjustment.value();
                    let bottom = top + adjustment.page_size();
                    let row_top = f64::from(bounds.y());
                    let row_bottom = f64::from(bounds.y() + bounds.height());
                    if row_top < top {
                        adjustment.set_value(row_top);
                    } else if row_bottom > bottom {
                        adjustment.set_value(row_bottom - adjustment.page_size());
                    }
                }
            });
        }
    }

    fn rebuild_list(&self) {
        self.scroller.vadjustment().set_value(0.0);
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
        move |_, key, _, modifiers| {
            if (key == gdk::Key::Tab || key == gdk::Key::ISO_Left_Tab)
                && modifiers.contains(gdk::ModifierType::CONTROL_MASK)
            {
                sender.input(WindowInput::ToggleSearchMode);
                glib::Propagation::Stop
            } else if key == gdk::Key::Escape {
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
