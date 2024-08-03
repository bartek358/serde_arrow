use crate::{
    _impl::arrow2::{
        array::{
            Array as A2Array, BinaryArray, BooleanArray, NullArray, PrimitiveArray, Utf8Array,
        },
        bitmap::Bitmap,
        buffer::Buffer,
        datatypes::DataType,
        types::{f16, NativeType, Offset},
    },
    internal::{
        arrow::Array,
        error::{fail, Error, Result},
    },
};

impl TryFrom<Array> for Box<dyn A2Array> {
    type Error = Error;

    fn try_from(value: Array) -> Result<Self> {
        use {Array as A, DataType as T};
        match value {
            A::Null(arr) => Ok(Box::new(NullArray::new(T::Null, arr.len))),
            A::Boolean(arr) => Ok(Box::new(BooleanArray::try_new(
                T::Boolean,
                Bitmap::from_u8_vec(arr.values, arr.len),
                arr.validity.map(|v| Bitmap::from_u8_vec(v, arr.len)),
            )?)),
            A::Int8(arr) => build_primitive_array(T::Int8, arr.values, arr.validity),
            A::Int16(arr) => build_primitive_array(T::Int16, arr.values, arr.validity),
            A::Int32(arr) => build_primitive_array(T::Int32, arr.values, arr.validity),
            A::Int64(arr) => build_primitive_array(T::Int64, arr.values, arr.validity),
            A::UInt8(arr) => build_primitive_array(T::UInt8, arr.values, arr.validity),
            A::UInt16(arr) => build_primitive_array(T::UInt16, arr.values, arr.validity),
            A::UInt32(arr) => build_primitive_array(T::UInt32, arr.values, arr.validity),
            A::UInt64(arr) => build_primitive_array(T::UInt64, arr.values, arr.validity),
            A::Float16(arr) => build_primitive_array(
                T::Float16,
                arr.values
                    .into_iter()
                    .map(|v| f16::from_bits(v.to_bits()))
                    .collect(),
                arr.validity,
            ),
            A::Float32(arr) => build_primitive_array(T::Float32, arr.values, arr.validity),
            A::Float64(arr) => build_primitive_array(T::Float64, arr.values, arr.validity),
            A::Date32(arr) => build_primitive_array(T::Date32, arr.values, arr.validity),
            A::Date64(arr) => build_primitive_array(T::Date64, arr.values, arr.validity),
            A::Duration(arr) => {
                build_primitive_array(T::Duration(arr.unit.into()), arr.values, arr.validity)
            }
            A::Time32(arr) => {
                build_primitive_array(T::Time32(arr.unit.into()), arr.values, arr.validity)
            }
            A::Time64(arr) => {
                build_primitive_array(T::Time64(arr.unit.into()), arr.values, arr.validity)
            }
            A::Timestamp(arr) => build_primitive_array(
                T::Timestamp(arr.unit.into(), arr.timezone),
                arr.values,
                arr.validity,
            ),
            A::Decimal128(arr) => build_primitive_array(
                T::Decimal(arr.precision as usize, usize::try_from(arr.scale)?),
                arr.values,
                arr.validity,
            ),
            A::Utf8(arr) => build_utf8_array(T::Utf8, arr.offsets, arr.data, arr.validity),
            A::LargeUtf8(arr) => {
                build_utf8_array(T::LargeUtf8, arr.offsets, arr.data, arr.validity)
            }
            A::Binary(arr) => build_binary_array(T::Binary, arr.offsets, arr.data, arr.validity),
            A::LargeBinary(arr) => {
                build_binary_array(T::LargeBinary, arr.offsets, arr.data, arr.validity)
            }
            _ => fail!("cannot convert array to arrow2 array"),
        }
    }
}

fn build_primitive_array<T: NativeType>(
    data_type: DataType,
    buffer: Vec<T>,
    validity: Option<Vec<u8>>,
) -> Result<Box<dyn A2Array>> {
    let validity = validity.map(|v| Bitmap::from_u8_vec(v, buffer.len()));
    let buffer = Buffer::from(buffer);
    Ok(Box::new(PrimitiveArray::try_new(
        data_type, buffer, validity,
    )?))
}

fn build_utf8_array<O: Offset>(
    data_type: DataType,
    offsets: Vec<O>,
    data: Vec<u8>,
    validity: Option<Vec<u8>>,
) -> Result<Box<dyn A2Array>> {
    let validity = validity.map(|v| Bitmap::from_u8_vec(v, offsets.len().saturating_sub(1)));
    Ok(Box::new(Utf8Array::new(
        data_type,
        offsets.try_into()?,
        Buffer::from(data),
        validity,
    )))
}

fn build_binary_array<O: Offset>(
    data_type: DataType,
    offsets: Vec<O>,
    data: Vec<u8>,
    validity: Option<Vec<u8>>,
) -> Result<Box<dyn A2Array>> {
    let validity = validity.map(|v| Bitmap::from_u8_vec(v, offsets.len().saturating_sub(1)));
    Ok(Box::new(BinaryArray::new(
        data_type,
        offsets.try_into()?,
        Buffer::from(data),
        validity,
    )))
}
