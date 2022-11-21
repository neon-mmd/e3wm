use std::{collections::HashMap, ptr::null, result};

use xcb::{
    x::{
        ChangeWindowAttributes, ConfigureWindow, Cw, EventMask, GetWindowAttributesCookie,
        KeyPressEvent, ReparentWindow, Visualid, Visualtype, Window, COPY_FROM_PARENT,
    },
    Xid, XidNew,
};
// use xcb::{Event::X, Xid};

struct XDisplay {
    conn: xcb::Connection,
    root: Window,
    clients: HashMap<u32, Client>,
    border_color: u32,
    border_hl_color: u32,
}

#[derive(Debug)]
struct Client {
    window: Window,
    parent: Window,
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

// fn handle_mapping_of_already_present_wins(conn: &xcb::Connection, root: Window) {
//     let cookie = conn.send_request_checked(&xcb::x::GrabServer {});
//     let result = conn.check_request(cookie);
//     if result.is_err() {
//         println!("MapRequest failed {:?}", result);
//         return;
//     }
//     let returned_root: Window;
//     let returned_parent: Window;
//     let top_level_windows: Vec<Window> = Vec::new();
//     let num_top_level_windows: u32;

//     let cookie = xcb::x::QueryTree { window: root };
//     println!("{:?}", cookie);
// }

impl XDisplay {
    fn new() -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).expect("can't connect display");
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize).unwrap();
        let window: Window = screen.root();

        let border_color_cookie = conn.send_request(&xcb::x::AllocColor {
            cmap: screen.default_colormap(),
            red: (0xff0000 >> 16) as u16 * 257,
            green: (0x00ff00 >> 8 & 0x0000ff) as u16 * 257,
            blue: (0x0000ff & 0x0000ff) as u16 * 257,
        });

        let border_hl_color_cookie = conn.send_request(&xcb::x::AllocColor {
            cmap: screen.default_colormap(),
            red: (0xff0000 >> 16) as u16 * 257,
            green: (0x00ff00 >> 8 & 0x0000ff) as u16 * 257,
            blue: (0x0000ff & 0x0000ff) as u16 * 257,
        });
        // let window_dummy:_ = conn.generate_id::<_>();
        register_for_xcb_events(&conn, window)
            .expect("Failed to register for XCB events. Other window manager running?");
        // handle_mapping_of_already_present_wins(&conn, window);

        let border_color = conn.wait_for_reply(border_color_cookie).unwrap().pixel() | 0xff << 24;
        let border_hl_color =
            conn.wait_for_reply(border_hl_color_cookie).unwrap().pixel() | 0xff << 24;

        Self {
            conn,
            root: window,
            clients: HashMap::new(),
            border_color,
            border_hl_color,
        }
    }

    fn frame(&mut self, win: Window) {
        let cookie = self.conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(win),
        });

        let result = self.conn.wait_for_reply(cookie).expect("err");
        let (x, y, width, height) = (result.x(), result.y(), result.width(), result.height());

        let mut new_win: xcb::x::Window = self.conn.generate_id();
        for (_, j) in self.clients.iter() {
            if j.window == win {
                new_win = j.parent;
            }
        }
        self.conn.send_request(&xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: new_win,
            parent: self.root,
            x,
            y,
            width,
            height,
            border_width: 3,
            class: xcb::x::WindowClass::InputOutput,
            visual: xcb::x::COPY_FROM_PARENT as Visualid,
            value_list: &[
                xcb::x::Cw::BorderPixel(self.border_color),
                xcb::x::Cw::EventMask(
                    xcb::x::EventMask::ENTER_WINDOW | xcb::x::EventMask::PROPERTY_CHANGE,
                ),
            ],
        });
        // self.conn.send_request(&xcb::x::ChangeWindowAttributes {
        //     window: win,
        //     value_list: &[
        //         xcb::x::Cw::BorderPixel(self.border_color),
        //         xcb::x::Cw::EventMask(
        //             xcb::x::EventMask::ENTER_WINDOW | xcb::x::EventMask::PROPERTY_CHANGE,
        //         ),
        //     ],
        // });
        let cookie = self
            .conn
            .send_request_checked(&xcb::x::MapWindow { window: new_win });

        let result = self.conn.check_request(cookie);
        if result.is_err() {
            println!("MapRequest failed {:?}", result);
            return;
        }

        self.conn.send_request(&xcb::x::ChangeSaveSet {
            mode: xcb::x::SetMode::Insert,
            window: win,
        });

        self.conn.send_request(&xcb::x::ReparentWindow {
            window: win,
            parent: new_win,
            x: 0,
            y: 0,
        });
    }

    fn unframe(&mut self, win: Window) {
        let mut new_win: xcb::x::Window = self.conn.generate_id();
        for (_, j) in self.clients.iter() {
            if j.window == win {
                new_win = j.parent;
            }
        }

        let cookie = self
            .conn
            .send_request_checked(&xcb::x::UnmapWindow { window: new_win });

        let result = self.conn.check_request(cookie);
        if result.is_err() {
            println!("UnMapRequest failed {:?}", result);
            return;
        }

        self.conn.send_request(&xcb::x::ReparentWindow {
            window: win,
            parent: self.root,
            x: 0,
            y: 0,
        });

        self.conn.send_request(&xcb::x::ChangeSaveSet {
            mode: xcb::x::SetMode::Insert,
            window: win,
        });

        self.conn
            .send_request(&xcb::x::DestroyWindow { window: new_win });
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
                            xcb::x::ConfigWindow::BorderWidth(e.border_width().into()),
                            xcb::x::ConfigWindow::StackMode(e.stack_mode().into()),
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
                    let newWin: xcb::x::Window = self.conn.generate_id();
                    self.clients.insert(
                        e.window().resource_id(),
                        Client {
                            window: e.window(),
                            parent: newWin,
                        },
                    );
                }
                Ok(xcb::Event::X(xcb::x::Event::MapRequest(event))) => {
                    self.frame(event.window());
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
                Ok(xcb::Event::X(xcb::x::Event::UnmapNotify(e))) => {
                   // self.unframe(e.window());
                }
                // Ok(xcb::Event::X(xcb::x::Event::DestroyNotify(e))) => {
                // unimplemented!();
                // }
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
