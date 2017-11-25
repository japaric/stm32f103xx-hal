use stm32f103xx::{afio, AFIO};

pub trait AfioExt {
    fn split(self) -> Parts;
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
    fn split(self) -> Parts {
        // TODO enable AFIOEN in RCC

        Parts {
            mapr: MAPR { _0: () },
        }
    }
}
