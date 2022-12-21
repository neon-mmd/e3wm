use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask, KeyButMask},
    Xid,
};

use api::config_parser::ParsedConfig;

use crate::{
    atoms::InternedAtoms,
    keybinding_parser::Keybindings,
    layouts::{Layouts, WmLayouts},
    wm_layouts::{max::max, tile::tile},
    workspaces::Workspaces,
};

#[derive(Debug, Hash)]
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
    pub keybindings: Keybindings,
    pub atoms: InternedAtoms,
    pub bar_height: u16,
    vertical: bool,
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
    pub fn new(config: &ParsedConfig) -> Self {
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

        register_events(&conn, root_window.get_window());
        let mut keybindings = Keybindings::new();
        keybindings.string_to_keycodes(&config, &conn);
        // println!("{:#?}",keybindings)
        let atoms = InternedAtoms::new(&conn);
        atoms.setup(config, &conn, &root_window);

        Self::grab_keys(&mut keybindings, &conn, &root_window);

        Self {
            conn,
            root_window,
            root_index,
            screen_height,
            screen_width,
            focus_win_border_color,
            unfocus_win_border_color,
            keybindings,
            atoms,
            bar_height: 0,
            vertical: true,
        }
    }

    fn grab_keys(keybindings: &mut Keybindings, conn: &xcb::Connection, root: &Window) {
        for (keys, modifiers) in keybindings.keyseqs.iter() {
            let (mods, _) = modifiers;
            for key in keys {
                for modifiers in [
                    xcb::x::ModMask::from_bits_truncate(mods.bits()),
                    xcb::x::ModMask::from_bits_truncate(mods.bits()) | xcb::x::ModMask::LOCK,
                ] {
                    conn.send_request(&xcb::x::GrabKey {
                        owner_events: true,
                        grab_window: root.get_window(),
                        modifiers,
                        key: *key,
                        pointer_mode: xcb::x::GrabMode::Async,
                        keyboard_mode: xcb::x::GrabMode::Async,
                    });
                }
            }
        }
    }

    pub fn handle_key_press(
        &self,
        workspace: &mut Workspaces,
        e: xcb::x::KeyPressEvent,
        config: &ParsedConfig,
    ) {
        for (keys, modifiers) in self.keybindings.keyseqs.iter() {
            let (mods, cmd) = modifiers;
            for key in keys.iter() {
                if *key == e.detail() {
                    if e.state() == *mods || e.state() == *mods | xcb::x::KeyButMask::MOD2 {
                        let cmd: Vec<&str> = cmd[0].split(",").collect();
                        if cmd[0] == "quit" {
                            std::process::exit(0);
                        } else if cmd[0] == "change_layout" {
                            workspace.get_current_layout().change_layout(config, &self);
                        } else if cmd[0] == "cycle_window_focus_forward" {
                            workspace
                                .get_current_layout()
                                .cycle_window_focus_forward(&self, config);
                        } else if cmd[0] == "goto_tag" {
                            workspace
                                .go_to_tag(cmd[1].to_string().parse::<usize>().unwrap() - 1, &self);
                        } else {
                            std::process::Command::new(cmd[0].clone()).spawn();
                        }
                    }
                }
            }
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

    pub fn split_vertically(&mut self) {
        self.vertical = true
    }

    pub fn split_horizontally(&mut self) {
        self.vertical = false
    }

    pub fn map_req(
        &mut self,
        win: xcb::x::Window,
        workspace: &mut Workspaces,
        config: &ParsedConfig,
    ) {
        let layouts = workspace.get_current_layout();
        let cookie = self.conn.send_request(&x::GetProperty {
            delete: false,
            window: win,
            property: self.atoms.WM_CLASS,
            r#type: x::ATOM_STRING,
            long_offset: 0,
            long_length: 8,
        });

        let reply = self.conn.wait_for_reply(cookie).expect("err!!");
        let title =
            std::str::from_utf8(reply.value()).expect("The WM_NAME property is not valid UTF-8");

        if title.contains("bar") {
            self.bar_height = self.get_geometry(win).3;
            self.conn.send_request(&xcb::x::MapWindow { window: win });
            return;
        }
        if config.dynamic == true {
            layouts.windows.push(Window::new(&self, win));
            layouts.focus_win = layouts.windows.len() - 1;

            match layouts.current_layout {
                WmLayouts::Tile => {
                    tile(layouts, self);
                }
                WmLayouts::Max => {
                    max(layouts, self);
                }
                WmLayouts::Float => {
                    println!("to be implemented!!");
                }
            }
        } else {
            if self.vertical == true {
                layouts.split_window_horizontal(&self, Window::new(&self, win), config);
            } else {
                layouts.split_window_horizontal(&self, Window::new(&self, win), config);
            }
        }
        self.register_events_on_windows(win);
        self.conn.send_request(&xcb::x::MapWindow { window: win });
    }
    pub fn configure_request(&self, event: ConfigureRequestEvent) {
        if Window::new(&self, event.window()).height == self.bar_height {
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

    pub fn kill_window(&self, workspace: &mut Workspaces, config: &ParsedConfig) {
        let layouts = workspace.get_current_layout();
        if config.dynamic == true {
            if layouts.windows.is_empty() {
                return;
            }
            self.conn.send_request(&xcb::x::DestroyWindow {
                window: layouts.windows[layouts.focus_win].get_window(),
            });
        }
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

        let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
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
        let reply = self.conn.check_request(cookie);
        if reply.is_err() {
            return;
        }
        self.conn.flush().expect("err!");
    }

    pub fn destroy_notify(
        &self,
        workspace: &mut Workspaces,
        win: xcb::x::Window,
        config: &ParsedConfig,
    ) {
        let layouts = workspace.get_current_layout();
        if config.dynamic == true {
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
            match layouts.current_layout {
                WmLayouts::Tile => {
                    tile(layouts, &self);
                }
                _ => {
                    return;
                }
            }
        } else {
            if layouts.nodes.is_empty() {
                return;
            }
            if layouts.nodes[layouts.focus_win]
                .value
                .as_mut()
                .unwrap()
                .get_window()
                != win
            {
                return;
            }

            layouts.remove();
            if layouts.nodes.is_empty() {
                return;
            }
            layouts.resize(&self);
            self.update_focus(
                layouts.nodes[layouts.focus_win]
                    .value
                    .as_mut()
                    .unwrap()
                    .get_window(),
            );
        }
    }

    pub fn get_geometry(&self, win: xcb::x::Window) -> (i16, i16, u16, u16) {
        let cookie = self.conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(win),
        });
        let reply = self.conn.wait_for_reply(cookie).expect("geometry error!");
        (reply.x(), reply.y(), reply.width(), reply.height())
    }
}
