use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::x::Window;

#[derive(Debug)]
pub struct Tree {
    pub parent: Option<Box<Tree>>,
    pub value: Window,
    pub hash: u64,
    pub vertical: bool,
    pub left: Option<Box<Tree>>,
    pub right: Option<Box<Tree>>,
}

impl Tree {
    pub fn new(win: Window, parent: Option<Box<Tree>>) -> Self {
        let hash = Self::calc_hash(&win);
        Self {
            parent,
            value: win,
            hash,
            vertical:true,
            left: None,
            right: None,
        }
    }
    fn calc_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    pub fn search_win(root: Box<Tree>, win: &Window) -> Option<Box<Tree>> {
        if root.value.get_window() == win.get_window() {
            return Some(root);
        };

        match root.left {
            Some(e) => Self::search_win(e, &win),
            None => None,
        };

        match root.right {
            Some(e) => Self::search_win(e, &win),
            None => None,
        }
    }
}
