use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};

use crate::{
    config_parser::ParsedConfig,
    x::{Connection, Window, WindowChanges},
};

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
        let mut current_layout: WmLayouts = WmLayouts::Tile;
        if default_layout == "tile" {
            current_layout = WmLayouts::Tile;
        } else if default_layout == "max" {
            current_layout = WmLayouts::Max;
        } else {
            current_layout = WmLayouts::Float;
        }
        Self {
            windows: Vec::new(),
            focus_win: 0,
            current_layout,
        }
    }

    pub fn change_layout(&self) {
        todo!();
    }

    pub fn split_window(&self, conn: &Connection, vertical: bool) {
        if self.windows.is_empty() {
            return;
        }
        todo!();
    }

    pub fn cycle_forward(&mut self, conn: &Connection) {
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
        }
        conn.conn.flush().expect("err!");
    }

    pub fn cycle_backward(&mut self, conn: &Connection) {
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
        }
        conn.conn.flush().expect("err!");
    }

    pub fn floating() {
        unimplemented!();
    }
}
