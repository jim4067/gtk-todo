use crate::background::BgEvent;
use crate::utils::spawn;
use crate::{Event, TaskEntity};

use crate::widgets::Task; //because TASK IS NOT FOUND anywhere

use async_channel::Sender;
use glib::{clone, SourceId};
// use glib::SourceId; ARE CHANGES REQUIRED HERE TOO BECAUSEE I IMPORTED IT ABOVE
use gtk::prelude::*;
use slotmap::SlotMap;
use std::time::Duration;

pub struct App {
    pub container: gtk::Grid,
    pub delete_button: gtk::Button,
    pub tasks: SlotMap<TaskEntity, Task>,
    pub scheduled_write: Option<SourceId>,
    pub tx: Sender<Event>,
    pub btx: Sender<BgEvent>,
}

impl App {
    pub fn new(app: &gtk::Application, tx: Sender<Event>, btx: Sender<BgEvent>) -> Self {
        let container = cascade! {
            gtk::Grid::new();
            ..set_column_spacing(4);
            ..set_row_spacing(4);
            ..set_border_width(4);
            ..show();
        };

        let scrolled = cascade! { //code changed here so make the changes in the gitbook and the example src code
            gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT,gtk::NONE_ADJUSTMENT);
            // ..hscrollbar_policy(gtk::PolicyType::Never);
            ..set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        };

        // let scrolled = gtk::ScrolledWindowBuilder::new()
        //     .hscrollbar_policy(gtk::PolicyType::Never)
        //     .build();
        scrolled.add(&container);

        let delete_button = cascade!{
            gtk::Button::from_icon_name(Some("edit-delete-symbolic"), gtk::IconSize::Button);
            ..set_label("Delete");
            ..set_always_show_image(true);
            ..set_no_show_all(true);
            ..style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION); //ALSO CHANGE HERE IN 2X09
            ..connect_clicked(clone!(@strong tx => move |_|{
                let tx = tx.clone();
                spawn(async move {
                    let _ = tx.send(Event::Delete).await;
                });
            }));
            
        };

        let headerbar = cascade!{
            gtk::HeaderBar::new();
            ..pack_end(&delete_button);
            ..set_title(Some("gtk-todo"));
            ..set_show_close_button(true);
        };

        let _window = cascade! {
            gtk::ApplicationWindow::new(app);
            ..set_titlebar(Some(&headerbar));
            ..add(&scrolled);
            ..connect_delete_event(clone!(@strong tx, @strong scrolled => move |win, _| {
                //detach the window preserving the entry widgets which contain the text
                win.remove(&scrolled);
                let tx = tx.clone();
                spawn(async move {
                    let _ = tx.send(Event::Closed).await;
                });
                gtk::Inhibit(false)
            }));
            ..show_all();
        };

        gtk::Window::set_default_icon_name("not_yet_designed_icon_name_here");

        let mut app = Self {
            delete_button,
            container,
            tasks: SlotMap::with_key(),
            scheduled_write: None,
            tx,
            btx,
        };
        app.insert_row(0);

        app
    }

    pub fn clear (&mut self){
        while let Some(entity) = self.tasks.keys().next(){
            self.remove_(entity)
        }
    }

    pub fn load(&mut self, data: String){
        self.clear();

        for (row, line) in data.lines().enumerate(){
            let entity = self.insert_row(row as i32);
            self.tasks[entity].set_text(line);
        }
    }

    fn insert_row(&mut self, row: i32) -> TaskEntity {
        for task in self.tasks.values_mut() {
            if task.row >= row {
                task.row += 1;
            }
        }

        self.container.insert_row(row); //why are we calling a function inside itself
        let task = Task::new(row);

        self.container.attach(&task.check, 0, row, 1, 1);
        self.container.attach(&task.entry, 1, row, 1, 1);
        self.container.attach(&task.insert, 2, row, 1, 1);

        task.entry.grab_focus();

        let entity = self.tasks.insert(task);
        self.tasks[entity].connect(self.tx.clone(), entity); //what does connect do here?
        return entity;
    }

    pub fn insert(&mut self, entity: TaskEntity) {
        let mut insert_at = 0;

        if let Some(task) = self.tasks.get(entity) {
            insert_at = task.row + 1;
        } //should semi-colon be here
        self.insert_row(insert_at);
    }

    pub fn modified(&mut self) {
        if let Some(id) = self.scheduled_write.take() {
            glib::source_remove(id);
        }

        let tx = self.tx.clone();
        self.scheduled_write = Some(glib::timeout_add_local(Duration::from_secs(5), move || {
            //CORRECTIONS HERE REQUIRED 2x07
            let tx = tx.clone();
            spawn(async move {
                let _ = tx.send(Event::SyncToDisk).await;
            });

            glib::Continue(false)
        }));
    }

    pub async fn sync_to_disk(&mut self) {
        self.scheduled_write = None;

        let contents = fomat_macros::fomat!(
            for node in self.tasks.values(){
                if node.entry.to_string().len() != 0 { //CHANGE HERE //to_string().len() 2x07
                    (node.entry.to_string()) "\n"  //CHANGE HERE to_string 2x07
                }
            }
        );

        let _ = self.btx.send(BgEvent::Save("Task".into(), contents)).await;
    }

    pub async fn closed(&mut self){
        self.sync_to_disk().await;
        let _ = self.btx.send(BgEvent::Quit).await;
    }

    pub fn remove(&mut self, entity: TaskEntity) {
        if self.tasks.len() == 1 {
            return;
        }
        self.remove_(entity);
    }

    fn remove_(&mut self, entity: TaskEntity) {
        if let Some(removed) = self.tasks.remove(entity) {
            self.container.remove_row(removed.row);

            //decrement the row by one
            for task in self.tasks.values_mut() {
                if task.row > removed.row {
                    task.row -= 1;
                }
            }
        } //should semi-colon be here
    }
}
