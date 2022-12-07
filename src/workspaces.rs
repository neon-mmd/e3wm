use crate::{config_parser::ParsedConfig, layouts::Layouts};

pub struct Workspaces {
    pub workspaces: Vec<Layouts>,
    pub focus_workspace: usize,
}

impl Workspaces {
    pub fn new(config: &ParsedConfig) -> Self {
        let mut workspaces: Vec<Layouts> = Vec::new();
        for i in config.workspaces.iter() {
            workspaces.push(Layouts::new(config));
        }
        Self {
            workspaces,
            focus_workspace: 0,
        }
    }
}
