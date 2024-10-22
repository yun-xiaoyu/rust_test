use std::io::Seek;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;

enum Message{
    NewJob(Job),
    Terminate,
}
pub struct ThreadPool{
    workers:Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker{
    id :usize,
    thread :Option<thread::JoinHandle<()>>,

}

trait FnBox{
    fn call_box(self :Box<Self>);
}
//struct Job{}
type Job = Box<dyn FnBox + Send+'static>;


impl<F:FnOnce()> FnBox for F{
    fn call_box(self: Box<Self>) {
        (*self)()
    }
}
impl ThreadPool{
    ///Create a  new ThreadPool
    ///
    /// The size is number of thread in the pool
    ///
    /// #Panic
    ///
    /// The 'new' function will be panic if the size is zero.
    pub fn new(size:usize) -> ThreadPool{
        assert!(size > 0);
        let (sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size{
            workers.push(Worker::new(id,Arc::clone(&receiver)));
            //create some threads and store in this vector
        }

        ThreadPool{workers,sender}
    }

    pub fn execute<F>(&self,f:F)
    where
        F:FnOnce()+Send+'static,
    {
        let job =Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Worker {
    fn new(id:usize,receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread = thread::spawn(move || loop{
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("worker {} got a job; executing.",id);
                    job.call_box();
                }
                Message::Terminate => {
                    println!("worker {} was told to terminate",id);
                    break;
                }
            }
        });

        Worker{
            id,
            thread:Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {

        println!("send terminate message to all workers.");

        for _ in &mut self.workers{
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("shutting down all workers");

        for worker in &mut self.workers{
            println!("shutting down worker{}",worker.id);

            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }
        }
    }
}