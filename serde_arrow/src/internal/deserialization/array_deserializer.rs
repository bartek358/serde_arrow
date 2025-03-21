use half::f16;
use marrow::view::{BytesView, BytesViewView, View};
use serde::{
    de::{Deserialize, DeserializeSeed, VariantAccess, Visitor},
    Deserializer,
};

use crate::internal::{
    error::{fail, Context, Error, Result},
    schema::Strategy,
};

use super::{
    binary_deserializer::BinaryDeserializer,
    bool_deserializer::BoolDeserializer,
    date_deserializer::DateDeserializer,
    decimal_deserializer::DecimalDeserializer,
    dictionary_deserializer::DictionaryDeserializer,
    duration_deserializer::DurationDeserializer,
    enum_deserializer::EnumDeserializer,
    fixed_size_binary_deserializer::FixedSizeBinaryDeserializer,
    fixed_size_list_deserializer::FixedSizeListDeserializer,
    float_deserializer::FloatDeserializer,
    integer_deserializer::IntegerDeserializer,
    list_deserializer::ListDeserializer,
    map_deserializer::MapDeserializer,
    null_deserializer::NullDeserializer,
    random_access_deserializer::{PositionedDeserializer, RandomAccessDeserializer},
    string_deserializer::StringDeserializer,
    struct_deserializer::StructDeserializer,
    time_deserializer::TimeDeserializer,
    timestamp_deserializer::TimestampDeserializer,
};

pub enum ArrayDeserializer<'a> {
    Null(NullDeserializer),
    Bool(BoolDeserializer<'a>),
    U8(IntegerDeserializer<'a, u8>),
    U16(IntegerDeserializer<'a, u16>),
    U32(IntegerDeserializer<'a, u32>),
    U64(IntegerDeserializer<'a, u64>),
    I8(IntegerDeserializer<'a, i8>),
    I16(IntegerDeserializer<'a, i16>),
    I32(IntegerDeserializer<'a, i32>),
    I64(IntegerDeserializer<'a, i64>),
    F16(FloatDeserializer<'a, f16>),
    F32(FloatDeserializer<'a, f32>),
    F64(FloatDeserializer<'a, f64>),
    Decimal128(DecimalDeserializer<'a>),
    Duration(DurationDeserializer<'a>),
    Date32(DateDeserializer<'a, i32>),
    Date64(DateDeserializer<'a, i64>),
    Time32(TimeDeserializer<'a, i32>),
    Time64(TimeDeserializer<'a, i64>),
    Timestamp(TimestampDeserializer<'a>),
    Utf8(StringDeserializer<BytesView<'a, i32>>),
    LargeUtf8(StringDeserializer<BytesView<'a, i64>>),
    Utf8View(StringDeserializer<BytesViewView<'a>>),
    DictionaryU8I32(DictionaryDeserializer<'a, u8, i32>),
    DictionaryU16I32(DictionaryDeserializer<'a, u16, i32>),
    DictionaryU32I32(DictionaryDeserializer<'a, u32, i32>),
    DictionaryU64I32(DictionaryDeserializer<'a, u64, i32>),
    DictionaryI8I32(DictionaryDeserializer<'a, i8, i32>),
    DictionaryI16I32(DictionaryDeserializer<'a, i16, i32>),
    DictionaryI32I32(DictionaryDeserializer<'a, i32, i32>),
    DictionaryI64I32(DictionaryDeserializer<'a, i64, i32>),
    DictionaryU8I64(DictionaryDeserializer<'a, u8, i64>),
    DictionaryU16I64(DictionaryDeserializer<'a, u16, i64>),
    DictionaryU32I64(DictionaryDeserializer<'a, u32, i64>),
    DictionaryU64I64(DictionaryDeserializer<'a, u64, i64>),
    DictionaryI8I64(DictionaryDeserializer<'a, i8, i64>),
    DictionaryI16I64(DictionaryDeserializer<'a, i16, i64>),
    DictionaryI32I64(DictionaryDeserializer<'a, i32, i64>),
    DictionaryI64I64(DictionaryDeserializer<'a, i64, i64>),
    Struct(StructDeserializer<'a>),
    List(ListDeserializer<'a, i32>),
    LargeList(ListDeserializer<'a, i64>),
    FixedSizeList(FixedSizeListDeserializer<'a>),
    Binary(BinaryDeserializer<BytesView<'a, i32>>),
    LargeBinary(BinaryDeserializer<BytesView<'a, i64>>),
    BinaryView(BinaryDeserializer<BytesViewView<'a>>),
    FixedSizeBinary(FixedSizeBinaryDeserializer<'a>),
    Map(MapDeserializer<'a>),
    Enum(EnumDeserializer<'a>),
}

impl<'a> ArrayDeserializer<'a> {
    // TODO: decide whether to keep strategy parameter
    pub fn new(path: String, _strategy: Option<&Strategy>, array: View<'a>) -> Result<Self> {
        use {ArrayDeserializer as D, View as V};
        match array {
            View::Null(_) => Ok(Self::Null(NullDeserializer::new(path))),
            V::Boolean(view) => Ok(D::Bool(BoolDeserializer::new(path, view))),
            V::Int8(view) => Ok(D::I8(IntegerDeserializer::new(path, view))),
            V::Int16(view) => Ok(D::I16(IntegerDeserializer::new(path, view))),
            V::Int32(view) => Ok(D::I32(IntegerDeserializer::new(path, view))),
            V::Int64(view) => Ok(D::I64(IntegerDeserializer::new(path, view))),
            V::UInt8(view) => Ok(D::U8(IntegerDeserializer::new(path, view))),
            V::UInt16(view) => Ok(D::U16(IntegerDeserializer::new(path, view))),
            V::UInt32(view) => Ok(D::U32(IntegerDeserializer::new(path, view))),
            V::UInt64(view) => Ok(D::U64(IntegerDeserializer::new(path, view))),
            V::Float16(view) => Ok(D::F16(FloatDeserializer::new(path, view))),
            V::Float32(view) => Ok(D::F32(FloatDeserializer::new(path, view))),
            V::Float64(view) => Ok(D::F64(FloatDeserializer::new(path, view))),
            V::Decimal128(view) => Ok(D::Decimal128(DecimalDeserializer::new(path, view))),
            View::Date32(view) => Ok(Self::Date32(DateDeserializer::new(path, view))),
            View::Date64(view) => Ok(Self::Date64(DateDeserializer::new(path, view))),
            V::Time32(view) => Ok(D::Time32(TimeDeserializer::new(path, view))),
            V::Time64(view) => Ok(D::Time64(TimeDeserializer::new(path, view))),
            V::Timestamp(view) => Ok(Self::Timestamp(TimestampDeserializer::new(path, view)?)),
            V::Duration(view) => Ok(D::Duration(DurationDeserializer::new(path, view))),
            V::Utf8(view) => Ok(D::Utf8(StringDeserializer::new(path, view))),
            V::LargeUtf8(view) => Ok(D::LargeUtf8(StringDeserializer::new(path, view))),
            V::Utf8View(view) => Ok(D::Utf8View(StringDeserializer::new(path, view))),
            V::Binary(view) => Ok(D::Binary(BinaryDeserializer::new(path, view))),
            V::LargeBinary(view) => Ok(D::LargeBinary(BinaryDeserializer::new(path, view))),
            V::BinaryView(view) => Ok(D::BinaryView(BinaryDeserializer::new(path, view))),
            V::FixedSizeBinary(view) => Ok(D::FixedSizeBinary(FixedSizeBinaryDeserializer::new(
                path, view,
            )?)),
            V::List(view) => Ok(D::List(ListDeserializer::new(path, view)?)),
            V::LargeList(view) => Ok(D::LargeList(ListDeserializer::new(path, view)?)),
            V::FixedSizeList(view) => Ok(D::FixedSizeList(FixedSizeListDeserializer::new(
                path, view,
            )?)),
            V::Struct(view) => Ok(D::Struct(StructDeserializer::new(path, view)?)),
            V::Map(view) => Ok(D::Map(MapDeserializer::new(path, view)?)),
            View::Union(view) => Ok(Self::Enum(EnumDeserializer::new(path, view)?)),
            V::Dictionary(view) => match (*view.keys, *view.values) {
                (V::Int8(keys), V::Utf8(values)) => Ok(D::DictionaryI8I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int16(keys), V::Utf8(values)) => Ok(D::DictionaryI16I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int32(keys), V::Utf8(values)) => Ok(D::DictionaryI32I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int64(keys), V::Utf8(values)) => Ok(D::DictionaryI64I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt8(keys), V::Utf8(values)) => Ok(Self::DictionaryU8I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt16(keys), V::Utf8(values)) => Ok(D::DictionaryU16I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt32(keys), V::Utf8(values)) => Ok(D::DictionaryU32I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt64(keys), V::Utf8(values)) => Ok(D::DictionaryU64I32(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int8(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI8I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int16(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI16I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int32(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI32I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::Int64(keys), V::LargeUtf8(values)) => Ok(D::DictionaryI64I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt8(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU8I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt16(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU16I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt32(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU32I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                (V::UInt64(keys), V::LargeUtf8(values)) => Ok(D::DictionaryU64I64(
                    DictionaryDeserializer::new(path, keys, values)?,
                )),
                _ => fail!("Unsupported dictionary array type"),
            },
            _ => fail!("Unknown view"),
        }
    }
}

macro_rules! dispatch {
    ($obj:expr, $wrapper:ident($name:ident) => $expr:expr) => {
        match $obj {
            $wrapper::Null($name) => $expr,
            $wrapper::Bool($name) => $expr,
            $wrapper::U8($name) => $expr,
            $wrapper::U16($name) => $expr,
            $wrapper::U32($name) => $expr,
            $wrapper::U64($name) => $expr,
            $wrapper::I8($name) => $expr,
            $wrapper::I16($name) => $expr,
            $wrapper::I32($name) => $expr,
            $wrapper::I64($name) => $expr,
            $wrapper::F16($name) => $expr,
            $wrapper::F32($name) => $expr,
            $wrapper::F64($name) => $expr,
            $wrapper::Decimal128($name) => $expr,
            $wrapper::Duration($name) => $expr,
            $wrapper::Date32($name) => $expr,
            $wrapper::Date64($name) => $expr,
            $wrapper::Time32($name) => $expr,
            $wrapper::Time64($name) => $expr,
            $wrapper::Timestamp($name) => $expr,
            $wrapper::Utf8($name) => $expr,
            $wrapper::LargeUtf8($name) => $expr,
            $wrapper::Utf8View($name) => $expr,
            $wrapper::Struct($name) => $expr,
            $wrapper::List($name) => $expr,
            $wrapper::FixedSizeList($name) => $expr,
            $wrapper::LargeList($name) => $expr,
            $wrapper::Binary($name) => $expr,
            $wrapper::LargeBinary($name) => $expr,
            $wrapper::BinaryView($name) => $expr,
            $wrapper::FixedSizeBinary($name) => $expr,
            $wrapper::Map($name) => $expr,
            $wrapper::Enum($name) => $expr,
            $wrapper::DictionaryU8I32($name) => $expr,
            $wrapper::DictionaryU16I32($name) => $expr,
            $wrapper::DictionaryU32I32($name) => $expr,
            $wrapper::DictionaryU64I32($name) => $expr,
            $wrapper::DictionaryI8I32($name) => $expr,
            $wrapper::DictionaryI16I32($name) => $expr,
            $wrapper::DictionaryI32I32($name) => $expr,
            $wrapper::DictionaryI64I32($name) => $expr,
            $wrapper::DictionaryU8I64($name) => $expr,
            $wrapper::DictionaryU16I64($name) => $expr,
            $wrapper::DictionaryU32I64($name) => $expr,
            $wrapper::DictionaryU64I64($name) => $expr,
            $wrapper::DictionaryI8I64($name) => $expr,
            $wrapper::DictionaryI16I64($name) => $expr,
            $wrapper::DictionaryI32I64($name) => $expr,
            $wrapper::DictionaryI64I64($name) => $expr,
        }
    };
}

impl Context for ArrayDeserializer<'_> {
    fn annotate(&self, annotations: &mut std::collections::BTreeMap<String, String>) {
        dispatch!(self, ArrayDeserializer(deser) => deser.annotate(annotations))
    }
}

impl<'de> VariantAccess<'de> for PositionedDeserializer<'_, ArrayDeserializer<'de>> {
    type Error = Error;

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> std::result::Result<T::Value, Self::Error> {
        seed.deserialize(self)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_struct("UNUSED_ENUM_STRUCT_NAME", fields, visitor)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    fn unit_variant(self) -> std::result::Result<(), Self::Error> {
        <()>::deserialize(self)
    }
}

impl<'de> RandomAccessDeserializer<'de> for ArrayDeserializer<'de> {
    fn is_some(&self, idx: usize) -> Result<bool> {
        dispatch!(self, Self(this) => this.is_some(idx))
    }

    fn deserialize_any_some<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_any_some(visitor, idx))
    }

    fn deserialize_bool<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_bool(visitor, idx))
    }

    fn deserialize_i8<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_i8(visitor, idx))
    }

    fn deserialize_i16<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_i16(visitor, idx))
    }

    fn deserialize_i32<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_i32(visitor, idx))
    }

    fn deserialize_i64<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_i64(visitor, idx))
    }

    fn deserialize_u8<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_u8(visitor, idx))
    }

    fn deserialize_u16<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_u16(visitor, idx))
    }

    fn deserialize_u32<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_u32(visitor, idx))
    }

    fn deserialize_u64<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_u64(visitor, idx))
    }

    fn deserialize_f32<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_f32(visitor, idx))
    }

    fn deserialize_f64<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_f64(visitor, idx))
    }

    fn deserialize_char<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_char(visitor, idx))
    }

    fn deserialize_str<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_str(visitor, idx))
    }

    fn deserialize_string<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_string(visitor, idx))
    }

    fn deserialize_map<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_map(visitor, idx))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        &self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
        idx: usize,
    ) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_struct(name, fields, visitor, idx))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_byte_buf(visitor, idx))
    }

    fn deserialize_bytes<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_bytes(visitor, idx))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        &self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
        idx: usize,
    ) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_enum(name, variants, visitor, idx))
    }

    fn deserialize_identifier<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_identifier(visitor, idx))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        &self,
        name: &'static str,
        visitor: V,
        idx: usize,
    ) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_newtype_struct(name, visitor, idx))
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        &self,
        len: usize,
        visitor: V,
        idx: usize,
    ) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_tuple(len, visitor, idx))
    }

    fn deserialize_seq<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_seq(visitor, idx))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        &self,
        name: &'static str,
        len: usize,
        visitor: V,
        idx: usize,
    ) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_tuple_struct(name, len, visitor, idx))
    }

    fn deserialize_unit<V: Visitor<'de>>(&self, visitor: V, idx: usize) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_unit(visitor, idx))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        &self,
        name: &'static str,
        visitor: V,
        idx: usize,
    ) -> Result<V::Value> {
        dispatch!(self, Self(this) => this.deserialize_unit_struct(name, visitor, idx))
    }
}
