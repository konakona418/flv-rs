use std::cell::RefCell;
use crate::core::Core;
use std::fs::Metadata;
use std::rc::Rc;

pub struct Demuxer {
    pub metadata: Option<Metadata>,
    pub core: Rc<RefCell<Core>>
}

impl Demuxer {
    pub fn new(core: Rc<RefCell<Core>>) -> Self {
        Self {
            metadata: None,
            core,
        }
    }
}