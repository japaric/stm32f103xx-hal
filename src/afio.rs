use stm32f103xx::{afio, AFIO};

use rcc::APB2;

pub trait AfioExt {
    fn constrain(self, apb2: &mut APB2) -> Parts;
}

impl AfioExt for AFIO {
    fn constrain(self, apb2: &mut APB2) -> Parts {
        apb2.enr().modify(|_, w| w.afioen().enabled());
        apb2.rstr().modify(|_, w| w.afiorst().set_bit());
        apb2.rstr().modify(|_, w| w.afiorst().clear_bit());

        Parts {
            mapr: MAPR { _0: () },
        }
    }
}

pub struct Parts {
    pub mapr: MAPR,
}

pub struct MAPR {
    _0: (),
}

impl MAPR {
    pub fn mapr(&mut self) -> &afio::MAPR {
        unsafe { &(*AFIO::ptr()).mapr }
    }
}
