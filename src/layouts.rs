use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};

use crate::x::{Connection, Window, WindowChanges};

pub enum WmLayouts {
    Tile,
    Float,
    Max,
}

pub struct Workspaces {
    pub workspaces: Vec<Layouts>,
    pub focus_workspace: usize,
}

pub struct Layouts {
    pub windows: Vec<Window>,
    pub focus_win: usize,
    pub current_layout: WmLayouts,
}

impl Layouts {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            focus_win: 0,
            current_layout: WmLayouts::Tile,
        }
    }

    pub fn tile(&self, conn: &Connection) {
        if self.windows.is_empty() {
            return;
        }
        if self.windows.len() == 1 {
            self.max(&conn);
            return;
        }
        let gaps: i16 = 5;
        let border_width: u16 = 5;
        let bar_height: i16 = 26;

        let master_width = (conn.screen_width as f32 * 0.5) as u16;
        let master_height = conn.screen_height as u16;

        let num_stack_win = self.windows.len() - 1;
        let mut j = 0;

        let mut track_y = 0;
        for win in self.windows.iter() {
            if j == 0 {
                let cwin = WindowChanges {
                    x: 0,
                    y: 0,
                    width: master_width,
                    height: master_height,
                    border_width: 0,
                };
                conn.configure_window(&win, &cwin, border_width, bar_height, gaps);
            } else if j == 1 {
                let cwin = WindowChanges {
                    x: master_width as i16,
                    y: track_y,
                    width: master_width,
                    height: master_height / num_stack_win as u16,
                    border_width: 0,
                };
                conn.configure_window(win, &cwin, border_width, bar_height, gaps);
                track_y += master_height as i16 / num_stack_win as i16;
            } else {
                println!("{}", track_y);
                let cwin = WindowChanges {
                    x: master_width as i16,
                    y: track_y - bar_height,
                    width: master_width,
                    height: master_height / num_stack_win as u16 + bar_height as u16,
                    border_width: 0,
                };
                conn.configure_window(win, &cwin, border_width, bar_height, gaps);
                track_y += master_height as i16 / num_stack_win as i16;
            }
            if self.focus_win == j {
                conn.configure_border(self.windows[self.focus_win].get_window(), true);
            } else {
                conn.configure_border(self.windows[j as usize].get_window(), false);
            }
            j += 1;
        }
    }

    pub fn max(&self, conn: &Connection) {
        if self.windows.is_empty() {
            return;
        }

        let window_changes = WindowChanges {
            x: 0,
            y: 0,
            width: conn.screen_width,
            height: conn.screen_height,
            border_width: 0,
            // sibling: win,
            // stack_mode: xcb::x::StackMode::Above,
        };

        let gaps: i16 = 5;
        let border_width: u16 = 5;
        let bar_height: i16 = 26;
        for window in self.windows.iter() {
            // connection.stop_window_events(&window);
            // conn.map_window(&window);
            conn.configure_window(&window, &window_changes, border_width, bar_height, gaps);
            // connection.track_window_events(&window);
        }
        conn.configure_border(self.windows[self.focus_win].get_window(), true);
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
