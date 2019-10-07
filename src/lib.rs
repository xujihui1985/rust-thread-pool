use std::{fmt, thread};
use std::fmt::Formatter;
use std::sync::{Arc, mpsc, Mutex};

enum Message {
    NewJob(Job),
    Terminate,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        println!("fn box called");
        (*self)()
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            // Takes the value out of the option, leaving a [`None`] in its place.
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            //  This ensures that the lock is held during the call to recv, but it is released before the call to job.call_box(), allowing multiple requests to be serviced concurrently.
            while let Ok(message) = receiver.lock().unwrap().recv() {
                match message {
                    Message::NewJob(job) => {
                        println!("worker {} got a job", id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate", id);
                        break;
                    }
                }

            }
        });
        let w = Worker {
            id,
            thread: Some(thread),
        };
        w
    }
}

pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to create thread pool")
    }
}

impl fmt::Debug for PoolCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size <= 0 {
            return Err(PoolCreationError);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        Ok(ThreadPool {
            workers,
            sender,
        })
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

