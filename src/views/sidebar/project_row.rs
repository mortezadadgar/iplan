use gtk::{gdk, glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::read_projects;
use crate::views::sidebar::SidebarProjects;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/sidebar/project_row.ui")]
    pub struct ProjectRow {
        pub project: RefCell<Project>,
        #[template_child]
        pub icon_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectRow {
        const NAME: &'static str = "ProjectRow";
        type Type = super::ProjectRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectRow {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecObject::builder::<Project>("project").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "project" => {
                    let value = value.get::<Project>().expect("value must be a Project");
                    self.project.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "project" => self.project.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ProjectRow {}
    impl ListBoxRowImpl for ProjectRow {}
}

glib::wrapper! {
    pub struct ProjectRow(ObjectSubclass<imp::ProjectRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl ProjectRow {
    pub fn new(project: Project) -> Self {
        let obj = glib::Object::builder::<ProjectRow>()
            .property("project", project)
            .build();

        let imp = obj.imp();

        imp.icon_label.set_text(&obj.project().icon());
        imp.name_label.set_text(&obj.project().name());

        if obj.project().archive() {
            imp.name_label.add_css_class("dim-label")
        };

        obj
    }

    pub fn project(&self) -> Project {
        self.property("project")
    }

    #[template_callback]
    fn handle_drag_prepare(&self, _x: f64, _y: f64) -> Option<gdk::ContentProvider> {
        Some(gdk::ContentProvider::for_value(&self.to_value()))
    }

    #[template_callback]
    fn handle_drag_begin(&self, drag: gdk::Drag) {
        self.parent()
            .unwrap()
            .downcast::<gtk::ListBox>()
            .unwrap()
            .select_row(Some(self));
        let drag_icon: gtk::DragIcon = gtk::DragIcon::for_drag(&drag).downcast().unwrap();
        let label = gtk::Label::builder().label("").build();
        drag_icon.set_child(Some(&label));
        drag.set_hotspot(0, 0);
    }

    #[template_callback]
    fn handle_drag_cancel(&self, _drag: gdk::Drag) -> bool {
        let projects_box = self.parent().and_downcast::<gtk::ListBox>().unwrap();
        let projects = read_projects(true).expect("Failed to read projects");
        let rows = projects_box.observe_children();
        for row in rows.into_iter() {
            let row: ProjectRow = row.unwrap().downcast().unwrap();
            let row_project = row.project();
            for project in &projects {
                if project.id() == row_project.id() {
                    row_project.set_property("index", project.index());
                    row.set_property("project", row_project);
                    break;
                }
            }
        }
        projects_box.invalidate_sort();
        projects_box
            .parent()
            .and_downcast::<SidebarProjects>()
            .unwrap()
            .select_active_project();
        false
    }
}
