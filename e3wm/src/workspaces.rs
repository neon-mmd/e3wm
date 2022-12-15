use crate::atoms::InternedAtoms;
use crate::layouts::Layouts;
use crate::x::Connection;
use api::config_parser::ParsedConfig;

pub struct Workspaces {
    pub workspaces: Vec<Layouts>,
    pub focus_workspace: usize,
}

impl Workspaces {
    pub fn new(config: &ParsedConfig) -> Self {
        let mut workspaces: Vec<Layouts> = Vec::new();
        for _ in config.workspaces.iter() {
            workspaces.push(Layouts::new(config));
        }

        Self {
            workspaces,
            focus_workspace: 0,
        }
    }

    pub fn go_to_tag(&mut self, tag: usize, conn: &Connection) {
        for window in self.workspaces[self.focus_workspace].windows.iter() {
            conn.conn.send_request(&xcb::x::UnmapWindow {
                window: window.get_window(),
            });
        }
        self.focus_workspace = tag;
        for window in self.workspaces[self.focus_workspace].windows.iter() {
            conn.conn.send_request(&xcb::x::MapWindow {
                window: window.get_window(),
            });
        }
    }

    pub fn get_current_layout(&mut self) -> &mut Layouts {
        &mut self.workspaces[self.focus_workspace]
    }
}
