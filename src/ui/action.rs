use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use crate::backend::flatpak_backend::Package;

#[derive(Clone)]
pub enum ActionKind {
    RefreshInstalled,
}

// #[derive(Clone)]
// pub struct Action {
//     pub id: usize,
//     pub kind: ActionKind,
//     pub installed_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
// }

pub mod action {
    use std::sync::{Arc, Mutex};

    use iced::futures;
    use iced::futures::channel::mpsc;

    use crate::backend::flatpak_backend::{self, Package};

    #[derive(Debug, Clone)]
    pub enum Action {
        RefreshInstalled,
        StartA,
        StartB,
        CleanUp,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Ready(mpsc::Sender<Action>),
        Refreshed(Arc<Vec<Package>>),
        WorkAStarted,
        WorkBStarted,
        CleanUpStarted,
    }

    pub enum Event {}

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
            input: iced_futures::BoxStream<I>,
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
                    Action::StartA => Message::WorkAStarted,
                    Action::StartB => Message::WorkBStarted,
                    Action::CleanUp => Message::CleanUpStarted,
                }))
                .boxed()
        }
        // add code here
    }
}

// pub mod refresh {
//
//     use std::{
//         cell::RefCell,
//         sync::{Arc, Mutex},
//     };
//
//     use iced::{
//         futures::{channel::mpsc, StreamExt},
//         subscription, Subscription,
//     };
//
//     use crate::{
//         backend::flatpak_backend::{self, Package},
//         ui::action::ActionKind,
//     };
//     use iced_futures::futures;
//
//     use super::Action;
//
//     // #[derive(Debug, Clone)]
//     // pub enum Event {
//     //     Started(mpsc::Sender<Message>),
//     //     Done,
//     // }
//     #[derive(Debug, Clone)]
//     pub enum Event {
//         Started,
//         Done(usize),
//     }
//
//     // #[derive(Debug)]
//     // pub enum State {
//     //     Loading,
//     //     Loaded(mpsc::Receiver<Message>),
//     // }
//     #[derive(Debug)]
//     pub enum State {
//         Loading,
//         Loaded,
//     }
//
//     // pub fn refresh(action: &Action) -> Subscription<Event> {
//     //     struct Refresh;
//     //
//     //     let id = action.id;
//     //     let installed_apps = action.installed_apps.clone();
//     //
//     //     subscription::unfold(
//     //         std::any::TypeId::of::<Refresh>(),
//     //         State::Loading,
//     //         move |state| async move {
//     //             match state {
//     //                 State::Loading => {
//     //                     let apps = flatpak_backend::get_installed_apps();
//     //                     *installed_apps.lock().unwrap().borrow_mut() = apps;
//     //                     (Some(Event::Done(id)), State::Loaded)
//     //                 }
//     //                 State::Loaded => (None, State::Loaded),
//     //             }
//     //         },
//     //     )
//     // }
//     //
//     // pub fn refresh() -> Subscription<Event> {
//     //     struct Refresh;
//     //
//     //     subscription::unfold(
//     //         std::any::TypeId::of::<Refresh>(),
//     //         State::Loading,
//     //         |state| async move {
//     //             match state {
//     //                 State::Loading => {
//     //                     let (sender, receiver) = mpsc::channel(100);
//     //
//     //                     (Some(Event::Started(sender)), State::Loaded(receiver))
//     //                 }
//     //                 State::Loaded(mut receiver) => {
//     //                     if let Ok(Some(message)) = receiver.try_next() {
//     //                         match message {
//     //                             Message::Load(all_apps) => {
//     //                                 let apps = flatpak_backend::get_installed_apps();
//     //                                 *all_apps.lock().unwrap().borrow_mut() = apps;
//     //                                 (Some(Event::Done), State::Loaded(receiver))
//     //                             }
//     //                             _ => (None, State::Loaded(receiver)),
//     //                         }
//     //                     } else {
//     //                         (None, State::Loaded(receiver))
//     //                     }
//     //                 }
//     //             }
//     //         },
//     //     )
//     // }
//
//     #[derive(Debug, Clone)]
//     pub enum Message {
//         Load(Arc<Mutex<RefCell<Vec<Package>>>>),
//         Loaded,
//         Loading,
//     }
// }
