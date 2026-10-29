//! [`Queue`]: the jobs waiting for the render pool, in the order they
//! render. A refresh replaces them whole: the threads finish the job each
//! is rendering and take the next from the new set, so the newest state
//! wins at the source.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

use crate::plot::job::Job;

/// The jobs waiting, shared by the UI and the pool's threads.
pub(crate) struct Queue {
    inbox: Mutex<Inbox>,
    ready: Condvar,
}

/// What the queue holds: the jobs in order, and whether the pool was
/// dropped.
struct Inbox {
    jobs: VecDeque<Job>,
    closed: bool,
}

impl Queue {
    /// An open queue with no job.
    pub(crate) fn new() -> Queue {
        Queue {
            inbox: Mutex::new(Inbox {
                jobs: VecDeque::new(),
                closed: false,
            }),
            ready: Condvar::new(),
        }
    }

    /// Replaces the waiting jobs with `jobs`, which render in their order.
    pub(crate) fn replace(&self, jobs: Vec<Job>) {
        self.lock().jobs = jobs.into();
        self.ready.notify_all();
    }

    /// Waits for a job and takes the first; `None` once the queue is
    /// closed.
    pub(crate) fn take(&self) -> Option<Job> {
        let mut inbox = self.lock();
        loop {
            if inbox.closed {
                return None;
            }
            if let Some(job) = inbox.jobs.pop_front() {
                return Some(job);
            }
            inbox = self
                .ready
                .wait(inbox)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// The generation of each job waiting, in order.
    #[cfg(test)]
    pub(crate) fn generations(&self) -> Vec<u64> {
        self.lock().jobs.iter().map(|job| job.generation).collect()
    }

    /// Closes the queue: the threads stop after the job each renders, and
    /// the jobs waiting are dropped.
    pub(crate) fn close(&self) {
        let mut inbox = self.lock();
        inbox.closed = true;
        inbox.jobs.clear();
        drop(inbox);
        self.ready.notify_all();
    }

    /// The inbox; a lock poisoned by a panicking thread still holds a
    /// consistent inbox, since no code panics while holding it.
    fn lock(&self) -> MutexGuard<'_, Inbox> {
        self.inbox.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
