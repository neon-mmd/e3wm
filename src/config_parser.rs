use std::path::Path;
use std::{fs::File, io::Read};

use pyo3::prelude::*;
// use pyo3::types::PyTuple;

#[derive(Debug)]
pub struct ParsedConfig {
    pub workspaces: Vec<i32>,
    pub enabled_layouts: Vec<String>,
    pub dynamic: bool,
}

impl ParsedConfig {
    pub fn new() -> Self {
        let mut workspaces: Vec<i32> = Vec::new();
        let mut enabled_layouts: Vec<String> = Vec::new();
        let mut dynamic: bool = true;

        let mut file = File::open(Path::new("/home/destruct/.config/e3wm/config.py"))
            .expect("config file doesn't exists");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("failed to read from file");

        Python::with_gil(|py| {
            let fun = PyModule::from_code(py, &contents, "", "").unwrap();

            workspaces =
                Self::_retrieve_vector_integer_32_helper(fun, String::from("get_workspaces"));

            enabled_layouts =
                Self::_retrieve_vector_string_helper(fun, String::from("get_layouts"));
            dynamic = Self::_retrieve_boolean_helper(fun, String::from("get_dynamic"));
        });

        Self {
            workspaces,
            enabled_layouts,
            dynamic,
        }
    }
    fn _retrieve_boolean_helper(fun: &PyModule, to_get: String) -> bool {
        return fun
            .getattr(to_get.as_str())
            .expect("no such attribute")
            .call0()
            .expect("failed to call function")
            .extract()
            .expect("failed to get value");
    }

    fn _retrieve_vector_string_helper(fun: &PyModule, to_get: String) -> Vec<String> {
        return fun
            .getattr(to_get.as_str())
            .expect("no such attribute")
            .call0()
            .expect("failed to call function")
            .extract()
            .expect("failed to get value");
    }

    fn _retrieve_vector_integer_32_helper(fun: &PyModule, to_get: String) -> Vec<i32> {
        return fun
            .getattr(to_get.as_str())
            .expect("no such attribute")
            .call0()
            .expect("failed to call function")
            .extract()
            .expect("failed to get value");
    }
}
