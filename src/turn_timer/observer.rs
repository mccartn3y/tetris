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
            if let Err(_) = subscriber.send(context.clone()) {
                log::warn!("Attempted to send message on a closed channel.")
            };
        }
    }
}

pub trait Subscriber<T: std::clone::Clone> {
    fn update(&mut self);
    fn add_subscription(&mut self, reciever: mpsc::Receiver<T>);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNotifier<T> {
        subscribers: Vec<mpsc::Sender<T>>,
    }
    impl<T: std::clone::Clone> Notifier<T> for TestNotifier<T> {
        fn set_subscribers(&mut self) -> &mut Vec<mpsc::Sender<T>> {
            &mut self.subscribers
        }
        fn subscribers(&self) -> &Vec<mpsc::Sender<T>> {
            &self.subscribers
        }
    }

    struct TestSubscriber {
        value: String,
        subscription: Option<mpsc::Receiver<String>>,
    }
    impl Subscriber<String> for TestSubscriber {
        fn update(&mut self) {
            if let Some(subscription) = &self.subscription {
                if let Ok(value) = subscription.try_recv() {
                    self.value = value;
                }
            }
        }
        fn add_subscription(&mut self, reciever: mpsc::Receiver<String>) {
            self.subscription = Some(reciever);
        }
    }

    #[test]
    fn test_subscriber_works() {
        let mut notifier: TestNotifier<String> = TestNotifier {
            subscribers: Vec::new(),
        };
        let mut subscriber = TestSubscriber {
            value: "".to_string(),
            subscription: None,
        };
        notifier.add_subscriber(&mut subscriber);

        assert_eq!(notifier.subscribers.len(), 1);
        assert!(subscriber.subscription.is_some());

        notifier.notify(&"notified!".to_string());
        subscriber.update();
        assert_eq!(subscriber.value, "notified!");
    }

    #[test]
    fn test_notifier_does_nothing_with_no_subs() {
        let notifier: TestNotifier<String> = TestNotifier {
            subscribers: Vec::new(),
        };
        notifier.notify(&"notified!".to_string());
    }
}
