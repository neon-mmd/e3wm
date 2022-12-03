use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};

use crate::layouts::Layouts;

#[derive(Debug)]
pub struct Window {
    pub win: xcb::x::Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

pub struct WindowChanges {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    // sibling: xcb::x::Window,
    // stack_mode: xcb::x::StackMode,
}

impl Window {
    pub fn new(conn: &Connection, win: xcb::x::Window) -> Self {
        let (x, y, width, height) = conn.get_geometry(win);
        Self {
            win,
            x,
            y,
            width,
            height,
        }
    }
    pub fn get_window(&self) -> xcb::x::Window {
        self.win
    }
}

pub struct Connection {
    pub conn: xcb::Connection,
    pub root_window: Window,
    pub root_index: i32,
    pub screen_height: u16,
    pub screen_width: u16,
    pub focus_win_border_color: u32,
    pub unfocus_win_border_color: u32,
}

pub fn register_events(conn: &xcb::Connection, win: xcb::x::Window) {
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
    pub fn new() -> Self {
        let (conn, root_index) =
            xcb::Connection::connect(None).expect("couldn't connect to display");
        let builder = conn.get_setup().roots().nth(root_index as usize);

        let win: xcb::x::Window = builder.expect("root window err!").root();
        let screen = builder.unwrap();
        let screen_height = screen.height_in_pixels();
        let screen_width = screen.width_in_pixels();
        let focus_win_border_color = screen.white_pixel();
        let unfocus_win_border_color = screen.black_pixel();
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
            modifiers: xcb::x::ModMask::CONTROL,
            key: 26,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });
        conn.send_request(&xcb::x::GrabKey {
            owner_events: true,
            grab_window: root_window.get_window(),
            modifiers: xcb::x::ModMask::CONTROL,
            key: 27,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });
        conn.send_request(&xcb::x::GrabKey {
            owner_events: true,
            grab_window: root_window.get_window(),
            modifiers: xcb::x::ModMask::CONTROL,
            key: 25,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });
        conn.send_request(&xcb::x::GrabKey {
            owner_events: true,
            grab_window: root_window.get_window(),
            modifiers: xcb::x::ModMask::CONTROL,
            key: 24,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });

        Self {
            conn,
            root_window,
            root_index,
            screen_height,
            screen_width,
            focus_win_border_color,
            unfocus_win_border_color,
        }
    }

    pub fn update_focus(&self, win: xcb::x::Window) {
        self.conn.send_request(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: win,
            time: xcb::x::CURRENT_TIME,
        });
    }

    pub fn configure_window_stacking(&self, win: xcb::x::Window, sibling_win: xcb::x::Window) {
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window: win,
            value_list: &[
                xcb::x::ConfigWindow::Sibling(sibling_win),
                xcb::x::ConfigWindow::StackMode(xcb::x::StackMode::Below),
            ],
        });
    }

    pub fn register_events_on_windows(&self, win: xcb::x::Window) {
        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window: win,
            value_list: &[xcb::x::Cw::EventMask(
                xcb::x::EventMask::EXPOSURE
                    | xcb::x::EventMask::KEY_PRESS
                    | xcb::x::EventMask::STRUCTURE_NOTIFY
                    | xcb::x::EventMask::PROPERTY_CHANGE
                    | EventMask::FOCUS_CHANGE,
            )],
        });
    }

    pub fn configure_border(&self, win: xcb::x::Window, focus: bool) {
        if focus {
            self.conn.send_request(&xcb::x::ChangeWindowAttributes {
                window: win,
                value_list: &[xcb::x::Cw::BorderPixel(self.focus_win_border_color)],
            });
        } else {
            self.conn.send_request(&xcb::x::ChangeWindowAttributes {
                window: win,
                value_list: &[xcb::x::Cw::BorderPixel(self.unfocus_win_border_color)],
            });
        }
    }

    pub fn map_req(&self, win: xcb::x::Window, layouts: &mut Layouts) {
        if Window::new(&self, win).height == 26 {
            self.conn.send_request(&xcb::x::MapWindow { window: win });
            return;
        }
        layouts.windows.push(Window::new(&self, win));
        layouts.focus_win = layouts.windows.len() - 1;
        layouts.tile(&self);
        self.register_events_on_windows(win);
        self.conn.send_request(&xcb::x::MapWindow { window: win });
    }
    pub fn configure_request(&self, event: ConfigureRequestEvent) {
        if Window::new(&self, event.window()).height == 26 {
            return;
        }
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

        self.configure_window(&win, &cwin, 0, 0, 0)
    }

    pub fn kill_window(&self, layouts: &mut Layouts) {
        if layouts.windows.is_empty() {
            return;
        }
        self.conn.send_request(&xcb::x::KillClient {
            resource: layouts.windows[layouts.focus_win]
                .get_window()
                .resource_id(),
        });
        // self.conn.send_request(&xcb::x::DestroyWindow {
        // window: layouts.windows[layouts.focus_win].get_window(),
        // });
    }

    pub fn configure_window(
        &self,
        win: &Window,
        cwin: &WindowChanges,
        border_width: u16,
        bar_height: i16,
        gaps: i16,
    ) {
        // println!("{:?}", win.get_window());
        // x: 0 + gaps,
        // y: 26 + gaps,
        // width: conn.screen_width - gaps as u16 * 2,
        //height: conn.screen_height - 26 - gaps as u16 * 2,

        self.conn.send_request(&xcb::x::ConfigureWindow {
            window: win.get_window(),
            value_list: &[
                x::ConfigWindow::X((cwin.x + gaps).into()),
                x::ConfigWindow::Y((cwin.y + bar_height + gaps).into()),
                x::ConfigWindow::Width(
                    (cwin.width - (gaps as u16 * 2) - (border_width * 2)).into(),
                ),
                x::ConfigWindow::Height(
                    (cwin.height - bar_height as u16 - (gaps as u16 * 2) - (border_width * 2))
                        .into(),
                ),
                x::ConfigWindow::BorderWidth((cwin.border_width + border_width).into()),
                // x::ConfigWindow::Sibling(cwin.sibling.into()),
                // x::ConfigWindow::StackMode(cwin.stack_mode.into()),
            ],
        });
        self.conn.flush().expect("err!");
    }

    pub fn destroy_notify(&self, layouts: &mut Layouts, win: xcb::x::Window) {
        if layouts.windows.is_empty() {
            return;
        }
        if layouts.windows[layouts.focus_win].get_window() != win {
            return;
        }

        layouts.windows.remove(layouts.focus_win);
        layouts.focus_win = layouts.windows.len() - 1;
        if !layouts.windows.is_empty() {
            self.update_focus(layouts.windows[layouts.focus_win].get_window());
        }
        layouts.tile(&self);
    }

    pub fn get_geometry(&self, win: xcb::x::Window) -> (i16, i16, u16, u16) {
        let cookie = self.conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(win),
        });
        let reply = self.conn.wait_for_reply(cookie).expect("geometry error!");
        (reply.x(), reply.y(), reply.width(), reply.height())
    }
}
