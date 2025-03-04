use adw::traits::ExpanderRowExt;
use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use std::cell::RefCell;
use std::unimplemented;

use crate::db::models::{Record, Task};
use crate::db::operations::{create_task, read_record, read_records, read_tasks, update_task};
use crate::views::project::{RecordCreateWindow, RecordRow, TaskRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_page.ui")]
    #[properties(wrapper_type=super::TaskPage)]
    pub struct TaskPage {
        #[property(get, set)]
        pub task: RefCell<Task>,
        #[template_child]
        pub task_row: TemplateChild<TaskRow>,
        #[template_child]
        pub description_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub description_buffer: TemplateChild<gtk::TextBuffer>,
        #[template_child]
        pub lists_menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub lists_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub new_subtask_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub new_record_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub subtasks_page: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub subtasks_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub records_page: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub records_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskPage {
        const NAME: &'static str = "TaskPage";
        type Type = super::TaskPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, _value| {
                let imp = obj.imp();
                imp.subtasks_box.invalidate_sort();
            });
            klass.install_action("task.duration-update", None, move |obj, _, _value| {
                obj.imp().task_row.refresh_timer();
            });
            klass.install_action("project.update", None, move |obj, _, _value| {
                let task = obj.task();
                let imp = obj.imp();
                let mut records =
                    read_records(task.id(), false, None, None).expect("Failed to read records");
                imp.task_row.refresh_timer();
                if imp.records_box.observe_children().n_items() != (records.len() + 1) as u32 {
                    records.sort_by_key(|record| record.id());
                    let row = RecordRow::new(records.last().unwrap().to_owned());
                    imp.records_box.append(&row);
                }
            });
            klass.install_action("record.delete", None, move |obj, _, _value| {
                obj.imp().task_row.refresh_timer();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskPage {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }
    impl WidgetImpl for TaskPage {}
    impl BoxImpl for TaskPage {}
}

glib::wrapper! {
    pub struct TaskPage(ObjectSubclass<imp::TaskPage>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable, gtk::Orientable;
}

#[gtk::template_callbacks]
impl TaskPage {
    pub fn new(task: Task) -> Self {
        let obj: Self = glib::Object::builder().build();
        let imp = obj.imp();
        obj.set_task(task.clone());
        imp.task_row.reset(task.clone());

        let task_description = task.description();
        imp.description_expander_row
            .set_subtitle(&obj.description_display(&task_description));
        imp.description_buffer.set_text(&task_description);

        imp.subtasks_box.set_sort_func(|row1, row2| {
            let task1 = row1.property::<Task>("task");
            let task2 = row2.property::<Task>("task");
            if task1.position() < task2.position() {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });
        imp.subtasks_box.set_filter_func(glib::clone!(
            @weak imp => @default-return false,
            move |row| {
                let row = row.downcast_ref::<TaskRow>().unwrap();
                !row.task().suspended()
        }));

        let tasks = read_tasks(task.project(), None, None, Some(task.id()))
            .expect("Failed to read subtasks");
        for task in tasks {
            let row = TaskRow::new(task);
            imp.subtasks_box.append(&row);
        }

        imp.records_box
            .set_sort_func(|row1: &gtk::ListBoxRow, row2| {
                let row1_start = row1.property::<Record>("record").start();
                let row2_start = row2.property::<Record>("record").start();

                if row1_start > row2_start {
                    gtk::Ordering::Smaller
                } else {
                    gtk::Ordering::Larger
                }
            });

        let records = read_records(task.id(), false, None, None).expect("Failed to read records");
        for record in records {
            let row = RecordRow::new(record);
            imp.records_box.append(&row);
        }

        obj
    }

    pub fn add_record(&self, record_id: i64) {
        let imp = self.imp();
        imp.task_row.refresh_timer();
        let record = read_record(record_id).expect("Failed to read record");
        let row = RecordRow::new(record);
        imp.records_box.append(&row);
    }

    fn description_display(&self, text: &str) -> String {
        if let Some(first_line) = text.lines().next() {
            return String::from(first_line);
        }
        String::from("")
    }

    #[template_callback]
    fn handle_description_buffer_changed(&self, buffer: gtk::TextBuffer) {
        let imp = self.imp();
        let task = self.task();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        if task.description() != text {
            imp.description_expander_row
                .set_subtitle(&self.description_display(&text));
            task.set_property("description", text);
            update_task(&task).expect("Failed to update task");
        }
    }

    #[template_callback]
    fn handle_lists_menu_row_activated(&self, row: gtk::ListBoxRow, _lists_box: gtk::ListBox) {
        let imp = self.imp();
        imp.lists_popover.popdown();
        let label = row.child().and_downcast::<gtk::Label>().unwrap();
        match row.index() {
            // Subtasks
            0 => {
                imp.new_subtask_button.set_visible(true);
                imp.subtasks_page.set_visible(true);
                imp.lists_menu_button.set_label(&label.label());
            }
            // Records
            1 => {
                imp.new_subtask_button.set_visible(false);
                imp.subtasks_page.set_visible(false);
                imp.lists_menu_button.set_label(&label.label());
            }
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_new_record_button_clicked(&self, _button: gtk::Button) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let modal = RecordCreateWindow::new(&win.application().unwrap(), &win, self.task().id());
        modal.present();
    }

    #[template_callback]
    fn handle_new_subtask_button_clicked(&self, _button: gtk::Button) {
        let task = self.task();
        let task = create_task("", task.project(), 0, task.id()).expect("Failed to create subtask");
        let task_ui = TaskRow::new(task);
        let imp = self.imp();
        imp.subtasks_box.prepend(&task_ui);
        let task_imp = task_ui.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
    }

    #[template_callback]
    fn handle_subtasks_box_row_activated(&self, row: gtk::ListBoxRow, _tasks_box: gtk::ListBox) {
        let row = row.downcast::<TaskRow>().unwrap();
        row.cancel_timer();
        self.activate_action("subtask.open", Some(&row.task().id().to_variant()))
            .expect("Failed to send subtask.open action");
    }
}
