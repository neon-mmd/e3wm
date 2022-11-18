use std::error::Error;
use xcb::randr::{GetScreenInfo, GetScreenResources};
use xcb::x::{
    CreateWindow, Cw, Event, EventMask, MapWindow, Screen, Window, WindowClass, COPY_FROM_PARENT,
};
use xcb::{Event::X, Xid};

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
        let window_dummy = conn.generate_id();
        conn.send_request(&CreateWindow {
            depth: COPY_FROM_PARENT as u8,
            wid: window_dummy,
            parent: screen.root(),
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            class: WindowClass::InputOnly,
            visual: 0,
            value_list: &[],
        });
        let cookie = conn.send_request_checked(&MapWindow {
            window: window_dummy,
        });

        conn.check_request(cookie).expect("error occured");
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
