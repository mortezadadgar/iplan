use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

use crate::db::operations::read_tasks;

mod imp {
    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::Project)]
    pub struct Project {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub archive: Cell<bool>,
        #[property(get, set)]
        pub index: Cell<i32>,
        #[property(get, set)]
        pub icon: RefCell<String>,
        #[property(get, set)]
        pub description: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Project {
        const NAME: &'static str = "Project";
        type Type = super::Project;
    }

    impl ObjectImpl for Project {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct Project(ObjectSubclass<imp::Project>);
}

impl Project {
    pub fn new(
        id: i64,
        name: String,
        archive: bool,
        index: i32,
        icon: String,
        description: String,
    ) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("archive", archive)
            .property("index", index)
            .property("icon", icon)
            .property("description", description)
            .build()
    }

    pub fn duration(&self) -> i64 {
        let mut total = 0;
        for task in read_tasks(self.id(), None, None, Some(0)).expect("Failed to read tasks") {
            total += task.duration();
        }
        total
    }
}

impl TryFrom<&Row<'_>> for Project {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Project::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        ))
    }
}

impl Default for Project {
    fn default() -> Self {
        Project::new(1, String::new(), false, 0, String::new(), String::new())
    }
}
