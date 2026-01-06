//! A queue inspired by `tokio::sync::mpsc` but with more features
//!
//! Additional features:
//! - allow to peek into the queue to look at unprocessed data without removing
//!   it.
//! - remove data from anywhere in the queue.
//! - asynchronously wait for data to peek or remove using a filter callback
//!
//! Drawbacks compared to `tokio::sync::mpsc`:
//! - The queue is much slower since it doesn't use lockless types

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("receiver closed")]
    ReceiverClosed,
}

/// Internal type that holds the actual data
struct Queue<T> {
    /// a ringbuffer where we place user data
    data: std::collections::VecDeque<T>,
    /// waker to notify after [Self::data] became non-empty
    rx_waker: Option<core::task::Waker>,
    /// number of [Receiver] handles.
    ///
    /// We only allow one, but using a count shouldn't hurt the performance too
    /// much and makes it slightly easier to use.
    rx_count: usize,
    /// number of [Sender] handles
    tx_count: usize,
}

impl<T> Queue<T> {
    /// wake up RX waker if there is one
    fn wake_rx(&mut self) {
        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }
    }

    /// return [true] if the [Receiver] was dropped
    pub fn is_rx_closed(&self) -> bool {
        self.rx_count == 0
    }
}

/// a reference to a queued item that wasn't processed yet
///
/// This type basically represents one of the major features of this crate.
pub struct ValueRef<'a, T> {
    /// the mutex guard on the internal [Queue]
    guard: std::sync::MutexGuard<'a, Queue<T>>,
    /// the index of the item inside [Queue::data]
    ///
    /// This is not a reference because we'd have a self-referencing struct
    /// then since the data lives in [Self::guard].  
    /// That would make this type much harder to use and is probably not worth
    /// the performance overhead of [std::ops::Index].
    index: usize,
}

impl<T> std::ops::Deref for ValueRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.data[self.index]
    }
}

/// The receiver side of the queue
///
/// Currently, we only allow one even though there might not be a solid reason
/// for that limitation.
///
/// All functions extracting data from the queue complete with [None] when all
/// [Sender]s where dropped.
pub struct Receiver<T> {
    queue: std::sync::Arc<std::sync::Mutex<Queue<T>>>,
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut queue = self.queue.lock().unwrap();

        queue.rx_count -= 1;
        assert_eq!(queue.rx_count, 0);

        // once all receivers are gone we don't allow any access to the queue.
        // Let's also remove all queued data because that will notify other
        // mpsc's which are part of the queued data.
        queue.data.clear();
    }
}

impl<T> Receiver<T> {
    pub fn is_empty(&mut self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.data.is_empty()
    }

    /// remove and return an item from the queue as soon as one becomes
    /// available.
    ///
    /// This is the "normal" way to do this and not part of any of our special
    /// features.
    pub async fn recv(&mut self) -> Option<T> {
        futures_util::future::poll_fn(|cx| {
            let mut queue = self.queue.lock().unwrap();

            if queue.tx_count == 0 {
                return core::task::Poll::Ready(None);
            }

            if let Some(item) = queue.data.pop_front() {
                return core::task::Poll::Ready(Some(item));
            }

            queue.rx_waker = Some(cx.waker().clone());
            core::task::Poll::Pending
        })
        .await
    }

    /// wait for a queued item using a predicate function
    ///
    /// This function returns a reference to that value.
    /// The search does also include items already queued. Excluding those is
    /// beyond the scope of this crate and can be implemented inside the
    /// predicate.
    pub async fn wait_for<P: FnMut(&T) -> bool>(
        &mut self,
        mut predicate: P,
    ) -> Option<ValueRef<'_, T>> {
        futures_util::future::poll_fn(|cx| {
            let mut queue = self.queue.lock().unwrap();

            if queue.tx_count == 0 {
                return core::task::Poll::Ready(None);
            }

            for (index, item) in queue.data.iter().enumerate() {
                if !predicate(item) {
                    continue;
                }

                return core::task::Poll::Ready(Some(ValueRef {
                    guard: queue,
                    index,
                }));
            }

            queue.rx_waker = Some(cx.waker().clone());
            core::task::Poll::Pending
        })
        .await
    }

    /// Same as [Self::wait_for] but removes and returns the data
    ///
    /// Another major feature of this crate.  
    /// It allows removing certain items in the middle of the queue outside of
    /// the logic the user would place around the [Self::recv] function.
    pub async fn wait_for_remove<P: FnMut(&T) -> bool>(&mut self, mut predicate: P) -> Option<T> {
        futures_util::future::poll_fn(|cx| {
            let mut queue = self.queue.lock().unwrap();

            if queue.tx_count == 0 {
                return core::task::Poll::Ready(None);
            }

            for (index, item) in queue.data.iter().enumerate() {
                if !predicate(item) {
                    continue;
                }

                return core::task::Poll::Ready(Some(queue.data.remove(index).unwrap()));
            }

            queue.rx_waker = Some(cx.waker().clone());
            core::task::Poll::Pending
        })
        .await
    }

    /// remove all items matching `predicate`
    ///
    /// NOTE: this function is not panic-safe.
    ///       If the predicate causes a panic, items that do not match the
    ///       predicate will be lost as well.
    pub fn remove_matching<P: FnMut(&T) -> bool>(&mut self, mut predicate: P) -> usize {
        let mut queue = self.queue.lock().unwrap();

        let mut num_removed = 0;
        let mut new_queue = std::collections::VecDeque::with_capacity(queue.data.len());
        for item in queue.data.drain(..) {
            if predicate(&item) {
                num_removed += 1;
                continue;
            }

            new_queue.push_back(item);
        }

        queue.data = new_queue;

        num_removed
    }
}

/// The sender side of the queue
///
/// This is cloneable and you can have as many as you want.
/// Cloning does not copy the actual data.
pub struct Sender<T> {
    queue: std::sync::Arc<std::sync::Mutex<Queue<T>>>,
    counted: bool,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut queue = self.queue.lock().unwrap();

        queue.tx_count += 1;

        Self {
            queue: self.queue.clone(),
            counted: true,
        }
    }
}

impl<T> Sender<T> {
    /// send data into the queue
    pub fn send(&self, item: T) -> Result<(), Error> {
        let mut queue = self.queue.lock().unwrap();

        if queue.is_rx_closed() {
            return Err(Error::ReceiverClosed);
        }

        queue.data.push_back(item);
        queue.wake_rx();

        Ok(())
    }

    /// return true if the [Receiver] got dropped
    pub fn is_closed(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_rx_closed()
    }

    /// create a clone that is not counted
    ///
    /// If all counted senders are gone, the receiver gets closed. This
    /// function allows creating clones that are simply not considered for that
    /// check.
    pub fn clone_uncounted(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            counted: false,
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if !self.counted {
            return;
        }

        let mut queue = self.queue.lock().unwrap();

        queue.tx_count -= 1;
        if queue.tx_count != 0 {
            return;
        }

        queue.wake_rx();
    }
}

/// construct a new mpsc queue
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let queue = std::sync::Arc::new(std::sync::Mutex::new(Queue {
        data: std::collections::VecDeque::new(),
        rx_waker: None,
        rx_count: 1,
        tx_count: 1,
    }));

    (
        Sender {
            queue: queue.clone(),
            counted: true,
        },
        Receiver { queue },
    )
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn basics() {
        let (sender, mut receiver) = super::channel::<usize>();

        sender.send(42).unwrap();
        sender.send(43).unwrap();

        assert_eq!(receiver.recv().await, Some(42));
        assert_eq!(receiver.recv().await, Some(43));
        drop(sender);
        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    async fn basics_tasks() {
        let (sender, mut receiver) = super::channel::<usize>();
        let sender2 = sender.clone();

        tokio::task::spawn(async move {
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;

            sender.send(42).unwrap();

            tokio::time::sleep(core::time::Duration::from_millis(100)).await;
        });

        tokio::task::spawn(async move {
            tokio::time::sleep(core::time::Duration::from_millis(50)).await;

            sender2.send(43).unwrap();
        });

        assert_eq!(receiver.recv().await, Some(43));
        assert_eq!(receiver.recv().await, Some(42));
        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    async fn wait_for() {
        let (sender, mut receiver) = super::channel::<usize>();

        tokio::task::spawn(async move {
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;

            sender.send(43).unwrap();
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;

            sender.send(42).unwrap();
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;
        });

        assert_eq!(receiver.wait_for(|n| *n == 42).await.as_deref(), Some(&42));
        assert_eq!(receiver.recv().await, Some(43));
        assert_eq!(receiver.recv().await, Some(42));
        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    async fn wait_for_remove() {
        let (sender, mut receiver) = super::channel::<usize>();

        tokio::task::spawn(async move {
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;

            sender.send(43).unwrap();
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;

            sender.send(42).unwrap();
            tokio::time::sleep(core::time::Duration::from_millis(100)).await;
        });

        assert_eq!(receiver.wait_for_remove(|n| *n == 42).await, Some(42));
        assert_eq!(receiver.recv().await, Some(43));
        assert_eq!(receiver.recv().await, None);
    }
}
