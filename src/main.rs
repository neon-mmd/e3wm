use std::collections::HashMap;

use xcb::{
    randr::SelectInput,
    x::{
        ChangeWindowAttributes, ConfigureWindow, Cw, EventMask, GetWindowAttributesCookie,
        KeyPressEvent, ReparentWindow, Visualid, Window, COPY_FROM_PARENT,
    },
    Xid,
};
// use xcb::{Event::X, Xid};

struct XDisplay {
    conn: xcb::Connection,
    root: Window,
    clients: HashMap<u32, Client>,
}

struct Client {
    window: Window,
}

fn register_keybindings(e: KeyPressEvent) {
    if e.detail() == 0x18 {
        std::process::exit(0);
    } else if e.detail() == 0x19 {
        std::process::Command::new("dmenu_run")
            .spawn()
            .expect("error");
    }
}

fn register_for_xcb_events(conn: &xcb::Connection, root: Window) -> xcb::ProtocolResult<()> {
    let event_mask: EventMask = EventMask::SUBSTRUCTURE_REDIRECT
        | EventMask::STRUCTURE_NOTIFY
        | EventMask::SUBSTRUCTURE_NOTIFY
        | EventMask::PROPERTY_CHANGE
        | EventMask::BUTTON_PRESS
        | EventMask::BUTTON_RELEASE
        | EventMask::POINTER_MOTION
        | EventMask::FOCUS_CHANGE
        | EventMask::ENTER_WINDOW
        | EventMask::LEAVE_WINDOW
        | EventMask::KEY_PRESS;

    let req = ChangeWindowAttributes {
        window: root,
        value_list: &[Cw::EventMask(event_mask)],
    };

    let cookie = conn.send_request_checked(&req);

    conn.check_request(cookie)
}

impl XDisplay {
    fn new() -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).expect("can't connect display");
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();
        let window: Window = screen.root();
        // let window_dummy:_ = conn.generate_id::<_>();
        register_for_xcb_events(&conn, window)
            .expect("Failed to register for XCB events. Other window manager running?");
        Self {
            conn,
            root: window,
            clients: HashMap::new(),
        }
    }

    fn run(&mut self) {
        loop {
            let event = self.conn.wait_for_event();
            match event {
                Ok(xcb::Event::X(xcb::x::Event::ConfigureRequest(e))) => {
                    let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
                        window: self.root,
                        value_list: &[
                            xcb::x::ConfigWindow::X(e.x().into()),
                            xcb::x::ConfigWindow::Y(e.y().into()),
                            xcb::x::ConfigWindow::Width(e.width().into()),
                            xcb::x::ConfigWindow::Height(e.height().into()),
                            xcb::x::ConfigWindow::BorderWidth(3),
                        ],
                    });
                    self.conn.flush().unwrap();
                    let result = self.conn.check_request(cookie);
                    if result.is_err() {
                        println!("ConfigureRequest failed {:?}", result);
                    }
                }
                Ok(xcb::Event::X(xcb::x::Event::KeyPress(e))) => {
                    register_keybindings(e);
                }
                Ok(xcb::Event::X(xcb::x::Event::CreateNotify(e))) => {
                    self.clients
                        .insert(e.window().resource_id(), Client { window: e.window() });
                }
                Ok(xcb::Event::X(xcb::x::Event::MapRequest(event))) => {
                    let cookie = self.conn.send_request_checked(&xcb::x::MapWindow {
                        window: event.window(),
                    });

                    let result = self.conn.check_request(cookie);
                    if result.is_err() {
                        println!("MapRequest failed {:?}", result);
                        return;
                    }
                }
                Ok(xcb::Event::X(xcb::x::Event::ReparentNotify(e))) => {
                    self.conn.send_request(&ReparentWindow {
                        window: e.window(),
                        parent: self.root,
                        x: 0,
                        y: 0,
                    });
                }
                Ok(e) => {
                    print!("");
                }
                Err(e) => {
                    print!("");
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
