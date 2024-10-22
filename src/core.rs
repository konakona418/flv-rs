use std::cell::RefCell;
use std::collections::VecDeque;
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
        core.borrow_mut().flv_decoder = Some(Rc::new(RefCell::from(Decoder::new(VecDeque::new(), core.clone()))));
        core.borrow_mut().flv_demuxer = Some(Rc::new(RefCell::from(Demuxer::new(core.clone()))));
        core
    }

    pub fn push_data(&self, data: &mut VecDeque<u8>) {
        self.flv_decoder
            .clone()
            .unwrap()
            .borrow_mut()
            .push_data(data)
    }

    pub fn start_decoding(&self) {
        let header = self.flv_decoder.clone().unwrap().borrow_mut().decode_header().unwrap();
        dbg!(header);
        self.flv_decoder.clone().unwrap().borrow_mut().decode_body().unwrap();
        // todo: when the video stream is chunked, it's necessary to 'wait' for the next chunk than simply break the decoder loop.
    }
}

