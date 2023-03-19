pub mod action {
    use std::sync::Arc;

    use iced::futures;
    use iced::futures::channel::mpsc;

    use crate::backend::flatpak_backend::{self, Package};

    #[derive(Debug, Clone)]
    pub enum Action {
        RefreshInstalled,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Ready(mpsc::Sender<Action>),
        Refreshed(Arc<Vec<Package>>),
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
                }))
                .boxed()
        }
    }
}
