//! # Rust library for Neovim clients
//!
//! Implements support for rust plugins for [Neovim](https://github.com/neovim/neovim) through its msgpack-rpc API.
//! # Examples
//! ```no_run
//! use neovim_lib::{Neovim, NeovimApi, Session};
//!
//! let mut session = Session::new_tcp("127.0.0.1:6666").unwrap();
//! session.start_event_loop();
//! let mut nvim = Neovim::new(session);
//!
//! let buffers = nvim.get_buffers().unwrap();
//! buffers[0].set_line(&mut nvim, 0, "replace first line").unwrap();
//! nvim.command("vsplit").unwrap();
//! let windows = nvim.get_windows().unwrap();
//! windows[0].set_width(&mut nvim, 10).unwrap();
//! ```
extern crate rmp;
#[macro_use]
extern crate log;

mod rpc;
#[macro_use]
pub mod session;
pub mod neovim;
pub mod neovim_api;

pub use neovim::{Neovim, UiAttachOptions, CallError};
pub use neovim_api::NeovimApi;
pub use session::Session;

pub use rpc::value::{Value, Integer, Float};
