extern crate neovim_lib;
extern crate rmp;

use neovim_lib::session::Session;
use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;

#[ignore]
#[test]
fn start_stop_test() {
    let mut session = if cfg!(target_os = "windows") {
        Session::new_child_path("E:\\Neovim\\bin\\nvim.exe").unwrap()
    } else {
        Session::new_child().unwrap()
    };

    session.start_event_loop();

    let mut nvim = Neovim::new(session);
    println!("{:?}", nvim.get_api_info().unwrap());
}

#[ignore]
#[test]
fn remote_test() {
    let mut session = Session::new_tcp("127.0.0.1:6666").unwrap();
    session.start_event_loop();
    let mut nvim = Neovim::new(session);
    nvim.command("echo \"Test\"").unwrap();
}

#[ignore]
#[test]
fn edit_test() {
    let mut session = Session::new_tcp("127.0.0.1:6666").unwrap();
    session.start_event_loop();
    let mut nvim = Neovim::new(session);
    let buffers = nvim.get_buffers().unwrap();
    buffers[0].set_line(&mut nvim, 0, "replace first line").unwrap();
    nvim.command("vsplit").unwrap();
    let windows = nvim.get_windows().unwrap();
    windows[0].set_width(&mut nvim, 10).unwrap();
}
