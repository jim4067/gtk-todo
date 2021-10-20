#[macro_use]
extern crate cascade;

mod app;
mod background;
mod utils;
mod widgets;

use self::app::App;
use gio::prelude::*;

slotmap::new_key_type! {
    pub struct TaskEntity;
}

pub enum Event {
    //inserting a task, identified by its key
    Insert(TaskEntity),

    //a previous task list has been fetched from a file using a background thread
    //and now its our job to display it in our UI
    Load(String), //SUGGESTION -> MAKE THIS A PATH

    //signal that an entry(task) was modifies and at some point we should save it
    Modified,

    //remove a task identified by its key -> DOES THIS MAKE SENSE
    Remove(TaskEntity),

    //signals that we should collect up the text from each task and pass it to a background thread
    //to save it to a file
    SyncToDisk, //ME SaveToDisk
    //signals that the window has been closed, so we should clean up and Quit
    Closed,

    Delete,

    Toggled(bool),

    //Signals that the process has been saved to the disk and it is safe to Quit,
    Quit,
}

pub const APP_ID: &str = "me.mutuku.todo";

fn main() {
    let app_name = "gtk-todo";

    glib::set_program_name(app_name.into());
    glib::set_application_name(app_name);

    //init the gtk application and register the app_id
    let app = gtk::Application::new(APP_ID.into(), Default::default());

    //after app has been registered it will trigger an activate signal,
    //which will give us the okay to construct our app and set up the app logic
    app.connect_activate(|app| {
        //channel for UI events in the main thread
        let (tx, rx) = async_channel::unbounded();

        //channel for background events in the background thread
        let (btx, brx) = async_channel::unbounded();

        //take ownership of a copy of the UI event sender (tx)
        //and the background receiver (brx)
        std::thread::spawn(glib::clone!(@strong tx => move || {
            //fetch the executor registered for this thread
            utils::thread_context()
            //block this thread on an event loop future
            .block_on(background::run(tx, brx));
        }));

        let mut app = app::App::new(app, tx, btx);

        let event_handler = async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    //event are arranged in the order they are most-likely to be called, with the called first
                    Event::Modified => app.modified(),
                    Event::Insert(entity) => app.insert(entity),
                    Event::Remove(entity) => app.remove(entity),
                    Event::SyncToDisk => app.sync_to_disk().await,
                    Event::Toggled(active) => (),
                    // Event::Toggled(active) => app.toggled(active),
                    Event::Delete => (),
                    // Event::Delete => app.delete(),
                    Event::Load(data) => app.load(data),
                    Event::Closed => app.closed().await,
                    Event::Quit => (),
                    // Event::Quit => app.quit(),
                }
            }
        };

        utils::spawn(event_handler);
    });

    app.run(); //problem here too
}
