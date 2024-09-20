//! Marker traits and implementations.

use crate::{
    NbtByteArray, NbtCompound, NbtIntArray, NbtList, NbtLongArray, NbtMapRef, NbtRef, NbtString,
    NbtValue,
};

mod private {
    pub trait Sealed {}
}

use private::Sealed;

impl Sealed for NbtValue {}
impl Sealed for NbtByteArray {}
impl Sealed for NbtString {}
impl Sealed for NbtList {}
impl Sealed for NbtCompound {}
impl Sealed for NbtIntArray {}
impl Sealed for NbtLongArray {}

impl Sealed for i8 {}
impl Sealed for i16 {}
impl Sealed for i32 {}
impl Sealed for i64 {}
impl Sealed for f32 {}
impl Sealed for f64 {}

/// An [`NbtValue`] or an [`NbtRef`].
pub trait NbtValueOrRef: Copy + Sealed + 'static {}

impl NbtValueOrRef for NbtValue {}
impl<T> NbtValueOrRef for T where T: NbtRef {}

/// Values that are useful to the end user.
pub trait NbtValueOutput: Copy + Sealed + 'static {}

impl NbtValueOutput for i8 {}
impl NbtValueOutput for i16 {}
impl NbtValueOutput for i32 {}
impl NbtValueOutput for i64 {}
impl NbtValueOutput for f32 {}
impl NbtValueOutput for f64 {}
impl NbtValueOutput for NbtValue {}

/// An [`NbtMapRef`] that returns [`NbtValue`].
pub trait NbtContainer: for<'nbt> NbtMapRef<Value<'nbt> = NbtValue> {}

impl NbtContainer for NbtList {}
impl NbtContainer for NbtCompound {}
