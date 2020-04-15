use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct IntervalCollector<T,D> { //not sure about that name
    pub interval: Duration,
    pub last_result: Arc<Mutex<Option<T>>>,
    pub data: Arc<Mutex<Option<D>>>,
    pub collect: Option<Box<dyn FnMut(Arc<Mutex<Option<T>>>, Arc<Mutex<Option<D>>>) + Send>>,
}

pub struct IntervalCollectorHandle<T> {
    pub thread_handle: thread::JoinHandle<()>,
    pub last_result: Arc<Mutex<Option<T>>>,
}

impl<T: Send + Sync + 'static, D: Send + Sync + 'static> IntervalCollector<T,D> {
    pub fn new() -> Self {
        IntervalCollector {
            interval: Duration::from_millis(5000),
            last_result: Arc::new(Mutex::new(None)),
            data: Arc::new(Mutex::new(None)),
            collect: None,
        }
    }

    pub fn interval(&mut self, interval: Duration) -> &mut Self {
        self.interval = interval;
        self
    }

    pub fn data(&mut self, data: D) -> &mut Self {
        self.data = Arc::new(Mutex::new(Some(data)));
        self
    }

    pub fn collect<F>(&mut self, collect: F) -> &mut Self where 
        F: 'static + FnMut(Arc<Mutex<Option<T>>>, Arc<Mutex<Option<D>>>) + Send,
    {
        self.collect = Some(Box::new(collect));
        self
    }

    pub fn start(self) -> IntervalCollectorHandle<T> {
        let last_result = self.last_result.clone();
        let s = self;
        IntervalCollectorHandle {
            thread_handle: std::thread::spawn(move || {
                let mut f = s.collect.unwrap();
                loop {
                    f(s.last_result.clone(), s.data.clone());
                    thread::sleep(s.interval);
                }
            }),
            last_result: last_result.clone(),
        }
    }
} //i dont like this at all
//i spent ~6 hours writing this
