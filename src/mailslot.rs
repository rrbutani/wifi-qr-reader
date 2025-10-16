use std::{
    sync::{Arc, Mutex},
    thread::Thread,
};

#[derive(Clone)]
pub struct MailslotSender<T> {
    inner: Arc<Mutex<MailslotState<T>>>,
}

impl<T: Send> MailslotSender<T> {
    /// Send a value into the mailslot, replacing any previously placed values which have not yet been received.
    pub fn send_replace(&self, value: T) {
        let old_state = std::mem::replace(
            &mut *self.inner.lock().unwrap(),
            MailslotState::Present(value),
        );
        match old_state {
            MailslotState::Waiting { please_unpark } => {
                please_unpark.unpark();
            }
            MailslotState::Present(old_value) => {
                // old value goes unused
                drop(old_value);
            }
            MailslotState::Idle => {
                // no one is waiting, so no need to wait
            }
        }
    }
}

pub struct MailslotReceiver<T> {
    inner: Arc<Mutex<MailslotState<T>>>,
}

impl<T: Send> MailslotReceiver<T> {
    /// Wait to receive a value inserted into the mailslot, returning an error if all senders have been dropped.
    ///
    /// Warning: will block forever if all senders are dropped.
    pub fn recv(&mut self) -> T {
        loop {
            {
                let mut locked = self.inner.lock().unwrap();
                match std::mem::replace(&mut *locked, MailslotState::Idle) {
                    MailslotState::Present(value) => return value,
                    MailslotState::Waiting { .. } | MailslotState::Idle => {
                        *locked = MailslotState::Waiting {
                            please_unpark: std::thread::current(),
                        }
                    }
                };
            }
            std::thread::park();
        }
    }
}

#[derive(Debug)]
enum MailslotState<T> {
    Present(T),
    Waiting { please_unpark: Thread },
    Idle,
}
impl<T> Default for MailslotState<T> {
    fn default() -> Self {
        Self::Idle
    }
}

pub fn mailslot<T: Send>() -> (MailslotSender<T>, MailslotReceiver<T>) {
    let sender = MailslotSender {
        inner: Default::default(),
    };
    let receiver = MailslotReceiver {
        inner: Arc::clone(&sender.inner),
    };
    (sender, receiver)
}
