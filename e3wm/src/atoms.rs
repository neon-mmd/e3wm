use api::config_parser::ParsedConfig;
use xcb::x;

use crate::x::Window;

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct InternedAtoms {
    pub _NET_NUMBER_OF_DESKTOPS: xcb::x::Atom,
    pub _NET_CURRENT_DESKTOP: xcb::x::Atom,
    pub _NET_ACTIVE_WINDOW: xcb::x::Atom,
    pub _NET_DESKTOP_NAMES: xcb::x::Atom,
    pub WM_CLASS: xcb::x::Atom,
    pub WM_NAME: xcb::x::Atom,
}

#[allow(non_snake_case)]
impl InternedAtoms {
    pub fn new(conn: &xcb::Connection) -> Self {
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: b"_NET_NUMBER_OF_DESKTOPS",
        });
        let reply = conn.wait_for_reply(cookie).expect("err!");
        let _NET_NUMBER_OF_DESKTOPS = reply.atom();
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: b"_NET_CURRENT_DESKTOP",
        });
        let reply = conn.wait_for_reply(cookie).expect("err!");
        let _NET_CURRENT_DESKTOP = reply.atom();
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: b"_NET_ACTIVE_WINDOW",
        });
        let reply = conn.wait_for_reply(cookie).expect("err!");
        let _NET_ACTIVE_WINDOW = reply.atom();
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: b"WM_CLASS",
        });
        let reply = conn.wait_for_reply(cookie).expect("err!");
        let WM_CLASS = reply.atom();
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: b"WM_NAME",
        });
        let reply = conn.wait_for_reply(cookie).expect("err!");
        let WM_NAME = reply.atom();
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: b"_NET_DESKTOP_NAMES",
        });
        let reply = conn.wait_for_reply(cookie).expect("err!");
        let _NET_DESKTOP_NAMES = reply.atom();

        Self {
            _NET_NUMBER_OF_DESKTOPS,
            _NET_CURRENT_DESKTOP,
            _NET_ACTIVE_WINDOW,
            _NET_DESKTOP_NAMES,
            WM_CLASS,
            WM_NAME,
        }
    }

    pub fn setup(&self, config: &ParsedConfig, conn: &xcb::Connection, win: &Window) {
        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: win.get_window(),
            property: self._NET_NUMBER_OF_DESKTOPS,
            r#type: x::ATOM_CARDINAL,
            data: &config.workspaces.len().to_string().as_bytes(),
        });

        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: win.get_window(),
            property: self._NET_DESKTOP_NAMES,
            r#type: x::ATOM_STRING,
            data: "1,2".as_bytes(),
        });

        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: win.get_window(),
            property: self._NET_CURRENT_DESKTOP,
            r#type: x::ATOM_CARDINAL,
            data: "0".as_bytes(),
        });
    }
}
