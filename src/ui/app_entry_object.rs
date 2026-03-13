// NOTE: This file has a ton of "useless" comments that don't explain the reason,
// but actually just explain code. `glib` is something else, so I really need
// many of those clarifications to even read this code
//
// And if I am being honest, I am still re-reading this trying to
// understand what is happening
// ------------------------------------------------------------------------------

use crate::core::AppEntry;
use glib::subclass::prelude::*;
use gtk4::glib::{self, object::ObjectExt, value::ToValue};
use std::cell::RefCell;

// Lazy allows to initialize this static vector once, the first time it's needed.
use once_cell::sync::Lazy;

// ParamSpec describes a property in the GObject system.
// Value is GLib's dynamically typed value container.
use glib::{ParamSpec, ParamSpecString, Value};

// Convention in gtk-rs: put the actual low-level GObject implementation
// in a nested `imp` module.
//
// Because later there will be defined a public wrapper type ALSO called AppEntryObject.
//
// - imp::AppEntryObject = internal implementation/storage
// - AppEntryObject      = public wrapper used by the rest of the code

mod imp {
    use super::*;

    // This struct is the ACTUAL STORAGE of the object's fields.
    //
    // They are internal storage backing GObject properties.
    //
    // Use RefCell because set_property/property only get `&self`,
    // not `&mut self`.
    #[derive(Default)]
    pub struct AppEntryObject {
        pub id: RefCell<String>,
        pub name: RefCell<String>,
    }

    // This struct is the implementation of a custom GLib object subclass.
    //
    // It generates boilerplate for registering this type with GLib's runtime
    // type system.
    #[glib::object_subclass]
    impl ObjectSubclass for AppEntryObject {
        // This is the runtime type name in the GLib type system.
        // It should be unique.
        const NAME: &'static str = "DionysusAppEntryObject";

        // This links the internal implementation struct
        // to the PUBLIC wrapper type that is defined later.
        //
        // `super::AppEntryObject` does not refer to this internal struct.
        // It refers to the public wrapper below the `imp` module.
        type Type = super::AppEntryObject;
    }

    // `ObjectImpl` defines the behavior of a basic `glib::Object`.
    //
    // `ObjectSubclass` says:
    //   "this is a new object type"
    //
    // `ObjectImpl` says:
    //   "here is how that object behaves"
    // - what properties exist
    // - how to set them
    // - how to read them
    impl ObjectImpl for AppEntryObject {
        // This function tells GLib which properties this object exposes.
        //
        // Properties are named runtime fields that GTK can inspect,
        // bind to, read from, etc.
        //
        // Because GLib expects these definitions to live for the whole
        // program, return a `'static` slice.
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("id").build(),
                    ParamSpecString::builder("name").build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        // Called when GLib wants to SET one of the object's properties.
        // That eventually comes through here.
        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => {
                    // `value` is a dynamically typed GLib Value,
                    // need to extract the concrete Rust type from it.
                    let input: String = value.get().unwrap();

                    // Store it into an internal field.
                    // Use RefCell::replace because we only have `&self`.
                    self.id.replace(input);
                }
                "name" => {
                    let input: String = value.get().unwrap();
                    self.name.replace(input);
                }
                _ => {
                    // If GLib asks for a property we did not define,
                    // that's a programmer error.
                    unimplemented!()
                }
            }
        }

        // Called when GLib wants to READ one of the object's properties.
        // Return a `glib::Value`, because GObject properties are dynamically
        // typed at runtime.
        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => {
                    // Borrow the String from RefCell,
                    // then convert it into a GLib Value.
                    self.id.borrow().to_value()
                }
                "name" => self.name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

// `glib::wrapper!` creates the public Rust type that wraps the underlying
// GLib object instance.
//
// This is the type that is actually used in application logic.
// This is the thing to store in gio::ListStore.
glib::wrapper! {
    pub struct AppEntryObject(ObjectSubclass<imp::AppEntryObject>);
}

// Gives this custom object a Rust-friendly constructor and helper methods.
impl AppEntryObject {
    // A simple constructor that converts and AppEntry
    // into a GTK-friendly AppEntryObject.
    pub fn new(entry: &AppEntry) -> Self {
        // This creates a new GLib object and sets its properties.
        // Each `.property(...)` call will end up calling ObjectImpl::set_property(...)
        // implementation above.
        glib::Object::builder()
            .property("id", &entry.id)
            .property("name", &entry.name)
            .build()
    }

    // Helper getter for the "id" property.
    //
    // This is nicer than writing:
    // self.property::<String>("id")
    // everywhere in your code.
    //
    pub fn id(&self) -> String {
        self.property::<String>("id")
    }

    //
    // Helper getter for the "name" property.
    //
    pub fn name(&self) -> String {
        self.property::<String>("name")
    }
}
