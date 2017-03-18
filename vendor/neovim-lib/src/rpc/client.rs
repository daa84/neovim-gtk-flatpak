use std::io::{Read, Write};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{mpsc, Mutex, Arc};

use super::value::Value;

use super::model;

type Queue = Arc<Mutex<HashMap<u64, mpsc::Sender<Result<Value, Value>>>>>;

pub struct Client<R: Read + Send + 'static, W: Write> {
    reader: Option<R>,
    writer: W,
    dispatch_guard: Option<JoinHandle<()>>,
    event_loop_started: bool,
    queue: Queue,
    msgid_counter: u64,
}

impl<R: Read + Send + 'static, W: Write> Client<R, W> {
    pub fn take_dispatch_guard(&mut self) -> JoinHandle<()> {
        self.dispatch_guard.take().expect("Can only take join handle after running event loop")
    }

    pub fn start_event_loop_cb<F: FnMut(&str, Vec<Value>) + Send + 'static>(&mut self, cb: F) {
        self.dispatch_guard =
            Some(Self::dispatch_thread(self.queue.clone(), self.reader.take().unwrap(), cb));
        self.event_loop_started = true;
    }

    pub fn start_event_loop(&mut self) {
        self.dispatch_guard =
            Some(Self::dispatch_thread(self.queue.clone(), self.reader.take().unwrap(), |_, _| ()));
        self.event_loop_started = true;
    }

    pub fn new(reader: R, writer: W) -> Self {
        let queue = Arc::new(Mutex::new(HashMap::new()));
        Client {
            reader: Some(reader),
            writer: writer,
            msgid_counter: 0,
            queue: queue.clone(),
            dispatch_guard: None,
            event_loop_started: false,
        }
    }

    pub fn call_timeout(&mut self,
                        method: &str,
                        args: &Vec<Value>,
                        dur: Duration)
                        -> Result<Value, Value> {
        if !self.event_loop_started {
            return Err(Value::String("Event loop not started".to_owned()));
        }

        let mut wait_time = dur.as_secs() * 1_000_000_000 + dur.subsec_nanos() as u64;

        let receiver = self.send_msg(method, args);

        loop {
            match receiver.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::new(0, 1000_000));
                    wait_time -= 1000_000;
                    if wait_time <= 0 {
                        return Err(Value::String("Wait timeout".to_owned()));
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(Value::String("Channel disconnected".to_owned()))
                }
                Ok(val) => return val,
            };
        }
    }

    fn send_msg(&mut self,
                method: &str,
                args: &Vec<Value>)
                -> mpsc::Receiver<Result<Value, Value>> {
        let msgid = self.msgid_counter;
        self.msgid_counter += 1;

        let req = model::RpcMessage::RpcRequest {
            msgid: msgid,
            method: method.to_owned(),
            params: args.clone(),
        };

        let (sender, receiver) = mpsc::channel();
        self.queue.lock().unwrap().insert(msgid, sender);

        model::encode(&mut self.writer, &req).expect("Error send message");
        receiver
    }

    pub fn call(&mut self,
                method: &str,
                args: &Vec<Value>,
                dur: Option<Duration>)
                -> Result<Value, Value> {
        match dur {
            Some(dur) => self.call_timeout(method, args, dur),
            None => self.call_inf(method, args),
        }
    }

    pub fn call_inf(&mut self, method: &str, args: &Vec<Value>) -> Result<Value, Value> {
        if !self.event_loop_started {
            return Err(Value::String("Event loop not started".to_owned()));
        }

        let receiver = self.send_msg(method, args);

        receiver.recv().unwrap()
    }

    fn dispatch_thread<F: FnMut(&str, Vec<Value>) + Send + 'static>(queue: Queue,
                                                                    mut reader: R,
                                                                    mut cb: F)
                                                                    -> JoinHandle<()> {
        thread::spawn(move || loop {
            let msg = match model::decode(&mut reader) {
                Ok(msg) => msg,
                Err(e) => {
                    debug!("Error reading {}", e);
                    return;
                }
            };
            debug!("Get message {:?}", msg);
            match msg {
                model::RpcMessage::RpcResponse { msgid, result, error } => {
                    let sender = queue.lock().unwrap().remove(&msgid).unwrap();
                    if error != Value::Nil {
                        sender.send(Err(error)).unwrap();
                    }
                    sender.send(Ok(result)).unwrap();
                }
                model::RpcMessage::RpcNotification { method, params } => {
                    cb(&method, params);
                }
                _ => println!("Unknown type"),
            };
        })
    }
}
