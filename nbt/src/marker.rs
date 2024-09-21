//! Marker traits and implementations.

use crate::{
    NbtByteArray, NbtCompound, NbtIntArray, NbtList, NbtLongArray, NbtMapRef, NbtNodeRef, NbtRef,
    NbtString, NbtValue,
};
use core::fmt::Debug;

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

impl<Value> Sealed for NbtNodeRef<Value> where Value: NbtValueRepr {}

/// Anything that can represent an NBT value (an [`NbtValue`], an [`NbtRef`] or an [`NbtNodeRef`]).
pub trait NbtValueRepr: Copy + Sized + Debug + Sealed + 'static {}

impl NbtValueRepr for i8 {}
impl NbtValueRepr for i16 {}
impl NbtValueRepr for i32 {}
impl NbtValueRepr for i64 {}
impl NbtValueRepr for f32 {}
impl NbtValueRepr for f64 {}
impl NbtValueRepr for NbtValue {}
impl<T> NbtValueRepr for T where T: NbtRef {}
impl<Value> NbtValueRepr for NbtNodeRef<Value> where Value: NbtValueRepr {}

/// An [`NbtMapRef`] that returns [`NbtValue`].
pub trait NbtContainer: NbtMapRef<Value = NbtValue> {}

impl NbtContainer for NbtList {}
impl NbtContainer for NbtCompound {}

/// [`NbtByteArray`], [`NbtIntArray`] or [`NbtLongArray`]
pub trait NbtArray: NbtMapRef {}

impl NbtArray for NbtByteArray {}
impl NbtArray for NbtIntArray {}
impl NbtArray for NbtLongArray {}
