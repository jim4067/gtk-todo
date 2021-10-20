use crate::{utils::spawn, Event, TaskEntity};
use async_channel::Sender;
use glib::{clone, SignalHandlerId};
use gtk::prelude::*;

pub struct Task {
    pub check: gtk::CheckButton,
    pub entry: gtk::Entry,
    pub insert: gtk::Button,
    pub remove: gtk::Button,
    entry_signal: Option<SignalHandlerId>,

    //track our position on the row
    pub row: i32,
}

//now we can construct our widgets
impl Task {
    pub fn new(row: i32) -> Self {
        let check = cascade!{
            gtk::CheckButton::new();
            ..show();
        };

        let entry = cascade! {
            gtk::Entry::new();
            ..set_hexpand(true);
            ..show();
        };

        let insert = cascade! {
            gtk::Button::from_icon_name(Some("list-add-symbolic"), gtk::IconSize::Button);
            ..show();
        };

        let remove = cascade! {
            gtk::Button::from_icon_name(Some("list-remove-symbolic"), gtk::IconSize::Button);
            ..show();
        };

        let entry_signal = None;

        Self {
            check,
            entry,
            entry_signal,
            insert,
            remove,
            row,
        }
    }
    pub fn connect(&mut self, tx: Sender<Event>, entity: TaskEntity) {
        let signal = self.entry.connect_changed(clone!(@strong tx => move |_| {
            let tx = tx.clone();
            spawn(async move {
                let _ = tx.send(Event::Modified).await;
            });
        }));

        self.entry_signal = Some(signal);
        //communicating whether the task has changed
        self.entry.connect_changed(clone!(@strong tx => move |_|{
            let tx = tx.clone();
            spawn(async move {
                let _ = tx.send(Event::Modified).await;
            });
        }));

        // self.check.connect_toggled(clone!(@strong tx => move |check| {
        //     let tx = tx.clone();
        //     let check = check.clone();

        //     spawn(async move {
        //         let _ = tx.send(Event::Toggled(check.get_active())).await;
        //     });//NO SEMI COLON HERE 
        // }));

        self.insert
            .connect_clicked(clone!(@strong tx, @weak self.entry as entry => move |_|{
                if entry.to_string().is_empty() {
                    return;
                }
                let tx = tx.clone();
                spawn(async move {
                    let _ = tx.send(Event::Insert(entity)).await;
                });
            }));

        self.remove.connect_clicked(clone!(@strong tx => move |_|{
            let tx = tx.clone();
            spawn(async move {
                let _ = tx.send(Event::Remove(entity)).await;
            });
        }));
    }

    pub fn set_text(&mut self, text: &str) {
        let signal = self.entry_signal.as_ref().unwrap();
        self.entry.block_signal(signal);
        self.entry.set_text(text);
        self.entry.unblock_signal(signal);
    }
}
