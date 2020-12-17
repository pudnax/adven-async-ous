fn program_main() {
    println!("So we start the program here!");
    set_timeout(200, || {
        println!("We create tasks wiith a callback that runs once the task finished!");
    });
    set_timeout(100, || {
        println!("We can even chain sub-tasks...");
        set_timeout(50, || {
            println!("....like this!");
        })
    });
    println!("While our tasks are executions we can do other stuff instead of waitling.");
}

fn main() {
    RT.with(|rt| rt.run(program_main));
}

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

thread_local! {
    static RT: Runtime = Runtime::new();
}

struct Runtime {
    callback: RefCell<HashMap<usize, Box<dyn FnOnce()>>>,
    next_id: RefCell<usize>,
    evt_sender: Sender<usize>,
    evt_receiver: Receiver<usize>,
}

fn set_timeout(ms: u64, cb: impl FnOnce() + 'static) {
    RT.with(|rt| {
        let id = *rt.next_id.borrow();
        rt.callback.borrow_mut().insert(id, Box::new(cb));
        let evt_sender = rt.evt_sender.clone();
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(ms));
            evt_sender.send(id).unwrap();
        });
    });
}

impl Runtime {
    fn new() -> Self {
        let (evt_sender, evt_receiver) = channel();
        Runtime {
            callback: RefCell::new(HashMap::new()),
            next_id: RefCell::new(1),
            evt_sender,
            evt_receiver,
        }
    }

    fn run(&self, program: fn()) {
        program();
        for evt_id in &self.evt_receiver {
            let cb = self.callback.borrow_mut().remove(&evt_id).unwrap();
            cb();
            if self.callback.borrow().is_empty() {
                break;
            }
        }
    }
}
