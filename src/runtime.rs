use std::{
    collections::{BTreeMap, HashMap},
    sync::mpsc::{channel, Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
    time::{Duration, Instant},
};

const NUM_THREADS: usize = 4;

pub static mut RUNTIME: *mut Runtime = std::ptr::null_mut();

pub fn set_timeout(ms: u64, cb: impl Fn(Js) + 'static) {
    let rt = unsafe { &mut *(RUNTIME as *mut Runtime) };
    rt.set_timeout(ms, cb);
}

pub struct Runtime {
    available_threads: Vec<usize>,
    callbacks_to_run: Vec<(usize, Js)>,
    callback_queue: HashMap<usize, Box<dyn FnOnce(Js)>>,
    epoll_pending_events: usize,
    epoll_registrator: minimio::Registrator,
    epoll_thread: thread::JoinHandle<()>,
    epoll_timeout: Arc<Mutex<Option<i32>>>, // fix
    event_receiver: Receiver<PollEvent>,
    identity_token: usize,
    pending_events: usize,
    thread_pool: Vec<NodeThread>,
    timers: BTreeMap<Instant, usize>,
    timers_to_remove: Vec<Instant>,
}

impl Runtime {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = channel::<PollEvent>();
        let mut threads = Vec::with_capacity(4);

        for i in 0..4 {
            let (evt_sender, evt_receiver) = channel::<Task>();
            let event_sender = event_sender.clone();

            let handle = thread::Builder::new()
                .name(format!("pool{}", i))
                .spawn(move || {
                    while let Ok(task) = evt_receiver.recv() {
                        print(format!("received a task of type: {}", task.kind));

                        if let ThreadPoolTaskKind::Close = task.kind {
                            break;
                        };

                        let res = (task.task)();
                        print(format!("finished running a task of type: {}.", task.kind));

                        let event = PollEvent::ThreadPool((i, task.callback_id, res));
                        event_sender.send(event).expect("threadpool");
                    }
                })
                .expect("Couldn't initialize thread pool.");

            let node_thread = NodeThread {
                handle,
                sender: evt_sender,
            };

            threads.push(node_thread);
        }

        // ===== EPOLL THREAD =====
        let mut poll = minimio::Poll::new().expect("Error creating epoll queue");
        let registrator = poll.registrator();
        let epoll_timeout = Arc::new(Mutex::new(None));
        let epoll_timeout_clone = epoll_timeout.clone();

        let epoll_thread = thread::Builder::new()
            .name("epoll".to_string())
            .spawn(move || {
                let mut events = minimio::Events::with_capacity(1024);

                loop {
                    let epoll_timeout_handle = epoll_timeout_clone.lock().unwrap();
                    let timeout = *epoll_timeout_handle;
                    drop(epoll_timeout_handle);

                    match poll.poll(&mut events, timeout) {
                        Ok(v) if v > 0 => {
                            for i in 0..v {
                                let event = events.get_mut(i).expect("No events in event list.");
                                print(format!("epoll event {} is ready", event.id()));

                                let event = PollEvent::Epoll(event.id());
                                event_sender.send(event).expect("epoll event");
                            }
                        }
                        Ok(v) if v == 0 => {
                            print("epoll event timeout is ready");
                            event_sender
                                .send(PollEvent::Timeout)
                                .expect("epoll timeout");
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                            print("received event of type: Close");
                            break;
                        }
                        Err(e) => panic!("{:?}", e),
                        _ => unreachable!(),
                    }
                }
            })
            .expect("Error creating epoll thread");

        Runtime {
            available_threads: (0..4).collect(),
            callbacks_to_run: vec![],
            callback_queue: HashMap::new(),
            epoll_pending_events: 0,
            epoll_registrator: registrator,
            epoll_thread,
            epoll_timeout,
            event_receiver,
            identity_token: 0,
            pending_events: 0,
            thread_pool: threads,
            timers: BTreeMap::new(),
            timers_to_remove: vec![],
        }
    }

    pub fn run(mut self, f: impl Fn()) {
        let rt_ptr: *mut Runtime = &mut self;
        unsafe { RUNTIME = rt_ptr };
        let mut ticks = 0;

        f();

        while self.pending_events > 0 {
            ticks += 1;
            print(format!("===== TICK {} =====", ticks));
            self.process_expired_timers();
            self.run_callbacks();
            if self.pending_events == 0 {
                break;
            }
            let next_timeout = self.get_next_timeout();
            let mut epoll_timeout_lock = self.epoll_timeout.lock().unwrap();
            *epoll_timeout_lock = next_timeout;

            drop(epoll_timeout_lock);

            if let Ok(event) = self.event_reciever.recv() {
                match event {
                    PollEvent::Timeout => (),
                    PollEvent::Threadpool((thread_id, callback_id, data)) => {
                        self.process_threadpool_events(thread_id, callback_id, data);
                    }
                    PollEvent::Epoll(event_id) => {
                        self.process_epoll_events(event_id);
                    }
                }
            }
            self.run_callbacks();
        }
        for thread in self.thread_pool.into_iter() {
            thread
                .sender
                .send(Task::close())
                .expect("threadpool cleanup");
            thread.handle.join().unwrap();
        }

        self.epoll_registrator.close_loop().unwrap();
        self.epoll_thread.join().unwrap();

        print("FINISHED");
    }

    fn process_expired_timers(&mut self) {
        let timers_to_remove = &mut self.timers_to_remove;

        self.timers
            .range(..=Instant::now())
            .for_each(|(k, _)| timers_to_remove.push(*k));

        while let Some(key) = self.timers_to_remove.pop() {
            let callback_id = self.timers.remove(&key).unwrap();
            self.callbacks_to_run.push((callback_id, Js::Undefined));
        }
    }

    fn get_next_timeout(&self) -> Option<i32> {
        self.timers.iter().nth(0).map(|(&instant, _)| {
            let mut tim_to_next_timeout = instant - Instant::now();
            if tim_to_next_timeout < Duration::new(0, 0) {
                tim_to_next_timeout = Duration::new(0, 0);
            }
            tim_to_next_timeout.as_millis() as i32
        })
    }

    fn run_callback(&mut self) {
        while let Some((callback_id, data)) = self.callbacks_to_run.pop() {
            let cb = self.callback_queue.remove(&callback_id).unwrap();
            cb(data);
            self.pending_events -= 1;
        }
    }

    fn process_threadpool_events(&mut self, thread_id: usize, callback_id: usize, data: Js) {
        // fix
        self.callbacks_to_run.push((callback_id, data));
        self.available_threads.push(thread_id);
    }

    fn process_epoll_events(&mut self, event_id: usize) {
        self.callbacks_to_run.push((event_id, Js::Undefined));
        self.epoll_pending_events -= 1;
    }

    fn get_available_thread(&mut self) -> usize {
        match self.available_threads.pop() {
            Some(thread_id) => thread_id,
            // We would normally return None and not panic!
            None => panic!("Out of threads."),
        }
    }

    fn generate_identity(&mut self) -> usize {
        self.identity_token = self.identity_token.wrapping_add(1);
        self.identity_token
    }

    fn generate_cb_identity(&mut self) -> usize {
        let ident = self.generate_identity();
        let taken = self.callback_queue.contains_key(&ident); // fix

        if !taken {
            ident
        } else {
            loop {
                let possible_ident = self.generate_identity();
                if self.callback_queue.contains_key(&possible_ident) {
                    break possible_ident;
                }
            }
        }
    }

    fn add_callback(&mut self, ident: usize, cb: impl FnOnce(Js) + 'static) {
        let boxed_cb = Box::new(cb);
        self.callback_queue.insert(ident, boxed_cb);
    }

    pub fn register_event_epoll(&mut self, token: usize, cb: impl FnOnce(Js) + 'static) {
        self.add_callback(token, cb);

        print(format!("Event with id: {} registered.", token));
        self.pending_events += 1;
        self.epoll_pending_events += 1;
    }

    pub fn register_event_threadpool(
        &mut self,
        task: impl Fn() -> Js + Send + 'static,
        kind: ThreadPoolTaskKind,
        cb: impl FnOnce(Js) + 'static,
    ) {
        let callback_id = self.generate_cb_identity();
        self.add_callback(callback_id, cb);

        let event = Task {
            task: Box::new(task),
            callback_id,
            kind,
        };

        let available = self.get_available_thread();
        self.thread_pool[available]
            .sender
            .send(event)
            .expect("register work");
        self.pending_events += 1;
    }

    fn set_timeout(&mut self, ms: u64, cb: impl Fn(Js) + 'static) {
        // Is it theoretically possible to get two equal instants? If so we'll have a bug...
        let now = Instant::now();
        let cb_id = self.generate_cb_identity();
        self.add_callback(cb_id, cb);
        let timeout = now + Duration::from_millis(ms);
        self.timers.insert(timeout, cb_id);
        self.pending_events += 1;
        print(format!("Registered timer event id: {}", cb_id));
    }
}

fn f() {
    todo!()
}

struct Task {
    task: Box<dyn Fn() -> Js + Send + 'static>,
    callback_id: usize,
    kind: ThreadPoolTaskKind,
}

impl Task {
    fn close() -> Self {
        Task {
            task: Box::new(|| Js::Undefined),
            callback_id: 0,
            kind: ThreadPoolTaskKind::Close,
        }
    }
}

struct NodeThread {
    pub(crate) handle: JoinHandle<()>,
    sender: Sender<Task>,
}

pub enum ThreadPoolTaskKind {
    FileRead,
    Encrypt,
    Close,
}

pub enum Js {
    Undefined,
    String(String),
    Int(usize),
}

impl Js {
    fn into_string(self) -> Option<String> {
        match self {
            Js::String(s) => Some(s),
            _ => None,
        }
    }

    fn into_int(self) -> Option<usize> {
        match self {
            Js::Int(n) => Some(n),
            _ => None,
        }
    }
}

enum PollEvent {
    ThreadPool((usize, usize, Js)),
    Epoll(usize),
    Timeout,
}
