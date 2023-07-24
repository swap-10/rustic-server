use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// # Panics
    /// panics if the size is 0 because a threadpool
    /// with zero threads makes no sense
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);
        
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        //initialize a ThreadPool instance with this threads object and return
        ThreadPool {
            workers: workers,
            sender: Some(sender),
        }
    }
    
    pub fn execute<F> (&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.worker_thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let worker_thread: thread::JoinHandle<()> = thread::spawn(move || loop {
            let message = receiver
                                                        .lock()
                                                        .unwrap()
                                                        .recv();
            
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down");
                    break;
                }
            }
        });

        Worker {
            id: id,
            worker_thread: Some(worker_thread),
        }
    }
}