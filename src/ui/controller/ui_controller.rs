use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::core;
use crate::core::search_worker::SearchCommand;
use crate::ui::UiHandle;
use crate::ui::controller::activation::activate_position;
use crate::ui::controller::keyboard::handle_global_key;
use crate::ui::controller::visibility::prepare_for_show;

pub struct UiController {
    ui: UiHandle,
    state: core::SharedState,
    search_tx: std::cell::RefCell<Option<mpsc::Sender<SearchCommand>>>,
    debounce_source: std::cell::RefCell<Option<gtk4::glib::SourceId>>,
}

impl UiController {
    pub fn new(ui: UiHandle, state: core::SharedState) -> Self {
        Self {
            ui,
            state,
            search_tx: std::cell::RefCell::new(None),
            debounce_source: std::cell::RefCell::new(None),
        }
    }

    pub fn attach_search_sender(&self, search_tx: mpsc::Sender<SearchCommand>) {
        *self.search_tx.borrow_mut() = Some(search_tx);
    }

    pub fn bind_events(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.ui
            .entries_list
            .connect_activate(move |_view, position| {
                if let Some(controller) = weak.upgrade() {
                    controller.on_list_activate(position);
                }
            });

        let global_keys = gtk4::EventControllerKey::new();
        global_keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        global_keys.connect_key_pressed(move |_, key, _keycode, _state| {
            if let Some(controller) = weak.upgrade() {
                return handle_global_key(
                    key,
                    &controller.ui.selection_model,
                    &controller.ui.entries_list,
                    &controller.state,
                    &controller.ui.main_window,
                );
            }
            gtk4::glib::Propagation::Proceed
        });
        self.ui.main_window.add_controller(global_keys);

        let weak = Rc::downgrade(self);
        self.ui.prompt.connect_changed(move |_| {
            if let Some(controller) = weak.upgrade() {
                controller.on_prompt_changed();
            }
        });
    }

    pub fn prepare_for_show(&self) {
        prepare_for_show(&self.ui);
    }

    pub fn toggle_visibility(&self) {
        if self.ui.main_window.is_visible() {
            self.ui.prompt.set_text("");
            self.send_query(String::new());
            self.ui.main_window.hide();
        } else {
            self.prepare_for_show();
        }
    }

    pub fn update_entries(&self, apps: &[core::AppEntry]) {
        crate::ui::update_entries(&self.ui, apps);
        if !apps.is_empty() {
            self.ui.selection_model.set_selected(0);
        }
    }

    pub fn on_list_activate(&self, position: u32) {
        activate_position(
            &self.ui.selection_model,
            &self.state,
            &self.ui.main_window,
            position,
        );
    }

    fn on_prompt_changed(self: &Rc<Self>) {
        if let Some(source_id) = self.debounce_source.borrow_mut().take() {
            let _ = source_id.remove();
        }

        let tx = self.search_tx.borrow().clone();
        let query = self.ui.prompt.text().to_string();
        let weak = Rc::downgrade(self);
        let source_id = gtk4::glib::timeout_add_local(Duration::from_millis(150), move || {
            if let Some(controller) = weak.upgrade() {
                *controller.debounce_source.borrow_mut() = None;
            }
            if let Some(tx) = &tx {
                let _ = tx.send(SearchCommand::Query(query.clone()));
            }
            gtk4::glib::ControlFlow::Break
        });

        *self.debounce_source.borrow_mut() = Some(source_id);
    }

    fn send_query(&self, query: String) {
        if let Some(tx) = self.search_tx.borrow().as_ref() {
            let _ = tx.send(SearchCommand::Query(query));
        }
    }
}
