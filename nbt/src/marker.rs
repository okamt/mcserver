//! Typestate markers.

pub trait IteratorMarker {}

pub struct CompoundMarker {}
pub struct ListMarker {}

impl IteratorMarker for CompoundMarker {}
impl IteratorMarker for ListMarker {}
