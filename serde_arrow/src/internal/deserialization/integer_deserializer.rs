use marrow::view::PrimitiveView;
use serde::de::Visitor;

use crate::internal::{
    error::{set_default, try_, Context, ContextSupport, Result},
    utils::{array_view_ext::ViewAccess, NamedType},
};

use super::random_access_deserializer::RandomAccessDeserializer;

pub trait Integer: Sized + Copy {
    fn deserialize_any_at<'de, S: RandomAccessDeserializer<'de>, V: Visitor<'de>>(
        deser: &S,
        visitor: V,
        idx: usize,
    ) -> Result<V::Value>;

    fn into_bool(self) -> Result<bool>;

    fn into_i8(self) -> Result<i8>;
    fn into_i16(self) -> Result<i16>;
    fn into_i32(self) -> Result<i32>;
    fn into_i64(self) -> Result<i64>;

    fn into_u8(self) -> Result<u8>;
    fn into_u16(self) -> Result<u16>;
    fn into_u32(self) -> Result<u32>;
    fn into_u64(self) -> Result<u64>;
}

pub struct IntegerDeserializer<'a, T: Integer> {
    path: String,
    view: PrimitiveView<'a, T>,
}

impl<'a, T: Integer> IntegerDeserializer<'a, T> {
    pub fn new(path: String, view: PrimitiveView<'a, T>) -> Self {
        Self { path, view }
    }
}

impl<T: NamedType + Integer> Context for IntegerDeserializer<'_, T> {
    fn annotate(&self, annotations: &mut std::collections::BTreeMap<String, String>) {
        set_default(annotations, "field", &self.path);
        set_default(
            annotations,
            "data_type",
            match T::NAME {
                "i8" => "Int8",
                "i16" => "Int16",
                "i32" => "Int32",
                "i64" => "Int64",
                "u8" => "UInt8",
                "u16" => "UInt16",
                "u32" => "UInt32",
                "u64" => "UInt64",
                _ => "<unknown>",
            },
        );
    }
}

impl<'de, T: NamedType + Integer> RandomAccessDeserializer<'de> for IntegerDeserializer<'de, T> {
    fn is_some(&self, idx: usize) -> Result<bool> {
        self.view.is_some(idx)
    }

    fn deserialize_any_some<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        T::deserialize_any_at(self, visitor, idx)
    }

    fn deserialize_bool<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_bool(self.view.get_required(idx)?.into_bool()?)).ctx(self)
    }

    fn deserialize_char<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_char(self.view.get_required(idx)?.into_u32()?.try_into()?)).ctx(self)
    }

    fn deserialize_u8<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_u8(self.view.get_required(idx)?.into_u8()?)).ctx(self)
    }

    fn deserialize_u16<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_u16(self.view.get_required(idx)?.into_u16()?)).ctx(self)
    }

    fn deserialize_u32<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_u32(self.view.get_required(idx)?.into_u32()?)).ctx(self)
    }

    fn deserialize_u64<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_u64(self.view.get_required(idx)?.into_u64()?)).ctx(self)
    }

    fn deserialize_i8<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_i8(self.view.get_required(idx)?.into_i8()?)).ctx(self)
    }

    fn deserialize_i16<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_i16(self.view.get_required(idx)?.into_i16()?)).ctx(self)
    }

    fn deserialize_i32<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_i32(self.view.get_required(idx)?.into_i32()?)).ctx(self)
    }

    fn deserialize_i64<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        try_(|| visitor.visit_i64(self.view.get_required(idx)?.into_i64()?)).ctx(self)
    }
}
