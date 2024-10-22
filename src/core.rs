use std::cell::RefCell;
use std::rc::Rc;
use crate::flv::decoder::Decoder;
use crate::flv::demuxer::Demuxer;

pub struct Core {
    flv_demuxer: Option<Rc<RefCell<Demuxer>>>,
    flv_decoder: Option<Rc<RefCell<Decoder>>>,
}

impl Core {
    pub fn new() -> Rc<RefCell<Self>> {
        let core = Core {
            flv_decoder: None,
            flv_demuxer: None,
        };
        let core = Rc::new(RefCell::new(core));
        core.borrow_mut().flv_decoder = Some(Rc::new(RefCell::from(Decoder::new(vec![], core.clone()))));
        core.borrow_mut().flv_demuxer = Some(Rc::new(RefCell::from(Demuxer::new(core.clone()))));
        core
    }
}

