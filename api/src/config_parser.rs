use std::collections::HashMap;
use std::path::Path;
use std::{fs::File, io::Read};

use pyo3::prelude::*;
// use pyo3::types::PyTuple;

#[derive(Debug)]
pub struct ParsedConfig {
    pub workspaces: Vec<i32>,
    pub enabled_layouts: Vec<String>,
    pub dynamic: bool,
    pub keybindings: Option<HashMap<String, [String; 3]>>,
}

impl ParsedConfig {
    pub fn new() -> Self {
        let workspaces: Vec<i32> = Vec::new();
        let enabled_layouts: Vec<String> = Vec::new();
        let dynamic: bool = true;
        let keybindings: HashMap<String, [String; 3]> = HashMap::new();
        Self {
            workspaces,
            enabled_layouts,
            dynamic,
            keybindings: Some(keybindings),
        }
    }

    pub fn parse(&mut self) {
        let mut file =
            File::open(Path::new("/opt/e3wm/handler.py")).expect("config file doesn't exists");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("failed to read from file");

        Python::with_gil(|py| {
            let fun = PyModule::from_code(py, &contents, "", "").unwrap();

            let parser = fun.getattr("parse").unwrap();
            let parser = parser.call0().unwrap();
            self.workspaces =
                Self::_retrieve_vector_integer_32_helper(parser, String::from("get_workspaces"));

            self.enabled_layouts =
                Self::_retrieve_vector_string_helper(parser, String::from("get_layouts"));
            self.dynamic = Self::_retrieve_boolean_helper(parser, String::from("get_dynamic"));

            let mut cnt = 0;
            loop {
                let keybinding: Option<(String, String, String, String)> = parser
                    .call_method1("get_keybindings", (cnt,))
                    .expect("failed to get keybindings")
                    .extract()
                    .expect("failed to get value");
                match keybinding {
                    Some(e) => {
                        let (key, cmd, grp, desc) = e;
                        match &mut self.keybindings {
                            Some(e) => e.insert(key, [cmd, grp, desc]),
                            None => break,
                        };
                    }
                    None => break,
                }
                cnt += 1;
            }
        });
    }

    fn _retrieve_boolean_helper(parser: &PyAny, to_get: String) -> bool {
        return parser
            .call_method0(to_get.as_str())
            .expect("failed to call function")
            .extract()
            .expect("failed to get value");
    }

    fn _retrieve_vector_string_helper(parser: &PyAny, to_get: String) -> Vec<String> {
        return parser
            .call_method0(to_get.as_str())
            .expect("failed to call function")
            .extract()
            .expect("failed to get value");
    }

    fn _retrieve_vector_integer_32_helper(parser: &PyAny, to_get: String) -> Vec<i32> {
        return parser
            .call_method0(to_get.as_str())
            .expect("failed to call function")
            .extract()
            .expect("failed to get value");
    }
}
