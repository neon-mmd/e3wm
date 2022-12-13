use crate::x::Connection;
use crate::{layouts::Layouts, workspaces::Workspaces};
use api::config_parser::ParsedConfig;
use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};

pub struct Events {
    config: ParsedConfig,
    conn: Connection,
    layouts: Layouts,
    workspaces: Workspaces,
}

impl Events {
    pub fn new() -> Self {
        let mut config = ParsedConfig::new();
        config.parse();
        println!("{:#?}", config);
        let conn = Connection::new(&mut config);
        let layouts = Layouts::new(&mut config);
        let workspaces = Workspaces::new(&mut config);
        Self {
            config,
            conn,
            layouts,
            workspaces,
        }
    }
    pub fn handle_events(&mut self) {
        loop {
            let event = match self.conn.conn.wait_for_event() {
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
                    // println!("{:#?}", e.detail());
                    self.conn
                        .handle_key_press(&mut self.layouts, e, &self.config)
                }
                xcb::Event::X(x::Event::CreateNotify(e)) => {}
                xcb::Event::X(x::Event::ConfigureRequest(e)) => self.conn.configure_request(e),
                xcb::Event::X(x::Event::MapRequest(e)) => {
                    self.conn.map_req(e.window(), &mut self.layouts)
                }
                xcb::Event::X(x::Event::MapNotify(e)) => {}
                xcb::Event::X(x::Event::MappingNotify(e)) => {}
                xcb::Event::X(x::Event::LeaveNotify(e)) => {}
                xcb::Event::X(x::Event::DestroyNotify(e)) => {
                    self.conn.destroy_notify(&mut self.layouts, e.window())
                }
                _ => {}
            }
            self.conn.conn.flush().expect("err!");
        }
    }
}
