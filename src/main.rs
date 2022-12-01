use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};

#[derive(Debug)]
struct Window {
    win: xcb::x::Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

struct WindowChanges {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    border_width: u16,
    // sibling: xcb::x::Window,
    // stack_mode: xcb::x::StackMode,
}

impl Window {
    fn new(conn: &Connection, win: xcb::x::Window) -> Self {
        let (x, y, width, height) = conn.get_geometry(win);
        Self {
            win,
            x,
            y,
            width,
            height,
        }
    }
    fn get_window(&self) -> xcb::x::Window {
        self.win
    }
}

struct Connection {
    conn: xcb::Connection,
    root_window: Window,
    root_index: i32,
    screen_height: u16,
    screen_width: u16,
}

struct Layouts {
    windows: Vec<Window>,
    focus_win: usize,
}

impl Layouts {
    fn new() -> Self {
        Self {
            windows: Vec::new(),
            focus_win: 0,
        }
    }

    fn stack() {}

    fn max(&self, conn: &Connection, win: xcb::x::Window) {
        if self.windows.is_empty() {
            return;
        }

        let gaps = 5;
        let window_changes = WindowChanges {
            x: 0 + gaps,
            y: 26 + gaps,
            width: conn.screen_width - gaps as u16 * 2,
            height: conn.screen_height - 26 - gaps as u16 * 2,
            border_width: 0,
            // sibling: win,
            // stack_mode: xcb::x::StackMode::Above,
        };

        let border_width: u16 = 5;
        for window in self.windows.iter() {
            // connection.stop_window_events(&window);
            // conn.map_window(&window);
            conn.configure_window(&window, &window_changes, border_width);
            // connection.track_window_events(&window);
        }
    }

    fn cycle_windows(&mut self, conn: &Connection) {
        if self.windows.is_empty() {
            return;
        }
        if self.windows.len() == 1 {
            return;
        }
        println!("{:?}", self.windows);
        for win in self.windows.iter() {
            conn.conn.send_request(&xcb::x::UnmapWindow {
                window: win.get_window(),
            });
            conn.conn.flush().expect("err!");
        }

        let window: Window = self.windows.remove(self.focus_win);
        self.windows.insert(0, window);

        println!("{:?}", self.windows);
        for win in self.windows.iter() {
            conn.conn.send_request(&xcb::x::MapWindow {
                window: win.get_window(),
            });
            conn.conn.flush().expect("err!");
        }
    }

    fn floating() {
        unimplemented!();
    }
}

fn register_events(conn: &xcb::Connection, win: xcb::x::Window) {
    conn.check_request(conn.send_request_checked(&xcb::x::ChangeWindowAttributes {
        window: win,
        value_list: &[Cw::EventMask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::FOCUS_CHANGE
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::KEY_PRESS,
        )],
    }))
    .expect("register_events err!");
}

impl Connection {
    fn new() -> Self {
        let (conn, root_index) =
            xcb::Connection::connect(None).expect("couldn't connect to display");
        let builder = conn.get_setup().roots().nth(root_index as usize);

        let win: xcb::x::Window = builder.expect("root window err!").root();
        let screen = builder.unwrap();
        let screen_height = screen.height_in_pixels();
        let screen_width = screen.width_in_pixels();
        let cookie = conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(win),
        });
        let reply = conn.wait_for_reply(cookie).expect("geometry error!");
        let (x, y, width, height) = (reply.x(), reply.y(), reply.width(), reply.height());
        let root_window: Window = Window {
            win,
            x,
            y,
            width,
            height,
        };
        register_events(&conn, root_window.win);
        conn.send_request(&xcb::x::GrabKey {
            owner_events: true,
            grab_window: root_window.get_window(),
            modifiers: xcb::x::ModMask::ANY,
            key: 25,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });
        conn.send_request(&xcb::x::GrabKey {
            owner_events: true,
            grab_window: root_window.get_window(),
            modifiers: xcb::x::ModMask::ANY,
            key: 19,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });

        Self {
            conn,
            root_window,
            root_index,
            screen_height,
            screen_width,
        }
    }
    fn map_req(&self, win: xcb::x::Window, layouts: &mut Layouts) {
        if Window::new(&self, win).height < 50 {
            self.conn.send_request(&xcb::x::MapWindow { window: win });
            return;
        }
        layouts.windows.push(Window::new(&self, win));
        layouts.max(&self, win);
        layouts.focus_win = layouts.windows.len() - 1;
        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window: win,
            value_list: &[xcb::x::Cw::EventMask(
                xcb::x::EventMask::EXPOSURE
                    | xcb::x::EventMask::KEY_PRESS
                    | xcb::x::EventMask::STRUCTURE_NOTIFY
                    | xcb::x::EventMask::PROPERTY_CHANGE,
            )],
        });
        self.conn.send_request(&xcb::x::MapWindow { window: win });
    }
    fn configure_request(&self, event: ConfigureRequestEvent) {
        if Window::new(&self, event.window()).height < 50 {
            return;
        }
        let border_width: u16 = 0;
        let win = Window::new(&self, event.window());
        let cwin = WindowChanges {
            x: event.x(),
            y: event.y(),
            width: event.width(),
            height: event.height(),
            border_width: event.border_width(),
            // sibling: event.sibling(),
            // stack_mode: event.stack_mode(),
        };

        self.configure_window(&win, &cwin, border_width)
    }

    fn configure_window(&self, win: &Window, cwin: &WindowChanges, border_width: u16) {
        println!("{:?}", win.get_window());
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window: win.get_window(),
            value_list: &[
                x::ConfigWindow::X(cwin.x.into()),
                x::ConfigWindow::Y(cwin.y.into()),
                x::ConfigWindow::Width((cwin.width - (border_width * 2)).into()),
                x::ConfigWindow::Height((cwin.height - (border_width * 2)).into()),
                x::ConfigWindow::BorderWidth((cwin.border_width + border_width).into()),
                // x::ConfigWindow::Sibling(cwin.sibling.into()),
                // x::ConfigWindow::StackMode(cwin.stack_mode.into()),
            ],
        });
        self.conn.flush().expect("err!");
    }

    fn desroy_notify(&self, layouts: &mut Layouts) {
        if layouts.windows.is_empty() {
            return;
        }
        layouts.windows.remove(layouts.focus_win);
        layouts.focus_win = layouts.windows.len() - 1;
    }

    fn get_geometry(&self, win: xcb::x::Window) -> (i16, i16, u16, u16) {
        let cookie = self.conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(win),
        });
        let reply = self.conn.wait_for_reply(cookie).expect("geometry error!");
        (reply.x(), reply.y(), reply.width(), reply.height())
    }
}

fn main() {
    let mut layouts = Layouts::new();
    let conn = Connection::new();
    loop {
        let event = match conn.conn.wait_for_event() {
            Err(xcb::Error::Connection(xcb::ConnError::Connection)) => {
                break;
            }
            Err(e) => {
                panic!("unexpected error {:#?}", e);
            }
            Ok(e) => e,
        };
        match event {
            xcb::Event::X(x::Event::KeyPress(e)) => {
                println!("{:#?}", e.detail());
                if e.detail() == 24 {
                    break;
                } else if e.detail() == 25 {
                    std::process::Command::new("dmenu_run")
                        .spawn()
                        .expect("launch err!");
                } else if e.detail() == 26 {
                    layouts.cycle_windows(&conn);
                }
            }
            xcb::Event::X(x::Event::CreateNotify(e)) => {}
            xcb::Event::X(x::Event::ConfigureRequest(e)) => conn.configure_request(e),
            xcb::Event::X(x::Event::MapRequest(e)) => conn.map_req(e.window(), &mut layouts),
            xcb::Event::X(x::Event::MapNotify(e)) => {}
            xcb::Event::X(x::Event::MappingNotify(e)) => {}
            xcb::Event::X(x::Event::LeaveNotify(e)) => {}
            xcb::Event::X(x::Event::DestroyNotify(e)) => conn.desroy_notify(&mut layouts),
            _ => {}
        }
        conn.conn.flush().expect("err!");
    }
}
