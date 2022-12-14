use std::collections::HashMap;

use xcb::x::{KeyButMask, Keycode};

use api::config_parser::ParsedConfig;
use xkbcommon::xkb::keysyms;

#[derive(Debug)]
pub struct Keybindings {
    pub keyseqs: HashMap<Vec<Keycode>, (KeyButMask, [String; 3])>,
}

impl Keybindings {
    pub fn new() -> Self {
        Self {
            keyseqs: HashMap::new(),
        }
    }

    pub fn string_to_keycodes(&mut self, config: &ParsedConfig, conn: &xcb::Connection) {
        let keybindings = match &config.keybindings {
            Some(e) => e,
            None => return,
        };
        for (key, modifier) in keybindings.iter() {
            if key.trim().contains(" ") {
                panic!("keychords are not supported right now!");
            } else {
                let modifiers: Vec<&str> = key.trim().split("-").collect();
                let mut modifiers_modmask: KeyButMask = KeyButMask::empty();
                let mut keysym: xkbcommon::xkb::Keysym = 0;
                let mut keycodes: Vec<Keycode> = Vec::new();
                for keys in 0..modifiers.len() - 1 {
                    modifiers_modmask |= match modifiers[keys] {
                        "C" => KeyButMask::CONTROL,
                        "S" => KeyButMask::SHIFT,
                        "M" => KeyButMask::MOD4,
                        "A" => KeyButMask::MOD1,
                        _ => panic!("err!"),
                    };
                }
                if modifiers[modifiers.len() - 1] != "S"
                    && modifiers[modifiers.len() - 1] != "M"
                    && modifiers[modifiers.len() - 1] != "A"
                    && modifiers[modifiers.len() - 1] != "C"
                {
                    let last = modifiers.len() - 1;
                    if modifiers[last].ends_with(">") && modifiers[last].starts_with("<") {
                        if modifiers[last].to_lowercase() == "<tab>" {
                            keysym = keysyms::KEY_Tab;
                        } else if modifiers[last].to_lowercase() == "<enter>" {
                            keysym = keysyms::KEY_Return;
                        } else {
                            keysym = keysyms::KEY_Escape;
                        }
                    } else if (modifiers[last].ends_with(">") && !modifiers[last].starts_with("<"))
                        || (!modifiers[last].ends_with(">") && modifiers[last].starts_with("<"))
                    {
                        panic!("Unrecognized keys");
                    } else {
                        keysym = xkbcommon::xkb::keysym_from_name(
                            &modifiers[modifiers.len() - 1].to_lowercase(),
                            xkbcommon::xkb::KEYSYM_NO_FLAGS,
                        );

                        if keysym == xkbcommon::xkb::KEY_NoSymbol {
                            panic!("Unrecognized key");
                        }
                    }
                    keycodes = Self::_convert_to_keycode(&conn, keysym);
                }
                self.keyseqs
                    .insert(keycodes, (modifiers_modmask, modifier.clone()));
            }
        }
    }

    fn _convert_to_keycode(conn: &xcb::Connection, keysym: xkbcommon::xkb::Keysym) -> Vec<Keycode> {
        let setup = conn.get_setup();
        let min_keycode = setup.min_keycode();
        let max_keycode = setup.max_keycode();

        let cookie = conn.send_request(&xcb::x::GetKeyboardMapping {
            first_keycode: min_keycode,
            count: max_keycode - min_keycode + 1,
        });

        let reply: xcb::x::GetKeyboardMappingReply = conn.wait_for_reply(cookie).unwrap();
        let per = reply.keysyms_per_keycode() as usize;
        let keysyms = reply.keysyms();

        let mut keycodes = Vec::new();

        for col in 0..per {
            for keycode in min_keycode..=max_keycode {
                let keysym_group = (keycode - min_keycode) as usize * per;

                match keysyms.get(keysym_group + col) {
                    Some(ks) if *ks == keysym => {
                        keycodes.push(keycode);
                    }
                    _ => {}
                }
            }
        }

        keycodes.dedup();
        keycodes
    }
}
