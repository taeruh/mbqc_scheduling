/*!
Basic timing functionality (not precise).
*/

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// A simple timer.
///
/// Whether the timer has finished can be checked with the `finished` method.
///
/// **Note**: This should not be used for precise timing, since it is not precise.
/// Instead, it should be rather used, for example, to signal a thread to stop after the
/// timeout.
///
/// # Examples
// `#[cfg_attr(miri, ignore)] fn main() {` does not work in doc test (in general
// #[ingnore] does not work
/// ```no_run
/// # use std::thread; use std::time::Duration; use mbqc_scheduling::timer::Timer;
/// let mut timer = Timer::new();
/// timer.start(Duration::from_millis(20));
/// let timer = &timer;
/// thread::scope(|scope| {
///     scope.spawn(|| {
///         thread::sleep(Duration::from_millis(10));
///         assert!(!timer.finished());
///     });
///     scope.spawn(|| {
///         thread::sleep(Duration::from_millis(30));
///         assert!(timer.finished());
///     });
/// });
/// ```
pub struct Timer {
    pair: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
    // logically, this information is redundant with pair.0, but since it is not behind a
    // mutex, it can be read without locking
    finished: Arc<AtomicBool>,
}

impl Timer {
    /// Creates a new timer. Use `start` to start the timer.
    pub fn new() -> Self {
        Self {
            pair: Arc::new((Mutex::new(false), Condvar::new())),
            handle: None,
            finished: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the timer with the given duration.
    pub fn start(&mut self, duration: Duration) {
        let pair = self.pair.clone();
        let time = Arc::clone(&self.finished);

        self.handle = Some(thread::spawn(move || {
            let (lock, cvar) = &*pair;
            let (_lock, timeout) = cvar
                .wait_timeout_while(
                    lock.lock().expect("timer: locking at start failed"),
                    duration,
                    |&mut dropping_timer| !dropping_timer,
                )
                .expect("timer: re-acquiring lock after timeout/notifaction failed");
            if timeout.timed_out() {
                time.store(true, Ordering::Relaxed);
            }
        }));
    }

    /// Checks whether the timer has finished.
    ///
    /// If the timer never started, this will always return `false`.
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let (dropping_timer, cvar) = &*self.pair;
        *dropping_timer.lock().expect("timer: locking at drop failed") = true;
        cvar.notify_all();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap()
        } // otherwise it never started
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test() {
        let mut timer = Timer::new();
        timer.start(Duration::from_millis(20));
        let timer = &timer;
        thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(10));
                assert!(!timer.finished());
            });
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(30));
                assert!(timer.finished());
            });
        });
    }

    #[test]
    fn miri() {
        let mut timer = Timer::new();
        timer.start(Duration::from_millis(100));
        std::thread::sleep(Duration::from_millis(100));
    }
}
