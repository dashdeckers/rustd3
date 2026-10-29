//! [`Pool`]: the threads that rasterize jobs off the UI thread, each with
//! its own single-threaded renderer, the results channel and the wake
//! flag.
//!
//! The pool's threads are the only parallelism of its renders: each owns a
//! `raster::Renderer` with no thread pool of vello_cpu's own (`threads:
//! 0`), so the two never run together and oversubscribe the cores. By
//! default the pool has a thread per core but one (the UI's), 1 to 8,
//! counted by `available_parallelism`, which respects the process's CPU
//! affinity; with one thread the pool is Phase 19's single worker.

use std::io;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use iced_core::shell::Waker;
use rustd3_figure::raster;

use crate::plot::job::{Done, Job};
use crate::plot::queue::Queue;

/// The pool's threads' name, which tests count threads by.
pub(crate) const NAME: &str = "rustd3-plot";

/// The most threads `Plot::threads` asks for.
pub(crate) const MAX: usize = 64;

/// Threads rendering the newest jobs, nearest first.
pub(crate) struct Pool {
    queue: Arc<Queue>,
    done: mpsc::Receiver<Done>,
    /// Set by a thread before `wake`, cleared by the UI on drain: many
    /// finished jobs cost one `Event::Waken`.
    woken: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl Pool {
    /// Starts `size` threads (at least one), which wake the UI through
    /// `waker` after each result; an error when the OS cannot start one
    /// (the threads started are stopped).
    pub(crate) fn start(size: usize, waker: &Waker) -> io::Result<Pool> {
        let queue = Arc::new(Queue::new());
        let woken = Arc::new(AtomicBool::new(false));
        let (sender, done) = mpsc::channel();
        let mut pool = Pool {
            queue,
            done,
            woken,
            threads: Vec::with_capacity(size),
        };
        for _ in 0..size.max(1) {
            let (queue, woken) = (Arc::clone(&pool.queue), Arc::clone(&pool.woken));
            let (sender, waker) = (sender.clone(), waker.clone());
            let thread = thread::Builder::new()
                .name(NAME.to_owned())
                .spawn(move || run(&queue, &sender, &woken, &waker))?;
            pool.threads.push(thread);
        }
        Ok(pool)
    }

    /// How many threads it has.
    pub(crate) fn size(&self) -> usize {
        self.threads.len()
    }

    /// Replaces the waiting jobs with `jobs`, rendered in their order;
    /// each thread finishes the job it is rendering first.
    pub(crate) fn replace(&self, jobs: Vec<Job>) {
        self.queue.replace(jobs);
    }

    /// The generation of each job waiting, in order.
    #[cfg(test)]
    pub(crate) fn waiting(&self) -> Vec<u64> {
        self.queue.generations()
    }

    /// The results that arrived, in arrival order. The wake flag clears
    /// first, so a result sent during the drain wakes the UI again.
    pub(crate) fn drain(&self) -> Vec<Done> {
        self.woken.store(false, Ordering::Release);
        self.done.try_iter().collect()
    }

    /// Stops the threads, which finish the jobs they are rendering, and
    /// returns every result not drained yet.
    pub(crate) fn stop(mut self) -> Vec<Done> {
        self.join();
        self.done.try_iter().collect()
    }

    /// Closes the queue and joins the threads.
    fn join(&mut self) {
        self.queue.close();
        for thread in self.threads.drain(..) {
            // A thread that panicked has nothing left to clean up.
            let _ = thread.join();
        }
    }
}

impl Drop for Pool {
    /// Closes the queue and joins the threads, which finish the renders in
    /// flight (one job each) and abandon the rest.
    fn drop(&mut self) {
        self.join();
    }
}

/// The pool size when the app sets none: the available parallelism less
/// the UI thread, 1 to 8.
pub(crate) fn default_size() -> usize {
    size_for(thread::available_parallelism().map_or(1, NonZero::get))
}

/// The default pool size on `cores` cores.
fn size_for(cores: usize) -> usize {
    cores.saturating_sub(1).clamp(1, 8)
}

/// One thread: renders the queue's jobs in order with a single-threaded
/// renderer, sends each result and wakes the UI unless a wake is already
/// pending, until the queue closes.
fn run(queue: &Queue, sender: &mpsc::Sender<Done>, woken: &AtomicBool, waker: &Waker) {
    let mut renderer = raster::Renderer::new(raster::Settings {
        threads: 0,
        ..raster::Settings::default()
    });
    while let Some(job) = queue.take() {
        if sender.send(job.run(&mut renderer)).is_err() {
            return;
        }
        if !woken.swap(true, Ordering::AcqRel) {
            waker.wake();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::size_for;

    #[test]
    fn the_default_pool_leaves_a_core_to_the_ui() {
        // The spot rows: available parallelism 2 gives 1 thread, 16 give 8.
        assert_eq!([1, 2, 4, 9, 16, 64].map(size_for), [1, 1, 3, 8, 8, 8]);
    }
}
