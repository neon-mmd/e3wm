use xcb::{
    x::{self, ConfigureRequestEvent, Cw, EventMask},
    Xid,
};
use E3WM::x::Connection;
use E3WM::{config_parser::ParsedConfig, layouts::Layouts, workspaces::Workspaces};

fn main() {
    let config = ParsedConfig::new();
    let conn = Connection::new();
    let mut layouts = Layouts::new(&config);
    println!("{:#?}", config);
    let workspaces = Workspaces::new(&config);
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
                if e.detail() == 24 {
                    if e.state() == xcb::x::KeyButMask::CONTROL
                        || e.state() == xcb::x::KeyButMask::CONTROL | xcb::x::KeyButMask::MOD2
                    {
                        break;
                    }
                }
                if e.detail() == 25 {
                    if e.state() == xcb::x::KeyButMask::CONTROL
                        || e.state() == xcb::x::KeyButMask::CONTROL | xcb::x::KeyButMask::MOD2
                    {
                        std::process::Command::new("dmenu_run")
                            .spawn()
                            .expect("launch err!");
                    }
                }
                if e.detail() == 26 {
                    if e.state() == xcb::x::KeyButMask::CONTROL
                        || e.state() == xcb::x::KeyButMask::CONTROL | xcb::x::KeyButMask::MOD2
                    {
                        layouts.cycle_forward(&conn);
                    }
                    // println!("{:#?}", e.detail());
                }
                if e.detail() == 27 {
                    if e.state() == xcb::x::KeyButMask::CONTROL
                        || e.state() == xcb::x::KeyButMask::CONTROL | xcb::x::KeyButMask::MOD2
                    {
                        layouts.cycle_backward(&conn);
                    }
                    // println!("{:#?}", e);
                }
                if e.detail() == 28 {
                    if e.state() == xcb::x::KeyButMask::CONTROL
                        || e.state() == xcb::x::KeyButMask::CONTROL | xcb::x::KeyButMask::MOD2
                    {
                        conn.kill_window(&mut layouts);
                    }
                }
            }
            xcb::Event::X(x::Event::CreateNotify(e)) => {}
            xcb::Event::X(x::Event::ConfigureRequest(e)) => conn.configure_request(e),
            xcb::Event::X(x::Event::MapRequest(e)) => conn.map_req(e.window(), &mut layouts),
            xcb::Event::X(x::Event::MapNotify(e)) => {}
            xcb::Event::X(x::Event::MappingNotify(e)) => {}
            xcb::Event::X(x::Event::LeaveNotify(e)) => {}
            xcb::Event::X(x::Event::DestroyNotify(e)) => {
                conn.destroy_notify(&mut layouts, e.window())
            }
            _ => {}
        }
        conn.conn.flush().expect("err!");
    }
}
