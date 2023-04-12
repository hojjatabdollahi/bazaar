use std::sync::{Arc, Mutex};

use iced::futures;
use iced::futures::channel::mpsc;

use crate::{
    backend::flatpak_backend::{self, Package, PackageId},
    db::{
        search::{get_staff_picks, search},
        Storage,
    },
};

#[derive(Debug, Clone)]
pub enum Action {
    RefreshInstalled,
    RefreshUpdates,
    Uninstall(PackageId),
    Search((Arc<Mutex<Storage>>, String)),
    RefreshStaffPicks(Arc<Mutex<Storage>>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Ready(mpsc::Sender<Action>),
    Installed(Arc<Vec<Package>>),
    Updates(Arc<Vec<Package>>),
    StaffPicks(Arc<Vec<Package>>),
    Found(Arc<Vec<Package>>),
    Uninstalled(PackageId),
}

pub fn subscribe() -> iced::Subscription<Message> {
    iced::Subscription::from_recipe(BackendSubscription)
}

pub struct BackendSubscription;

impl<H, I> iced_native::subscription::Recipe<H, I> for BackendSubscription
where
    H: std::hash::Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::BoxStream<I>,
    ) -> iced_futures::BoxStream<Self::Output> {
        use futures::stream::StreamExt;
        let (tx, rx) = mpsc::channel(10);
        futures::stream::once(async { Message::Ready(tx) })
            .chain(rx.map(|action| match action {
                Action::RefreshInstalled => {
                    let apps = flatpak_backend::get_installed_apps();
                    Message::Installed(Arc::new(apps))
                }
                Action::RefreshUpdates => {
                    let apps = flatpak_backend::get_updatable_apps();
                    println!("Found {} updates", apps.len());
                    Message::Updates(Arc::new(apps))
                }
                Action::RefreshStaffPicks(db) => {
                    let apps = get_staff_picks(db);
                    Message::StaffPicks(Arc::new(apps))
                }
                Action::Uninstall(id) => {
                    flatpak_backend::uninstall(&id);
                    Message::Uninstalled(id.clone())
                }
                Action::Search((db, st)) => {
                    let apps = search(db.clone(), &st);
                    Message::Found(Arc::new(apps))
                }
            }))
            .boxed()
    }
}
