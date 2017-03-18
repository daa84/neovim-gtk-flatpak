use std::result;
use std::net::TcpStream;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::process::Stdio;
use std::process::{Command, Child, ChildStdin, ChildStdout};
use std::thread::JoinHandle;
use std::time::Duration;

use rpc::value::Value;

use rpc::Client;

/// An active Neovim session.
pub struct Session {
    client: ClientConnection,
    timeout: Option<Duration>,
}

macro_rules! call_args {
    () => (Vec::new());
    ($($e:expr), *) => {{
        let mut vec = Vec::new();
        $(
            vec.push($e.into_val());
        )*
        vec
    }};
}

impl Session {
    /// Connect to nvim instance via tcp
    pub fn new_tcp(addr: &str) -> Result<Session> {
        let stream = TcpStream::connect(addr)?;
        let read = stream.try_clone()?;
        Ok(Session {
            client: ClientConnection::Tcp(Client::new(stream, read)),
            timeout: Some(Duration::new(5, 0)),
        })
    }

    /// Connect to a Neovim instance by spawning a new one.
    pub fn new_child() -> Result<Session> {
        if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
            Self::new_child_path("nvim")
        } else {
            Self::new_child_path("nvim.exe")
        }
    }

    /// Connect to a Neovim instance by spawning a new one
    pub fn new_child_path(program: &str) -> Result<Session> {
        let mut child = Command::new(program).arg("--embed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdout"))?;
        let stdin = child.stdin
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdin"))?;

        Ok(Session {
            client: ClientConnection::Child(Client::new(stdout, stdin), child),
            timeout: Some(Duration::new(5, 0)),
        })
    }

    /// Set call timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }

    pub fn set_infinity_timeout(&mut self) {
        self.timeout = None;
    }

    /// Start processing rpc response and notifications
    pub fn start_event_loop_cb<F: FnMut(&str, Vec<Value>) + Send + 'static>(&mut self, cb: F) {
        match self.client {
            ClientConnection::Child(ref mut client, _) => client.start_event_loop_cb(cb),
            ClientConnection::Tcp(ref mut client) => client.start_event_loop_cb(cb),
        }
    }

    /// Start processing rpc response and notifications
    pub fn start_event_loop(&mut self) {
        match self.client {
            ClientConnection::Child(ref mut client, _) => client.start_event_loop(),
            ClientConnection::Tcp(ref mut client) => client.start_event_loop(),
        }
    }

    /// Sync call. Call can be made only after event loop begin processing
    pub fn call(&mut self, method: &str, args: &Vec<Value>) -> result::Result<Value, Value> {
        match self.client {
            ClientConnection::Child(ref mut client, _) => client.call(method, args, self.timeout),
            ClientConnection::Tcp(ref mut client) => client.call(method, args, self.timeout),
        }
    }

    /// Wait dispatch thread to finish.
    ///
    /// This can happens in case child process connection is lost for some reason.
    pub fn take_dispatch_guard(&mut self) -> JoinHandle<()> {
        match self.client {
            ClientConnection::Child(ref mut client, _) => client.take_dispatch_guard(),
            ClientConnection::Tcp(ref mut client) => client.take_dispatch_guard(),
        }
    }
}

enum ClientConnection {
    Child(Client<ChildStdout, ChildStdin>, Child),
    Tcp(Client<TcpStream, TcpStream>),
}
