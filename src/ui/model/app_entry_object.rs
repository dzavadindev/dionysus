use crate::core::AppEntry;
use crate::core::IconRef;
use glib::subclass::prelude::*;
use gtk4::glib::{self, object::ObjectExt, value::ToValue};
use once_cell::sync::Lazy;
use std::cell::RefCell;

use glib::{ParamSpec, ParamSpecString, Value};

// NOTE:
// Gotta be honest, I think I know what is happening here, but I am not sure
// I haven't quite grasped GObjects yet, but this is intended to be a GTK
// friendly way to represent data objects.
// So this is a way to interpret raw data as something that
// ---

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AppEntryObject {
        pub id: RefCell<String>,
        pub name: RefCell<String>,
        pub exec: RefCell<String>,
        pub icon: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppEntryObject {
        const NAME: &'static str = "DionysusAppEntryObject";
        type Type = super::AppEntryObject;
    }

    impl ObjectImpl for AppEntryObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("id").build(),
                    ParamSpecString::builder("name").build(),
                    ParamSpecString::builder("icon").build(),
                    ParamSpecString::builder("exec").build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => {
                    let input: String = value.get().unwrap();
                    self.id.replace(input);
                }
                "name" => {
                    let input: String = value.get().unwrap();
                    self.name.replace(input);
                }
                "exec" => {
                    let input: String = value.get().unwrap();
                    self.exec.replace(input);
                }
                "icon" => {
                    let input: String = value.get().unwrap();
                    self.icon.replace(input);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.borrow().to_value(),
                "name" => self.name.borrow().to_value(),
                "exec" => self.exec.borrow().to_value(),
                "icon" => self.icon.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct AppEntryObject(ObjectSubclass<imp::AppEntryObject>);
}

impl AppEntryObject {
    pub fn new(entry: &AppEntry) -> Self {
        let mut icon_string = String::new();
        if let Some(icon_ref) = &entry.icon {
            icon_string = match icon_ref {
                IconRef::ThemedName(name) => name.to_string(),
                IconRef::FilePath(path) => path.to_string_lossy().to_string(),
            };
        }

        glib::Object::builder()
            .property("id", &entry.id)
            .property("name", &entry.name)
            .property("exec", &entry.exec)
            .property("icon", icon_string)
            .build()
    }

    pub fn id(&self) -> String {
        self.property::<String>("id")
    }

    pub fn name(&self) -> String {
        self.property::<String>("name")
    }

    pub fn exec(&self) -> String {
        self.property::<String>("exec")
    }

    pub fn icon(&self) -> String {
        self.property::<String>("icon")
    }
}
