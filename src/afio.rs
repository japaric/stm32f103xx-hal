use stm32f103xx::{afio, AFIO};

use rcc::ENR;

pub trait AfioExt {
    fn split(self, enr: &mut ENR) -> Parts;
}

pub struct Parts {
    pub mapr: MAPR,
}

pub struct MAPR {
    _0: (),
}

impl MAPR {
    pub(crate) fn mapr(&mut self) -> &afio::MAPR {
        unsafe { &(*AFIO::ptr()).mapr }
    }
}

impl AfioExt for AFIO {
    fn split(self, enr: &mut ENR) -> Parts {
        enr.apb2().modify(|_, w| w.afioen().enabled());

        Parts {
            mapr: MAPR { _0: () },
        }
    }
}
