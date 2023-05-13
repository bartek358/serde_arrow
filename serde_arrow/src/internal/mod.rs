pub mod bytecode;
pub(crate) mod error;
pub(crate) mod event;
pub(crate) mod generic_sinks;
pub(crate) mod generic_sources;
pub(crate) mod schema;
pub(crate) mod sink;
pub(crate) mod source;

use std::sync::RwLock;

use serde::Serialize;

use self::{
    error::{fail, Result},
    generic_sinks::{
        DictionaryUtf8ArrayBuilder, ListArrayBuilder, MapArrayBuilder, NaiveDateTimeStrBuilder,
        PrimitiveBuilders, StructArrayBuilder, TupleStructBuilder, UnionArrayBuilder,
        UtcDateTimeStrBuilder,
    },
    schema::{GenericDataType, GenericField, Tracer, TracingOptions},
    sink::{
        serialize_into_sink, ArrayBuilder, DynamicArrayBuilder, EventSerializer, EventSink,
        StripOuterSequenceSink,
    },
};

pub static CONFIGURATION: RwLock<Configuration> = RwLock::new(Configuration {
    serialize_with_bytecode: false,
});

/// The crate settings can be configured by calling [configure]
#[derive(Default, Clone)]
pub struct Configuration {
    /// If `true`, use the exerperimental bytecode serializer
    ///
    pub serialize_with_bytecode: bool,
}

/// Change global configuration options
///
pub fn configure<F: FnOnce(&mut Configuration)>(f: F) {
    let mut guard = CONFIGURATION.write().unwrap();
    f(&mut guard)
}

pub fn serialize_into_fields<T>(items: &T, options: TracingOptions) -> Result<Vec<GenericField>>
where
    T: Serialize + ?Sized,
{
    let tracer = Tracer::new(String::from("$"), options);
    let mut tracer = StripOuterSequenceSink::new(tracer);
    serialize_into_sink(&mut tracer, items)?;
    let root = tracer.into_inner().to_field("root")?;

    match root.data_type {
        GenericDataType::Struct => {}
        GenericDataType::Null => fail!("No records found to determine schema"),
        dt => fail!("Unexpected root data type {dt:?}"),
    };

    Ok(root.children)
}

pub fn serialize_into_field<T>(
    items: &T,
    name: &str,
    options: TracingOptions,
) -> Result<GenericField>
where
    T: Serialize + ?Sized,
{
    let tracer = Tracer::new(String::from("$"), options);
    let tracer = StripOuterSequenceSink::new(tracer);
    let mut tracer = tracer;
    serialize_into_sink(&mut tracer, items)?;

    let field = tracer.into_inner().to_field(name)?;
    Ok(field)
}

pub fn serialize_into_arrays<T, Arrow>(
    fields: &[GenericField],
    items: &T,
) -> Result<Vec<Arrow::Output>>
where
    T: Serialize + ?Sized,
    Arrow: PrimitiveBuilders,
    NaiveDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UtcDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    TupleStructBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    StructArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UnionArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    DictionaryUtf8ArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    MapArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i32>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i64>: ArrayBuilder<Arrow::Output>,
{
    let builder = generic_sinks::build_struct_array_builder::<Arrow>(String::from("$"), fields)?;
    let mut builder = StripOuterSequenceSink::new(builder);

    serialize_into_sink(&mut builder, items)?;
    builder.into_inner().build_arrays()
}

pub fn serialize_into_array<T, Arrow>(field: &GenericField, items: &T) -> Result<Arrow::Output>
where
    T: Serialize + ?Sized,
    Arrow: PrimitiveBuilders,
    NaiveDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UtcDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    TupleStructBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    StructArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UnionArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    DictionaryUtf8ArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    MapArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i32>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i64>: ArrayBuilder<Arrow::Output>,
{
    let builder = generic_sinks::build_array_builder::<Arrow>(String::from("$"), field)?;
    let builder = StripOuterSequenceSink::new(builder);
    let mut builder = builder;

    serialize_into_sink(&mut builder, items).unwrap();
    builder.into_inner().build_array()
}

pub struct GenericArrayBuilder<Arrow: PrimitiveBuilders> {
    builder: DynamicArrayBuilder<Arrow::Output>,
    field: GenericField,
}

impl<Arrow> GenericArrayBuilder<Arrow>
where
    Arrow: PrimitiveBuilders,
    NaiveDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UtcDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    TupleStructBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    StructArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UnionArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    DictionaryUtf8ArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    MapArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i32>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i64>: ArrayBuilder<Arrow::Output>,
{
    pub fn new(field: GenericField) -> Result<Self> {
        Ok(Self {
            builder: generic_sinks::build_array_builder::<Arrow>(String::from("$"), &field)?,
            field,
        })
    }

    pub fn push<T: Serialize + ?Sized>(&mut self, item: &T) -> Result<()> {
        item.serialize(EventSerializer(&mut self.builder))?;
        Ok(())
    }

    pub fn extend<T: Serialize + ?Sized>(&mut self, items: &T) -> Result<()> {
        let mut builder = StripOuterSequenceSink::new(&mut self.builder);
        items.serialize(EventSerializer(&mut builder))?;
        Ok(())
    }

    pub fn build_array(&mut self) -> Result<Arrow::Output> {
        let mut builder =
            generic_sinks::build_array_builder::<Arrow>(String::from("$"), &self.field)?;
        std::mem::swap(&mut builder, &mut self.builder);

        builder.finish()?;
        builder.build_array()
    }
}

pub struct GenericArraysBuilder<Arrow: PrimitiveBuilders> {
    fields: Vec<GenericField>,
    builder: StructArrayBuilder<DynamicArrayBuilder<Arrow::Output>>,
}

impl<Arrow> GenericArraysBuilder<Arrow>
where
    Arrow: PrimitiveBuilders,
    NaiveDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UtcDateTimeStrBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    TupleStructBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    StructArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    UnionArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    DictionaryUtf8ArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    MapArrayBuilder<DynamicArrayBuilder<Arrow::Output>>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i32>: ArrayBuilder<Arrow::Output>,
    ListArrayBuilder<DynamicArrayBuilder<Arrow::Output>, i64>: ArrayBuilder<Arrow::Output>,
{
    pub fn new(fields: Vec<GenericField>) -> Result<Self> {
        Ok(Self {
            builder: generic_sinks::build_struct_array_builder::<Arrow>(
                String::from("$"),
                &fields,
            )?,
            fields,
        })
    }

    pub fn push<T: Serialize + ?Sized>(&mut self, item: &T) -> Result<()> {
        item.serialize(EventSerializer(&mut self.builder))?;
        Ok(())
    }

    pub fn extend<T: Serialize + ?Sized>(&mut self, items: &T) -> Result<()> {
        let mut builder = StripOuterSequenceSink::new(&mut self.builder);
        items.serialize(EventSerializer(&mut builder))?;
        Ok(())
    }

    pub fn build_arrays(&mut self) -> Result<Vec<Arrow::Output>> {
        let mut builder =
            generic_sinks::build_struct_array_builder::<Arrow>(String::from("$"), &self.fields)?;
        std::mem::swap(&mut builder, &mut self.builder);

        builder.finish()?;
        builder.build_arrays()
    }
}
