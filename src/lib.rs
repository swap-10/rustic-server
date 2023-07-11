use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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
        ThreadPool { _workers: workers, sender: sender }
    }
    
    pub fn execute<F> (&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}


struct Worker {
    _id: usize,
    _worker_thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let worker_thread: thread::JoinHandle<()> = thread::spawn(move || loop {
            let job = receiver
                                                        .lock()
                                                        .expect("Failed to acquire mutex lock")
                                                        .recv()
                                                        .unwrap();
            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker { _id: id, _worker_thread: worker_thread }
    }
}