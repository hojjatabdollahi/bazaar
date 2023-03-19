use std::sync::{Arc, Mutex};

use iced::futures;
use iced::futures::channel::mpsc;
use rusqlite::Connection;

use crate::{
    backend::flatpak_backend::{self, Package, PackageId},
    db::Storage,
};

#[derive(Debug, Clone)]
pub enum Action {
    RefreshInstalled,
    Uninstall(PackageId),
    Search((Arc<Mutex<Storage>>, String)),
}

#[derive(Debug, Clone)]
pub enum Message {
    Ready(mpsc::Sender<Action>),
    Refreshed(Arc<Vec<Package>>),
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
        let (tx, rx) = mpsc::channel(1);
        println!("stream");

        futures::stream::once(async { Message::Ready(tx) })
            .chain(rx.map(|action| match action {
                Action::RefreshInstalled => {
                    let apps = flatpak_backend::get_installed_apps();
                    Message::Refreshed(Arc::new(apps))
                }
                Action::Uninstall(id) => {
                    flatpak_backend::uninstall(&id);
                    Message::Uninstalled(id)
                }
                Action::Search((db, st)) => {
                    let mut apps = vec![];
                    if let Ok(mut stmt) = db
                        .lock()
                        .unwrap()
                        .conn
                        .prepare("SELECT name, desc FROM packages WHERE name = :name")
                    {
                        let _ = stmt
                            .query_map(&[(":name", st.to_string().as_str())], |row| {
                                println!("found something");
                                apps.push(Package {
                                    name: row.get(0).unwrap(),
                                    description: row.get(1).unwrap(),
                                    ..Package::default()
                                });
                                Ok(())
                            })
                            .unwrap()
                            .collect::<Vec<_>>();
                    }

                    Message::Found(Arc::new(apps))
                }
            }))
            .boxed()
    }
}
