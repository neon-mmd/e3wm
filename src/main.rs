use std::error::Error;
use xcb::x::{Screen, Window};
use xcb::Xid;

struct XDisplay {
    conn: xcb::Connection,
    root: Window,
}

impl XDisplay {
    fn new() -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).expect("can't connect display");
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();
        let window: Window = screen.root();
        Self { conn, root: window }
    }

    fn run(&mut self) {
        loop {
            let event = self.conn.wait_for_event();
            match event {
                Ok(e) => {
                    println!("hello!! ");
                }
                Err(e) => {
                    println!("error occured!!");
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut app = XDisplay::new();
    app.run();
}
