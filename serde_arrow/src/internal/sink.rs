pub mod macros;

use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

use crate::internal::{
    error::{fail, Error, Result},
    event::Event,
};

/// Serialize a type into an [EventSink]
///
/// This function may be helpful when creating custom formats.
///
pub fn serialize_into_sink<T: Serialize + ?Sized, S: EventSink>(
    sink: &mut S,
    value: &T,
) -> Result<()> {
    value.serialize(EventSerializer(sink))?;
    sink.finish()?;
    Ok(())
}

/// Processes [Events][Event] emitted during serialization of a type
///
/// Note: both the generic `accept` and the specific `accept_*` methods may be
/// called and must result in the same behavior. In the default implementation,
/// this is accomplished by forwarding any of the specific methods to the
/// generic method. When implementing the logic in terms of the specific
/// methods, the generic `accept` method must be implemented to forward to the
/// specific methods.
///
/// For example, to implement the behavior in the generic accept method use:
///
/// ```ignore
/// fn accept(&mut self, event: Event<'_>) -> Result<()> {
///     match event {
///         Event::I8(val) => { /* some action */},
///         ev => fail!("Unknown event {ev}"),
///     }
/// }
/// ```
///
/// To implement the behavior in the specific methods use:
///
/// ```ignore
/// fn accept(&mut self, event: Event<'_>) -> Result<()> {
///     match event {
///         Event::I8(val) => self.accept_i8(val),
///         ev => fail!("Unknown event {ev}"),
///     }
/// }
///
/// fn accept_i8(&mut self, val: i8) -> Result<()> {
///     /* some action */
/// }
/// ```
///
/// The specific methods can be much more performant in practice, but are more
/// complicated to implement.
///
pub trait EventSink {
    fn accept_start_sequence(&mut self) -> Result<()>;
    fn accept_end_sequence(&mut self) -> Result<()>;
    fn accept_start_tuple(&mut self) -> Result<()>;
    fn accept_end_tuple(&mut self) -> Result<()>;
    fn accept_start_struct(&mut self) -> Result<()>;
    fn accept_end_struct(&mut self) -> Result<()>;
    fn accept_start_map(&mut self) -> Result<()>;
    fn accept_end_map(&mut self) -> Result<()>;
    fn accept_item(&mut self) -> Result<()>;
    fn accept_some(&mut self) -> Result<()>;
    fn accept_null(&mut self) -> Result<()>;
    fn accept_default(&mut self) -> Result<()>;
    fn accept_str(&mut self, val: &str) -> Result<()>;
    fn accept_variant(&mut self, name: &str, idx: usize) -> Result<()>;
    fn accept_bool(&mut self, val: bool) -> Result<()>;
    fn accept_i8(&mut self, val: i8) -> Result<()>;
    fn accept_i16(&mut self, val: i16) -> Result<()>;
    fn accept_i32(&mut self, val: i32) -> Result<()>;
    fn accept_i64(&mut self, val: i64) -> Result<()>;
    fn accept_u8(&mut self, val: u8) -> Result<()>;
    fn accept_u16(&mut self, val: u16) -> Result<()>;
    fn accept_u32(&mut self, val: u32) -> Result<()>;
    fn accept_u64(&mut self, val: u64) -> Result<()>;
    fn accept_f32(&mut self, val: f32) -> Result<()>;
    fn accept_f64(&mut self, val: f64) -> Result<()>;
    fn accept(&mut self, event: Event<'_>) -> Result<()>;
    fn finish(&mut self) -> Result<()>;
}

#[allow(unused)]
pub(crate) struct DebugSink<E> {
    wrapped: E,
}

impl<E> DebugSink<E> {
    #[allow(unused)]
    pub fn new(wrapped: E) -> Self {
        Self { wrapped }
    }

    #[allow(unused)]
    pub fn into_inner(self) -> E {
        self.wrapped
    }
}

impl<E: EventSink> EventSink for DebugSink<E> {
    macros::forward_specialized_to_generic!();

    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        println!("{event}");
        self.wrapped.accept(event)
    }

    fn finish(&mut self) -> Result<()> {
        self.wrapped.finish()
    }
}

pub(crate) struct StripOuterSequenceSink<E> {
    wrapped: E,
    state: StripOuterSequenceState,
}

#[derive(Debug, Clone, Copy)]
enum StripOuterSequenceState {
    WaitForStart,
    WaitForItem,
    Item(usize),
}

impl<E> StripOuterSequenceSink<E> {
    pub fn new(wrapped: E) -> Self {
        Self {
            wrapped,
            state: StripOuterSequenceState::WaitForStart,
        }
    }

    pub fn into_inner(self) -> E {
        self.wrapped
    }
}

impl<E: EventSink> EventSink for StripOuterSequenceSink<E> {
    macros::forward_generic_to_specialized!();
    macros::accept_start!((this, ev, val, next) {
        use StripOuterSequenceState::*;
        this.state = match this.state {
            WaitForStart => WaitForItem,
            Item(depth) => {
                next(&mut this.wrapped, val)?;
                Item(depth + 1)
            }
            state => fail!("Invalid event {ev} in state {state:?} for StripOuterSequence"),
        };
        Ok(())
    });
    macros::accept_end!((this, ev, val, next) {
        use StripOuterSequenceState::*;
        this.state = match this.state {
            Item(1) => {
                next(&mut this.wrapped, val)?;
                WaitForItem
            }
            Item(depth) if depth > 1 => {
                next(&mut this.wrapped, val)?;
                Item(depth - 1)
            }
            WaitForItem => WaitForStart,
            state => fail!("Invalid event {ev} in state {state:?} for StripOuterSequence"),
        };
        Ok(())
    });
    macros::accept_value!((this, ev, val, next) {
        use StripOuterSequenceState::*;
        this.state = match this.state {
            Item(0) => {
                next(&mut this.wrapped, val)?;
                WaitForItem
            }
            Item(depth) => {
                next(&mut this.wrapped, val)?;
                Item(depth)
            }
            state => fail!("Invalid event {ev} in state {state:?} for StripOuterSequence"),
        };
        Ok(())
    });
    macros::accept_marker!((this, ev, val, next) {
        use StripOuterSequenceState::*;
        this.state = match this.state {
            WaitForItem if matches!(ev, Event::Item) => Item(0),
            Item(depth) => {
                next(&mut this.wrapped, val)?;
                Item(depth)
            }
            state => fail!("Invalid event {ev} in state {state:?} for StripOuterSequence"),
        };
        Ok(())
    });

    fn finish(&mut self) -> Result<()> {
        self.wrapped.finish()
    }
}

impl EventSink for Vec<Event<'static>> {
    macros::forward_specialized_to_generic!();

    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        self.push(event.to_static());
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: EventSink> EventSink for Box<T> {
    macros::accept_start!((this, _ev, val, next) {
        next(this.as_mut(), val)
    });
    macros::accept_end!((this, _ev, val, next) {
        next(this.as_mut(), val)
    });
    macros::accept_marker!((this, _ev, val, next) {
        next(this.as_mut(), val)
    });
    macros::accept_value!((this, _ev, val, next) {
        next(this.as_mut(), val)
    });

    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        self.as_mut().accept(event)
    }

    fn finish(&mut self) -> Result<()> {
        self.as_mut().finish()
    }
}

pub trait ArrayBuilder<A>: EventSink {
    /// Build the arrays and clear any internal buffers
    ///
    /// Note: some builder may retain internal state, e.g., dictionary builds
    /// keep the previous key mapping. This behavior allows to write separate
    /// chunks of dictionary arrays.
    fn build_array(&mut self) -> Result<A>;
}

impl<A, T: ArrayBuilder<A>> ArrayBuilder<A> for Box<T> {
    fn build_array(&mut self) -> Result<A> {
        self.as_mut().build_array()
    }
}

pub struct DynamicArrayBuilder<A> {
    builder: Box<dyn ArrayBuilder<A>>,
}

impl<A> DynamicArrayBuilder<A> {
    pub fn new<B: ArrayBuilder<A> + 'static>(builder: B) -> Self {
        Self {
            builder: Box::new(builder),
        }
    }
}

impl<E: EventSink> EventSink for &mut E {
    macros::accept_start!((this, _ev, val, next) {
        next(*this, val)
    });
    macros::accept_end!((this, _ev, val, next) {
        next(*this, val)
    });
    macros::accept_marker!((this, _ev, val, next) {
        next(*this, val)
    });
    macros::accept_value!((this, _ev, val, next) {
        next(*this, val)
    });

    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        (*self).accept(event)
    }

    fn finish(&mut self) -> Result<()> {
        (*self).finish()
    }
}

impl<A> EventSink for DynamicArrayBuilder<A> {
    macros::accept_start!((this, _ev, val, next) {
        next(this.builder.as_mut(), val)
    });
    macros::accept_end!((this, _ev, val, next) {
        next(this.builder.as_mut(), val)
    });
    macros::accept_marker!((this, _ev, val, next) {
        next(this.builder.as_mut(), val)
    });
    macros::accept_value!((this, _ev, val, next) {
        next(this.builder.as_mut(), val)
    });

    fn accept(&mut self, event: Event<'_>) -> Result<()> {
        self.builder.accept(event)
    }

    fn finish(&mut self) -> Result<()> {
        self.builder.finish()
    }
}

impl<A> ArrayBuilder<A> for DynamicArrayBuilder<A> {
    fn build_array(&mut self) -> Result<A> {
        self.builder.build_array()
    }
}

impl<A> From<Box<dyn ArrayBuilder<A>>> for DynamicArrayBuilder<A> {
    fn from(builder: Box<dyn ArrayBuilder<A>>) -> Self {
        Self { builder }
    }
}

pub(crate) struct EventSerializer<'a, S>(pub &'a mut S);

impl<'a, S: EventSink> Serializer for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, val: bool) -> Result<()> {
        self.0.accept_bool(val)
    }

    fn serialize_i8(self, val: i8) -> Result<()> {
        self.0.accept_i8(val)
    }

    fn serialize_i16(self, val: i16) -> Result<()> {
        self.0.accept_i16(val)
    }

    fn serialize_i32(self, val: i32) -> Result<()> {
        self.0.accept_i32(val)
    }

    fn serialize_i64(self, val: i64) -> Result<()> {
        self.0.accept_i64(val)
    }

    fn serialize_u8(self, val: u8) -> Result<()> {
        self.0.accept_u8(val)
    }

    fn serialize_u16(self, val: u16) -> Result<()> {
        self.0.accept_u16(val)
    }

    fn serialize_u32(self, val: u32) -> Result<()> {
        self.0.accept_u32(val)
    }

    fn serialize_u64(self, val: u64) -> Result<()> {
        self.0.accept_u64(val)
    }

    fn serialize_f32(self, val: f32) -> Result<()> {
        self.0.accept_f32(val)
    }

    fn serialize_f64(self, val: f64) -> Result<()> {
        self.0.accept_f64(val)
    }

    fn serialize_char(self, val: char) -> Result<()> {
        self.0.accept_u32(u32::from(val))
    }

    fn serialize_str(self, val: &str) -> Result<()> {
        self.0.accept_str(val)
    }

    fn serialize_bytes(self, val: &[u8]) -> Result<()> {
        self.0.accept_start_sequence()?;
        for &b in val {
            self.0.accept_item()?;
            self.0.accept_u8(b)?;
        }
        self.0.accept_end_sequence()?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.0.accept_null()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
        self.0.accept_some()?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.0.accept_null()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.0.accept_start_sequence()?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.0.accept_start_tuple()?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.0.accept_start_tuple()?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.0.accept_start_map()?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.0.accept_start_struct()?;
        Ok(self)
    }

    // Union support
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.0.accept_variant(variant, variant_index as usize)?;
        self.0.accept_null()
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        self.0.accept_variant(variant, variant_index as usize)?;
        value.serialize(EventSerializer(&mut *self.0))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.0.accept_variant(variant, variant_index as usize)?;
        self.0.accept_start_tuple()?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.0.accept_variant(variant, variant_index as usize)?;
        self.0.accept_start_struct()?;
        Ok(self)
    }
}

impl<'a, S: EventSink> SerializeSeq for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.0.accept_item()?;
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.0.accept_end_sequence()?;
        Ok(())
    }
}

impl<'a, S: EventSink> SerializeTuple for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.0.accept_item()?;
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.0.accept_end_tuple()?;
        Ok(())
    }
}

impl<'a, S: EventSink> SerializeTupleStruct for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        self.0.accept_item()?;
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.0.accept_end_tuple()?;
        Ok(())
    }
}

impl<'a, S: EventSink> SerializeTupleVariant for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        self.0.accept_item()?;
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.0.accept_end_tuple()?;
        Ok(())
    }
}

impl<'a, S: EventSink> SerializeStruct for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.0.accept_str(key)?;
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.0.accept_end_struct()?;
        Ok(())
    }
}

impl<'a, S: EventSink> SerializeStructVariant for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.0.accept_str(key)?;
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.0.accept_end_struct()?;
        Ok(())
    }
}

impl<'a, S: EventSink> SerializeMap for EventSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(EventSerializer(&mut *self.0))?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.accept_end_map()?;
        Ok(())
    }
}
