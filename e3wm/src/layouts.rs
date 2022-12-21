use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask, WindowClass},
    Xid,
};

use crate::{
    wm_layouts::max::max,
    wm_layouts::tile::{self, tile},
    x::{Connection, Window, WindowChanges},
};
use api::config_parser::ParsedConfig;

#[derive(Debug)]
pub struct Node {
    pub parent: Option<usize>,
    pub value: Option<Window>,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub vertical: Option<bool>,
}

pub enum WmLayouts {
    Tile,
    Float,
    Max,
}

pub struct Layouts {
    pub nodes: Vec<Node>,
    pub windows: Vec<Window>,
    pub focus_win: usize,
    pub current_layout: WmLayouts,
}

impl Layouts {
    pub fn new(config: &ParsedConfig) -> Self {
        let default_layout = match config.enabled_layouts.get(0) {
            Some(e) => e,
            None => panic!("failed to get value from vector"),
        };
        Self {
            windows: Vec::new(),
            focus_win: 0,
            current_layout: Self::get_enum_from_layout(default_layout),
            nodes: Vec::new(),
        }
    }

    pub fn resize(&mut self, conn: &Connection) {
        let parent = match self.nodes[self.focus_win].parent {
            Some(value) => value,
            None => {
                let current = self.nodes[self.focus_win].value.as_mut().unwrap();
                let cwin = WindowChanges {
                    x: current.x,
                    y: current.y,
                    width: current.width * 2 + conn.bar_height + 5,
                    height: current.height + 10,
                    border_width: 0,
                };
                conn.configure_window(&current, &cwin, 5, 0, 0);
                return;
            }
        };
        let vertical = match self.nodes[parent].vertical {
            Some(value) => value,
            None => return,
        };

        if vertical == true {
            let current = self.nodes[self.focus_win].value.as_mut().unwrap();
            let cwin = WindowChanges {
                x: current.x,
                y: current.y,
                width: current.width * 2 + conn.bar_height + 5,
                height: current.height + 10,
                border_width: 0,
            };
            conn.configure_window(&current, &cwin, 5, 0, 0);
        } else {
            let current = self.nodes[self.focus_win].value.as_mut().unwrap();
            let cwin = WindowChanges {
                x: current.x,
                y: current.y,
                width: current.width,
                height: current.height * 2,
                border_width: 0,
            };
            conn.configure_window(&current, &cwin, 0, 0, 0);
        }
    }

    pub fn remove(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        let parent_index = match self.nodes[self.focus_win].parent {
            Some(value) => value,
            None => {
                self.nodes.remove(self.focus_win);
                self.focus_win = 0;
                return;
            }
        };
        let parent = &self.nodes[parent_index];
        if parent.left.unwrap_or_default() == self.focus_win {
            self.nodes.remove(self.focus_win);
            self.focus_win = parent_index;
            self.nodes[self.focus_win] = Node {
                parent: self.nodes[self.focus_win].parent,
                value: self
                    .nodes
                    .remove(self.nodes[self.focus_win].right.unwrap())
                    .value,
                left: None,
                right: None,
                vertical: None,
            };
        } else if parent.right.unwrap() == self.focus_win {
            self.nodes.remove(self.focus_win);
            self.focus_win = parent_index;
            self.nodes[self.focus_win] = Node {
                parent: self.nodes[self.focus_win].parent,
                value: self
                    .nodes
                    .remove(self.nodes[self.focus_win].left.unwrap())
                    .value,
                left: None,
                right: None,
                vertical: None,
            };
        }
    }

    fn insert(&mut self, win: Window, vertical: bool, conn: &Connection) {
        if self.nodes.is_empty() {
            self.nodes.push(Node {
                parent: None,
                value: Some(Window::new(conn, win.get_window())),
                left: None,
                right: None,
                vertical: None,
            });
            conn.configure_border(win.get_window(), true);
        } else {
            // let new_parent = self.nodes[self.focus_win];
            let left = Node {
                parent: Some(self.focus_win),
                value: Some(Window::new(
                    conn,
                    self.nodes[self.focus_win]
                        .value
                        .as_mut()
                        .unwrap()
                        .get_window(),
                )),
                left: None,
                right: None,
                vertical: None,
            };
            let right = Node {
                parent: Some(self.focus_win),
                value: Some(Window::new(conn, win.get_window())),
                left: None,
                right: None,
                vertical: None,
            };
            self.nodes.push(left);
            self.nodes.push(right);
            self.nodes[self.focus_win] = Node {
                parent: self.nodes[self.focus_win].parent,
                value: None,
                left: Some(self.nodes.len() - 1),
                right: Some(self.nodes.len() - 2),
                vertical: Some(vertical),
            };

            let left = self.nodes[self.focus_win].right.unwrap();
            conn.configure_border(self.nodes[left].value.as_mut().unwrap().get_window(), false);
            self.focus_win = self.nodes.len() - 1;
            conn.configure_border(win.get_window(), true);
        }
    }

    fn get_enum_from_layout(default_layout: &String) -> WmLayouts {
        if default_layout == "tile" {
            return WmLayouts::Tile;
        } else if default_layout == "max" {
            return WmLayouts::Max;
        } else {
            return WmLayouts::Float;
        }
    }

    fn get_layout_from_enum(&self) -> String {
        return match self.current_layout {
            WmLayouts::Tile => String::from("tile"),
            WmLayouts::Max => String::from("max"),
            WmLayouts::Float => String::from("float"),
        };
    }

    pub fn change_layout(&mut self, config: &ParsedConfig, conn: &Connection) {
        if config.dynamic == true {
            let mut next: usize = 0;
            let current_layout = self.get_layout_from_enum();
            for i in 0..config.enabled_layouts.len() {
                if config.enabled_layouts[i] == current_layout {
                    next = i;
                    break;
                }
            }

            if next == config.enabled_layouts.len() - 1 {
                next = 0;
            } else {
                next += 1;
            }

            self.current_layout = Layouts::get_enum_from_layout(&config.enabled_layouts[next]);
            match self.current_layout {
                WmLayouts::Tile => tile(self, conn),
                WmLayouts::Max => max(self, conn),
                WmLayouts::Float => println!("todo"),
            }
        }
    }

    pub fn split_window_vertical(&mut self, conn: &Connection, win: Window, config: &ParsedConfig) {
        if config.dynamic == false {
            if self.nodes.is_empty() {
                let window_changes = WindowChanges {
                    x: 0,
                    y: 0,
                    width: conn.screen_width,
                    height: conn.screen_height,
                    border_width: 0,
                    // sibling: win,
                    // stack_mode: xcb::x::StackMode::Above,
                };

                conn.configure_window(&win, &window_changes, 5, conn.bar_height as i16, 5);
                self.insert(win, true, conn);
                return;
            }
            //  x::ConfigWindow::X((cwin.x + gaps).into()),
            //         x::ConfigWindow::Y((cwin.y + bar_height + gaps).into()),
            //         x::ConfigWindow::Width(
            //             (cwin.width - (gaps as u16 * 2) - (border_width * 2)).into(),
            //         ),
            //         x::ConfigWindow::Height(
            //             (cwin.height - bar_height as u16 - (gaps as u16 * 2) - (border_width * 2))
            //                 .into(),
            //         ),
            //         x::ConfigWindow::BorderWidth((cwin.border_width + border_width).into()),
            let gaps = 5;
            let bar_height = 26;
            let border_width = 5;
            let focused_win = Window::new(
                conn,
                self.nodes[self.focus_win]
                    .value
                    .as_mut()
                    .unwrap()
                    .get_window(),
            );
            if focused_win.width == 1 {
                return;
            }
            let cwin_of_focused_win = WindowChanges {
                x: focused_win.x - gaps,
                y: focused_win.y - bar_height - gaps,
                width: (focused_win.width + (gaps * 2) as u16 + border_width * 2) / 2,
                height: (focused_win.height
                    + (gaps * 2) as u16
                    + border_width * 2
                    + bar_height as u16),
                border_width: 0,
            };

            let cwin_of_new_window = WindowChanges {
                x: ((focused_win.width + (gaps * 2) as u16 + border_width * 2) / 2) as i16
                    + focused_win.x
                    - gaps,
                y: focused_win.y - bar_height - gaps,
                width: (focused_win.width + (gaps * 2) as u16 + border_width * 2) / 2,
                height: (focused_win.height
                    + (gaps * 2) as u16
                    + border_width * 2
                    + bar_height as u16),
                border_width: 0,
            };

            conn.configure_window(
                &focused_win,
                &cwin_of_focused_win,
                border_width,
                bar_height,
                gaps,
            );
            conn.configure_window(&win, &cwin_of_new_window, border_width, bar_height, gaps);
            self.insert(win, true, conn);
        };
    }

    pub fn split_window_horizontal(
        &mut self,
        conn: &Connection,
        win: Window,
        config: &ParsedConfig,
    ) {
        if config.dynamic == false {
            if self.nodes.is_empty() {
                let window_changes = WindowChanges {
                    x: 0,
                    y: 0,
                    width: conn.screen_width,
                    height: conn.screen_height,
                    border_width: 0,
                    // sibling: win,
                    // stack_mode: xcb::x::StackMode::Above,
                };

                conn.configure_window(&win, &window_changes, 5, conn.bar_height as i16, 5);
                self.insert(win, true, conn);
                return;
            }
            //  x::ConfigWindow::X((cwin.x + gaps).into()),
            //         x::ConfigWindow::Y((cwin.y + bar_height + gaps).into()),
            //         x::ConfigWindow::Width(
            //             (cwin.width - (gaps as u16 * 2) - (border_width * 2)).into(),
            //         ),
            //         x::ConfigWindow::Height(
            //             (cwin.height - bar_height as u16 - (gaps as u16 * 2) - (border_width * 2))
            //                 .into(),
            //         ),
            //         x::ConfigWindow::BorderWidth((cwin.border_width + border_width).into()),
            let gaps = 5;
            let bar_height = 26;
            let border_width = 5;
            let focused_win = Window::new(
                conn,
                self.nodes[self.focus_win]
                    .value
                    .as_mut()
                    .unwrap()
                    .get_window(),
            );
            let cwin_of_focused_win = WindowChanges {
                x: focused_win.x - gaps,
                y: focused_win.y - bar_height - gaps,
                width: (focused_win.width + (gaps * 2) as u16 + border_width * 2),
                height: (focused_win.height
                    + (gaps * 2) as u16
                    + border_width * 2
                    + bar_height as u16)
                    / 2,
                border_width: 0,
            };

            let cwin_of_new_window = WindowChanges {
                x: focused_win.x - gaps,
                y: ((focused_win.height + border_width * 2) / 2) as i16 + focused_win.y
                    - bar_height
                    - gaps * 3,
                width: (focused_win.width + (gaps * 2) as u16 + border_width * 2),
                height: (focused_win.height
                    + (gaps * 2) as u16
                    + border_width
                    + (bar_height * 3) as u16)
                    / 2,
                border_width: 0,
            };

            conn.configure_window(
                &focused_win,
                &cwin_of_focused_win,
                border_width,
                bar_height,
                gaps,
            );
            conn.configure_window(&win, &cwin_of_new_window, border_width, bar_height, gaps);
            self.insert(win, true, conn);
        }
    }

    pub fn cycle_window_forward(&mut self, conn: &Connection, config: &ParsedConfig) {
        if config.dynamic == true {
            if self.windows.is_empty() {
                return;
            }
            if self.windows.len() == 1 {
                return;
            }

            match &self.current_layout {
                WmLayouts::Max => {
                    let window: Window = self.windows.remove(self.focus_win);
                    conn.configure_window_stacking(
                        window.get_window(),
                        self.windows[self.focus_win - 1].get_window(),
                    );
                    conn.update_focus(self.windows[self.focus_win - 1].get_window());
                    self.windows.insert(0, window);
                }
                _ => {
                    let window_1: usize = self.focus_win;
                    let mut window_2: usize = self.focus_win;
                    if self.focus_win == 0 {
                        self.focus_win = self.windows.len() - 1;
                        window_2 = self.focus_win;
                    } else {
                        self.focus_win = self.focus_win - 1;
                        window_2 = self.focus_win;
                    }
                    let temp_win = Window::new(conn, self.windows[window_1].get_window());

                    self.windows[window_1] = Window::new(conn, self.windows[window_2].get_window());
                    self.windows[window_2] = Window::new(conn, temp_win.get_window());

                    tile(self, conn);
                }
            }
            conn.conn.flush().expect("err!");
        }
    }

    pub fn cycle_window_focus_forward(&mut self, conn: &Connection, config: &ParsedConfig) {
        if config.dynamic == true {
            if self.windows.is_empty() {
                return;
            }
            if self.windows.len() == 1 {
                return;
            }

            match &self.current_layout {
                WmLayouts::Tile => {
                    if self.focus_win == 0 {
                        conn.configure_border(self.windows[self.focus_win].get_window(), false);
                        self.focus_win = self.windows.len() - 1;
                        conn.configure_border(self.windows[self.focus_win].get_window(), true)
                    } else {
                        conn.configure_border(self.windows[self.focus_win].get_window(), false);
                        self.focus_win = self.focus_win - 1;
                        conn.configure_border(self.windows[self.focus_win].get_window(), true)
                    }
                    conn.update_focus(self.windows[self.focus_win].get_window());
                }
                _ => {
                    return;
                }
            }
            conn.conn.flush().expect("err!");
        }
    }

    pub fn cycle_window_backward(&mut self, conn: &Connection, config: &ParsedConfig) {
        if config.dynamic == true {
            if self.windows.is_empty() {
                return;
            }
            if self.windows.len() == 1 {
                return;
            }
            match &self.current_layout {
                WmLayouts::Max => {
                    let window: Window = self.windows.remove(0);
                    self.windows.push(window);
                    conn.configure_window_stacking(
                        self.windows[self.focus_win - 1].get_window(),
                        self.windows[self.focus_win].get_window(),
                    );
                    conn.update_focus(self.windows[self.focus_win].get_window());
                }
                _ => {
                    let window_1: usize = self.focus_win;
                    let mut window_2: usize = self.focus_win;
                    if self.focus_win == self.windows.len() - 1 {
                        self.focus_win = 0;
                        window_2 = self.focus_win;
                    } else {
                        self.focus_win = self.focus_win + 1;
                        window_2 = self.focus_win;
                    }
                    let temp_win = Window::new(conn, self.windows[window_1].get_window());

                    self.windows[window_1] = Window::new(conn, self.windows[window_2].get_window());
                    self.windows[window_2] = Window::new(conn, temp_win.get_window());

                    tile(self, conn);
                }
            }
            conn.conn.flush().expect("err!");
        }
    }

    pub fn cycle_window_focus_backward(&mut self, conn: &Connection, config: &ParsedConfig) {
        if config.dynamic == true {
            if self.windows.is_empty() {
                return;
            }
            if self.windows.len() == 1 {
                return;
            }
            match &self.current_layout {
                WmLayouts::Tile => {
                    if self.focus_win == self.windows.len() - 1 {
                        conn.configure_border(self.windows[self.focus_win].get_window(), false);
                        self.focus_win = 0;
                        conn.configure_border(self.windows[self.focus_win].get_window(), true)
                    } else {
                        conn.configure_border(self.windows[self.focus_win].get_window(), false);
                        self.focus_win = self.focus_win + 1;
                        conn.configure_border(self.windows[self.focus_win].get_window(), true)
                    }
                    conn.update_focus(self.windows[self.focus_win].get_window());
                }
                _ => {
                    return;
                }
            }
            conn.conn.flush().expect("err!");
        }
    }
}
