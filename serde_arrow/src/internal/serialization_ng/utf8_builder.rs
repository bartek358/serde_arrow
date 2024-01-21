use crate::{
    internal::common::{MutableBitBuffer, MutableOffsetBuffer, Offset},
    Result,
};

use super::utils::{push_validity, push_validity_default, Mut, SimpleSerializer};

#[derive(Debug, Clone)]
pub struct Utf8Builder<O> {
    pub validity: Option<MutableBitBuffer>,
    pub offsets: MutableOffsetBuffer<O>,
    pub buffer: Vec<u8>,
}

impl<O: Offset> Utf8Builder<O> {
    pub fn new(is_nullable: bool) -> Self {
        Self {
            validity: is_nullable.then(MutableBitBuffer::default),
            offsets: MutableOffsetBuffer::default(),
            buffer: Vec::new(),
        }
    }
}

impl<O: Offset> SimpleSerializer for Utf8Builder<O> {
    fn name(&self) -> &str {
        "Utf8Builder"
    }

    fn serialize_default(&mut self) -> Result<()> {
        push_validity_default(&mut self.validity);
        self.offsets.push_current_items();
        Ok(())
    }

    fn serialize_none(&mut self) -> Result<()> {
        push_validity(&mut self.validity, false)?;
        self.offsets.push_current_items();
        Ok(())
    }

    fn serialize_some<V: serde::Serialize + ?Sized>(&mut self, value: &V) -> Result<()> {
        value.serialize(Mut(self))
    }

    fn serialize_str(&mut self, v: &str) -> Result<()> {
        push_validity(&mut self.validity, true)?;
        self.offsets.push(v.len())?;
        self.buffer.extend(v.as_bytes());

        Ok(())
    }
}
