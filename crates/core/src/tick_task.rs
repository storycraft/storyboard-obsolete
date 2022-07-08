use std::{
    fmt::Debug,
    panic,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use parking_lot::Mutex;
use replace_with::replace_with_or_abort;

#[derive(Debug)]
/// Run Repeated task on current / other thread independently
pub struct IndependentTickTask<T>(TickTaskVariant<T>);

impl<T: Send + 'static> IndependentTickTask<T> {
    pub fn run(item: T, func: fn(&mut T)) -> Self {
        #[cfg(debug_assertions)] {
            Self::run_none_threaded(item, func)
        }

        #[cfg(not(debug_assertions))] {
            Self::run_threaded(item, func)
        }
    }

    /// Try running task on newly created thread. If thread cannot be made, fallback to [`DedicatedTickTask::run_none_threaded`]
    pub fn run_threaded(item: T, handler: fn(&mut T)) -> Self {
        let interrupted = Arc::new(AtomicBool::new(false));

        // Using option to fallback if thread fail to spawn.
        let task_handle = Arc::new(Mutex::new(Some(TaskHandle {
            item,
            handler,
        })));

        let thread_handle = {
            let task_handle = task_handle.clone();
            let interrupted = interrupted.clone();

            thread::Builder::new().spawn(move || {
                let mut task_handle = task_handle.lock().take().unwrap();

                while !interrupted.load(Ordering::Relaxed) {
                    (task_handle.handler)(&mut task_handle.item);
                }

                task_handle
            })
        };

        match thread_handle {
            Ok(handle) => Self(TickTaskVariant::Threaded(ThreadedTask {
                interrupted,
                handle: Some(handle),
            })),

            Err(_) => {
                let task_handle = task_handle.lock().take().unwrap();
                Self::run_none_threaded(task_handle.item, task_handle.handler)
            }
        }
    }

    pub fn run_none_threaded(item: T, handler: fn(&mut T)) -> Self {
        Self(TickTaskVariant::NonThreaded(NonThreadedTask {
            interrupted: false,
            handle: TaskHandle {
                item,
                handler
            }
        }))
    }

    pub fn interrupted(&self) -> bool {
        match &self.0 {
            TickTaskVariant::Threaded(task) => task.interrupted(),
            TickTaskVariant::NonThreaded(task) => task.interrupted(),
        }
    }

    pub fn interrupt(&mut self) {
        match &mut self.0 {
            TickTaskVariant::Threaded(task) => task.interrupt(),
            TickTaskVariant::NonThreaded(task) => task.interrupt(),
        };
    }
    
    pub const fn threaded(&self) -> bool {
        matches!(self.0, TickTaskVariant::Threaded(_))
    }

    pub fn set_threaded(&mut self, threaded: bool) {
        // Ensure the task is not interrupted since switching mode will revive task.
        if self.interrupted() || self.threaded() == threaded {
            return;
        }

        if let TickTaskVariant::Threaded(task) = &mut self.0 {
            task.tick();
        }

        replace_with_or_abort(self, |this| {
            match this.0 {
                TickTaskVariant::Threaded(task) => {
                    let task_handle = match task.handle.unwrap().join() {
                        Ok(task_handle) => task_handle,
                        Err(_) => unreachable!(),
                    };
    
                    Self::run_none_threaded(task_handle.item, task_handle.handler)
                },
                TickTaskVariant::NonThreaded(task) => Self::run_threaded(task.handle.item, task.handle.handler),
            }
        });
    }

    pub fn tick(&mut self) {
        match &mut self.0 {
            TickTaskVariant::Threaded(task) => task.tick(),
            TickTaskVariant::NonThreaded(task) => task.tick(),
        }
    }

    pub fn join(self) -> T {
        match self.0 {
            TickTaskVariant::Threaded(task) => task.join(),
            TickTaskVariant::NonThreaded(task) => task.join(),
        }
    }
}

#[derive(Debug)]
enum TickTaskVariant<T> {
    Threaded(ThreadedTask<T>),
    NonThreaded(NonThreadedTask<T>),
}

#[derive(Debug)]
struct ThreadedTask<T> {
    interrupted: Arc<AtomicBool>,
    handle: Option<JoinHandle<TaskHandle<T>>>,
}

impl<T> ThreadedTask<T> {
    pub fn interrupted(&self) -> bool {
        self.interrupted.load(Ordering::Relaxed)
    }

    pub fn interrupt(&self) {
        self.interrupted.store(true, Ordering::Relaxed);
    }

    pub fn tick(&mut self) {
        if Arc::strong_count(&self.interrupted) < 2 && !self.interrupted() {
            panic::resume_unwind(self.handle.take().unwrap().join().err().unwrap());
        }
    }

    pub fn join(self) -> T {
        match self.handle.unwrap().join() {
            Ok(task_handle) => task_handle.item,
            Err(err) => panic::resume_unwind(err),
        }
    }
}

struct NonThreadedTask<T> {
    interrupted: bool,
    handle: TaskHandle<T>
}

impl<T> NonThreadedTask<T> {
    pub fn interrupted(&self) -> bool {
        self.interrupted
    }

    pub fn interrupt(&mut self) {
        self.interrupted = true;
    }

    pub fn tick(&mut self) {
        if !self.interrupted {
            (self.handle.handler)(&mut self.handle.item);
        }
    }

    #[inline]
    pub fn join(self) -> T {
        self.handle.item
    }
}

impl<T> Debug for NonThreadedTask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonThreadedTask")
            .field("interrupted", &self.interrupted)
            .finish_non_exhaustive()
    }
}

struct TaskHandle<T> {
    item: T,
    handler: fn(&mut T),
}
