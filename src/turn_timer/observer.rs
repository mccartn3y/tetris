use std::sync::mpsc;
pub trait Notifier<T: std::clone::Clone> {
    fn add_subscriber(&mut self, subscriber: &mut impl Subscriber<T>) {
        let (notifier_sender, notifier_receiver) = mpsc::channel::<T>();
        self.set_subscribers().push(notifier_sender);
        subscriber.add_subscription(notifier_receiver);
    }
    fn set_subscribers(&mut self) -> &mut Vec<mpsc::Sender<T>>;
    fn subscribers(&self) -> &Vec<mpsc::Sender<T>>;
    fn notify(&self, context: &T) {
        for subscriber in self.subscribers() {
            subscriber
                .send(context.clone())
                .expect(&format!("Failed to update subscriber {subscriber:?}"));
        }
    }
}

pub trait Subscriber<T: std::clone::Clone> {
    fn update(&mut self);
    fn add_subscription(&mut self, reciever: mpsc::Receiver<T>);
}
