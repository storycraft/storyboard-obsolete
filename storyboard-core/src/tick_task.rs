use std::{
    fmt::Debug,
    io, panic,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub enum DedicatedTickTask<T> {
    Threaded(ThreadedTask<T>),
    NonThreaded(NonThreadedTask<T>),
}

impl<T: Send + 'static> DedicatedTickTask<T> {
    #[cfg(not(target_family = "wasm"))]
    pub fn run(item: T, func: impl FnMut(&mut T) + Send + 'static) -> io::Result<Self> {
        Self::run_threaded(item, func)
    }

    #[cfg(target_family = "wasm")]
    pub fn run(item: T, func: impl FnMut(&mut T) + Send + 'static) -> io::Result<Self> {
        Ok(Self::run_none_threaded(item, func))
    }

    pub fn run_threaded(item: T, func: impl FnMut(&mut T) + Send + 'static) -> io::Result<Self> {
        let interrupted = Arc::new(AtomicBool::new(false));

        let handle = {
            let mut item = item;
            let mut func = func;
            let interrupted = interrupted.clone();

            thread::Builder::new().spawn(move || {
                while !interrupted.load(Ordering::Relaxed) {
                    func(&mut item);
                }

                (
                    item,
                    Box::new(func) as Box<dyn FnMut(&mut T) + Send + 'static>,
                )
            })?
        };

        Ok(Self::Threaded(ThreadedTask {
            interrupted,
            handle: Some(handle),
        }))
    }

    pub fn threaded(&self) -> bool {
        matches!(self, Self::Threaded(_))
    }

    pub fn run_none_threaded(item: T, func: impl FnMut(&mut T) + Send + 'static) -> Self {
        Self::NonThreaded(NonThreadedTask {
            interrupted: false,
            item,
            func: Box::new(func),
        })
    }

    pub fn interrupted(&self) -> bool {
        match self {
            DedicatedTickTask::Threaded(task) => task.interrupted(),
            DedicatedTickTask::NonThreaded(task) => task.interrupted(),
        }
    }

    pub fn interrupt(&mut self) {
        match self {
            DedicatedTickTask::Threaded(task) => task.interrupt(),
            DedicatedTickTask::NonThreaded(task) => task.interrupt(),
        };
    }

    pub fn tick(&mut self) {
        if let Self::NonThreaded(task) = self {
            task.tick();
        }
    }

    pub fn to_threaded(self) -> io::Result<Self> {
        if let Self::NonThreaded(task) = self {
            Self::run_threaded(task.item, task.func)
        } else {
            Ok(self)
        }
    }

    pub fn to_non_threaded(self) -> Self {
        if let Self::Threaded(task) = self {
            task.interrupt();
            let (item, func) = match task.handle.unwrap().join() {
                Ok(items) => items,
                Err(err) => panic::resume_unwind(err),
            };

            Self::run_none_threaded(item, func)
        } else {
            self
        }
    }

    pub fn join(self) -> T {
        match self {
            Self::Threaded(task) => task.join(),
            Self::NonThreaded(task) => task.item,
        }
    }
}

#[derive(Debug)]
pub struct ThreadedTask<T> {
    interrupted: Arc<AtomicBool>,
    handle: Option<JoinHandle<(T, Box<dyn FnMut(&mut T) + Send + 'static>)>>,
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
            Ok((item, _)) => item,
            Err(err) => panic::resume_unwind(err),
        }
    }
}

pub struct NonThreadedTask<T> {
    interrupted: bool,
    item: T,
    func: Box<dyn FnMut(&mut T) + Send + 'static>,
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
            self.func.as_mut()(&mut self.item);
        }
    }

    pub fn join(self) -> T {
        self.item
    }
}

impl<T> Debug for NonThreadedTask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonThreadedTask")
            .field("interrupted", &self.interrupted)
            .finish_non_exhaustive()
    }
}
