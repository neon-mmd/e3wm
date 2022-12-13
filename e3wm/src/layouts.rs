use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};

use crate::{
    wm_layouts::max::max,
    wm_layouts::tile::{self, tile},
    x::{Connection, Window, WindowChanges},
};
use api::config_parser::ParsedConfig;

pub enum WmLayouts {
    Tile,
    Float,
    Max,
}

pub struct Layouts {
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

    pub fn split_window(&self, conn: &Connection, vertical: bool) {
        if self.windows.is_empty() {
            return;
        }
        todo!();
    }

    pub fn cycle_window_forward(&mut self, conn: &Connection) {
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

    pub fn cycle_window_focus_forward(&mut self, conn: &Connection) {
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

    pub fn cycle_window_backward(&mut self, conn: &Connection) {
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

    pub fn cycle_window_focus_backward(&mut self, conn: &Connection) {
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

    pub fn floating() {
        unimplemented!();
    }
}
