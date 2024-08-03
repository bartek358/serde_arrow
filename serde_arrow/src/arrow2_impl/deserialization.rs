use crate::internal::{
    deserialization::{
        array_deserializer::ArrayDeserializer, dictionary_deserializer::DictionaryDeserializer,
        integer_deserializer::Integer, utils::BitBuffer,
    },
    error::{fail, Result},
    schema::{GenericDataType, GenericField},
    utils::Offset,
};

use crate::_impl::arrow2::{
    array::{Array, DictionaryArray, DictionaryKey, Utf8Array},
    types::Offset as ArrowOffset,
};

pub fn build_array_deserializer<'a>(
    field: &GenericField,
    array: &'a dyn Array,
) -> Result<ArrayDeserializer<'a>> {
    use GenericDataType as T;
    match &field.data_type {
        T::Dictionary => build_dictionary_deserializer(field, array),
        _ => ArrayDeserializer::new(field.strategy.as_ref(), array.try_into()?),
    }
}

pub fn build_dictionary_deserializer<'a>(
    field: &GenericField,
    array: &'a dyn Array,
) -> Result<ArrayDeserializer<'a>> {
    use GenericDataType as T;

    let Some(key_field) = field.children.first() else {
        fail!("Missing key field");
    };
    let Some(value_field) = field.children.get(1) else {
        fail!("Missing key field");
    };

    return match (&key_field.data_type, &value_field.data_type) {
        (T::U8, T::Utf8) => typed::<u8, i32>(field, array),
        (T::U16, T::Utf8) => typed::<u16, i32>(field, array),
        (T::U32, T::Utf8) => typed::<u32, i32>(field, array),
        (T::U64, T::Utf8) => typed::<u64, i32>(field, array),
        (T::I8, T::Utf8) => typed::<i8, i32>(field, array),
        (T::I16, T::Utf8) => typed::<i16, i32>(field, array),
        (T::I32, T::Utf8) => typed::<i32, i32>(field, array),
        (T::I64, T::Utf8) => typed::<i64, i32>(field, array),
        (T::U8, T::LargeUtf8) => typed::<u8, i64>(field, array),
        (T::U16, T::LargeUtf8) => typed::<u16, i64>(field, array),
        (T::U32, T::LargeUtf8) => typed::<u32, i64>(field, array),
        (T::U64, T::LargeUtf8) => typed::<u64, i64>(field, array),
        (T::I8, T::LargeUtf8) => typed::<i8, i64>(field, array),
        (T::I16, T::LargeUtf8) => typed::<i16, i64>(field, array),
        (T::I32, T::LargeUtf8) => typed::<i32, i64>(field, array),
        (T::I64, T::LargeUtf8) => typed::<i64, i64>(field, array),
        _ => fail!("invalid dicitonary key / value data type"),
    };

    pub fn typed<'a, K, V>(
        _field: &GenericField,
        array: &'a dyn Array,
    ) -> Result<ArrayDeserializer<'a>>
    where
        K: DictionaryKey + Integer,
        V: Offset + ArrowOffset,
        DictionaryDeserializer<'a, K, V>: Into<ArrayDeserializer<'a>>,
    {
        let Some(array) = array.as_any().downcast_ref::<DictionaryArray<K>>() else {
            fail!("cannot convert array into dictionary array");
        };
        let Some(values) = array.values().as_any().downcast_ref::<Utf8Array<V>>() else {
            fail!("invalid values");
        };

        let keys_buffer = array.keys().values();
        let keys_validity = get_validity(array);

        let values_data = values.values().as_slice();
        let values_offsets = values.offsets().as_slice();

        Ok(
            DictionaryDeserializer::new(keys_buffer, keys_validity, values_data, values_offsets)
                .into(),
        )
    }
}

pub fn build_struct_fields<'a>(
    fields: &[GenericField],
    arrays: &[&'a dyn Array],
) -> Result<(Vec<(String, ArrayDeserializer<'a>)>, usize)> {
    if fields.len() != arrays.len() {
        fail!(
            "different number of fields ({}) and arrays ({})",
            fields.len(),
            arrays.len()
        );
    }
    let len = arrays.first().map(|array| array.len()).unwrap_or_default();

    let mut deserializers = Vec::new();
    for (field, &array) in std::iter::zip(fields, arrays) {
        if array.len() != len {
            fail!("arrays of different lengths are not supported");
        }

        deserializers.push((field.name.clone(), build_array_deserializer(field, array)?));
    }

    Ok((deserializers, len))
}

fn get_validity(arr: &dyn Array) -> Option<BitBuffer<'_>> {
    let validity = arr.validity()?;
    let (data, offset, number_of_bits) = validity.as_slice();
    Some(BitBuffer {
        data,
        offset,
        number_of_bits,
    })
}
