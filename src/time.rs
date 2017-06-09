//! Units of time

macro_rules! map {
    ($Self:ident) => {
        impl $Self {
            /// Applies the function `f` to inner value
            pub fn map<F>(self, f: F) -> $Self
            where
                F: FnOnce(u32) -> u32
            {
                $Self(f(self.0))
            }
        }
    }
}

/// `Hz^-1`
#[derive(Clone, Copy, Debug)]
pub struct IHertz(pub u32);

impl IHertz {
    /// Invert this quantity
    pub fn invert(self) -> Hertz {
        Hertz(self.0)
    }
}

map!(IHertz);

/// `Hz`
#[derive(Clone, Copy, Debug)]
pub struct Hertz(pub u32);

impl Hertz {
    /// Invert this quantity
    pub fn invert(self) -> IHertz {
        IHertz(self.0)
    }
}

map!(Hertz);

/// `us`
#[derive(Clone, Copy, Debug)]
pub struct Microseconds(pub u32);

map!(Microseconds);

/// `ms`
#[derive(Clone, Copy, Debug)]
pub struct Milliseconds(pub u32);

map!(Milliseconds);

/// `s`
#[derive(Clone, Copy, Debug)]
pub struct Seconds(pub u32);

map!(Seconds);

/// `u32` extension trait
pub trait U32Ext {
    /// Wrap in `Hz`
    fn hz(self) -> Hertz;

    /// Wrap in `Milliseconds`
    fn ms(self) -> Milliseconds;

    /// Wrap in `Seconds`
    fn s(self) -> Seconds;

    /// Wrap in `Microseconds`
    fn us(self) -> Microseconds;
}

impl U32Ext for u32 {
    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn ms(self) -> Milliseconds {
        Milliseconds(self)
    }

    fn s(self) -> Seconds {
        Seconds(self)
    }

    fn us(self) -> Microseconds {
        Microseconds(self)
    }
}
