use std::sync::{Arc, Condvar, Mutex};
use std::collections::VecDeque;


pub struct Sender<T>{
    shared : Arc<Shared<T>>

}
pub struct Receiver<T>{
    shared : Arc<Shared<T>>, 
    buffer : VecDeque<T>
}


pub struct Inner<T>{
    queue : VecDeque<T>,
    senders : usize
}
pub struct Shared<T>{
    inner : Mutex<Inner<T>>,
    available : Condvar
}
impl<T> Sender<T>{
    pub fn send(&mut self , t : T){
    let mut inner = self.shared.inner.lock().unwrap();

    inner.queue.push_front(t);
    drop(inner);
    self.shared.available.notify_all();
    }
}

impl<T> Clone for Sender<T>{
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner);
        Sender{
            shared : Arc::clone(&self.shared)
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();

        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner);
        if was_last{
            self.shared.available.notify_one();;
        }

    }
}
impl<T> Receiver<T>{
    pub fn recv(&mut self) ->Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t)
        }
        loop{
        let mut inner = self.shared.inner.lock().unwrap();

        
        match inner.queue.pop_front(){
            Some(n) => {
                if self.buffer.is_empty() {
                    std::mem::swap(&mut self.buffer, &mut inner.queue);
                }
                return Some(n)
            }, 
            None if inner.senders == 0 => {
                return None
            }
            None => {
             inner =  self.shared.available.wait(inner).unwrap();
            }
        }}

    }
}



pub fn channel<T>()-> (Sender<T> , Receiver<T>){
    
    let inner = Inner{
        queue : VecDeque::new(),
        senders : 1
    };

    let shared = Shared{
        inner: Mutex::new(inner),
        available : Condvar::new()

    };
    let shared = Arc::new(shared);
    
    (Sender{
        shared : shared.clone()
    }, Receiver{
        shared : shared.clone(),
        buffer : VecDeque::new()
    })

}
