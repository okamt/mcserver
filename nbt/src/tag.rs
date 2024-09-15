use std::fmt::{Debug, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tag {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl Tag {
    #[inline]
    pub fn to_u8(self) -> u8 {
        <Self as Into<u8>>::into(self)
    }

    #[inline]
    pub const fn from_u8(value: u8) -> Self {
        if value > 12 {
            panic!("invalid tag value");
        }

        // SAFETY: Tag covers all u8s in 0..=12, and is a "fieldless" enum with #[repr(u8)], so transmuting to u8 is safe.
        // https://doc.rust-lang.org/nomicon/other-reprs.html#repru-repri
        unsafe { std::mem::transmute(value) }
    }

    #[inline]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        std::mem::transmute(value)
    }
}

impl TryFrom<u8> for Tag {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 13 {
            // SAFETY: Tag covers all u8s in 0..=12, and is a "fieldless" enum with #[repr(u8)], so transmuting to u8 is safe.
            // https://doc.rust-lang.org/nomicon/other-reprs.html#repru-repri
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(value)
        }
    }
}

impl Into<u8> for Tag {
    #[inline]
    fn into(self) -> u8 {
        // SAFETY: Tag is a "fieldless" enum with #[repr(u8)], so transmuting to u8 is safe.
        // https://doc.rust-lang.org/nomicon/other-reprs.html#repru-repri
        unsafe { std::mem::transmute(self) }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Tag as Debug>::fmt(self, f)
    }
}
