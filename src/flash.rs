use stm32f103xx::{FLASH, flash};

pub trait FlashExt {
    fn split(self) -> Flash;
}

impl FlashExt for FLASH {
    fn split(self) -> Flash {
        Flash {
            acr: ACR { _0: () },
        }
    }
}

pub struct Flash {
    pub acr: ACR,
}

pub struct ACR {
    _0: (),
}

impl ACR {
    pub(crate) fn acr(&mut self) -> &flash::ACR {
        unsafe {
            &(*FLASH::ptr()).acr
        }
    }
}
