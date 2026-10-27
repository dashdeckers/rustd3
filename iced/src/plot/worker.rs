//! [`Worker`]: the thread that rasterizes job sets off the UI thread, its
//! queue of at most one pending set, the results channel and the wake
//! flag.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError, mpsc};
use std::thread::{self, JoinHandle};

use iced_core::shell::Waker;
use rustd3_figure::raster;

use crate::plot::job::{Done, Job};

/// The worker thread's name, which tests count threads by.
pub(crate) const NAME: &str = "rustd3-plot";

/// One thread rendering the newest job set with vello_cpu's own pool.
pub(crate) struct Worker {
    queue: Arc<Queue>,
    done: mpsc::Receiver<Done>,
    /// Set by the worker before `wake`, cleared by the UI on drain: many
    /// finished cells cost one `Event::Waken`.
    woken: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

/// The pending job set, shared by the UI and the thread.
struct Queue {
    inbox: Mutex<Inbox>,
    ready: Condvar,
}

/// What the queue holds: a set waiting to be taken, and whether the
/// worker was dropped.
struct Inbox {
    jobs: Option<Vec<Job>>,
    closed: bool,
}

impl Worker {
    /// Starts the thread, which wakes the UI through `waker` after each
    /// result; an error when the OS cannot start it.
    pub(crate) fn start(waker: Waker) -> io::Result<Worker> {
        let queue = Arc::new(Queue {
            inbox: Mutex::new(Inbox {
                jobs: None,
                closed: false,
            }),
            ready: Condvar::new(),
        });
        let woken = Arc::new(AtomicBool::new(false));
        let (sender, done) = mpsc::channel();
        let thread = thread::Builder::new().name(NAME.to_owned()).spawn({
            let (queue, woken) = (Arc::clone(&queue), Arc::clone(&woken));
            move || run(&queue, &sender, &woken, &waker)
        })?;
        Ok(Worker {
            queue,
            done,
            woken,
            thread: Some(thread),
        })
    }

    /// Replaces the pending set with `jobs`; the thread abandons what is
    /// left of the set it is rendering after its current job.
    pub(crate) fn replace(&self, jobs: Vec<Job>) {
        self.queue.lock().jobs = Some(jobs);
        self.queue.ready.notify_all();
    }

    /// The results that arrived, in arrival order. The wake flag clears
    /// first, so a result sent during the drain wakes the UI again.
    pub(crate) fn drain(&self) -> Vec<Done> {
        self.woken.store(false, Ordering::Release);
        self.done.try_iter().collect()
    }
}

impl Drop for Worker {
    /// Closes the queue and joins the thread, which finishes the render in
    /// flight (one cell) and abandons the rest.
    fn drop(&mut self) {
        self.queue.lock().closed = true;
        self.queue.ready.notify_all();
        if let Some(thread) = self.thread.take() {
            // A worker that panicked has nothing left to clean up.
            let _ = thread.join();
        }
    }
}

impl Queue {
    /// The inbox; a lock poisoned by a panicking thread still holds a
    /// consistent inbox, since no code panics while holding it.
    fn lock(&self) -> MutexGuard<'_, Inbox> {
        self.inbox.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Waits for a job set and takes it; `None` once the worker is closed.
    fn take(&self) -> Option<Vec<Job>> {
        let mut inbox = self.lock();
        loop {
            if inbox.closed {
                return None;
            }
            if let Some(jobs) = inbox.jobs.take() {
                return Some(jobs);
            }
            inbox = self
                .ready
                .wait(inbox)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Whether a newer set replaced the one being rendered, or the worker
    /// closed.
    fn is_interrupted(&self) -> bool {
        let inbox = self.lock();
        inbox.closed || inbox.jobs.is_some()
    }
}

/// The thread: renders each set's jobs in order with one renderer on
/// vello_cpu's own pool, sends each result and wakes the UI unless a wake
/// is already pending, and abandons a set once a newer one replaced it.
fn run(queue: &Queue, sender: &mpsc::Sender<Done>, woken: &AtomicBool, waker: &Waker) {
    let mut renderer = raster::Renderer::new(raster::Settings::parallel());
    while let Some(jobs) = queue.take() {
        for job in jobs {
            if sender.send(job.run(&mut renderer)).is_err() {
                return;
            }
            if !woken.swap(true, Ordering::AcqRel) {
                waker.wake();
            }
            if queue.is_interrupted() {
                break;
            }
        }
    }
}
