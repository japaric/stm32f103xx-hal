//! Units of time

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Bps(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Hertz(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct KiloHertz(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct MegaHertz(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Seconds(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct MilliSeconds(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct MicroSeconds(pub u32);

pub trait TimeExt {
    fn bps(self) -> Bps;
    fn hz(self) -> Hertz;
    fn khz(self) -> KiloHertz;
    fn mhz(self) -> MegaHertz;
    fn ms(self) -> MilliSeconds;
    fn s(self) -> Seconds;
    fn us(self) -> MicroSeconds;
}

impl TimeExt for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn mhz(self) -> MegaHertz {
        MegaHertz(self)
    }

    fn ms(self) -> MilliSeconds {
        MilliSeconds(self)
    }

    fn s(self) -> Seconds {
        Seconds(self)
    }

    fn us(self) -> MicroSeconds {
        MicroSeconds(self)
    }
}

impl Into<Hertz> for KiloHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000)
    }
}

impl Into<Hertz> for MegaHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000_000)
    }
}

impl Into<KiloHertz> for MegaHertz {
    fn into(self) -> KiloHertz {
        KiloHertz(self.0 * 1_000)
    }
}

impl Into<MicroSeconds> for MilliSeconds {
    fn into(self) -> MicroSeconds {
        MicroSeconds(self.0 * 1_000)
    }
}

impl Into<MicroSeconds> for Seconds {
    fn into(self) -> MicroSeconds {
        MicroSeconds(self.0 * 1_000_000)
    }
}

impl Into<MilliSeconds> for Seconds {
    fn into(self) -> MilliSeconds {
        MilliSeconds(self.0 * 1_000)
    }
}
