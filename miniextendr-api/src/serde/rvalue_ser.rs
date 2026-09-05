//! Serializer for converting Rust values to [`RValue`] via serde.
//!
//! The owned, `Send` counterpart of [`super::ser::RSerializer`]: the same type
//! mapping, but the output is an [`RValue`] tree instead of a live `SEXP`, so it
//! can be built on any thread and travel through `panic_any`. This is what
//! condition `data =` payloads and `#[miniextendr(serde_error)]` need: a
//! serialized error's fields have to cross the unwind (and possibly the
//! worker→main thread) boundary before any R object exists.
//!
//! # Type mapping
//!
//! | Rust | [`RValue`] |
//! |---|---|
//! | `bool` | `Logical([Some(b)])` |
//! | `i8` / `i16` / `i32` / `u8` / `u16` | `Integer([Some(v)])` |
//! | `i64` / `u32` / `u64` | `Integer` when the value fits `i32` and is not `NA_integer_`, else `Double` |
//! | `f32` / `f64` | `Double([v])` |
//! | `char` / `str` / `String` | `Character([Some(s)])` |
//! | `serialize_bytes` | `Raw` |
//! | `()` / unit struct / `None` | `Null` |
//! | unit enum variant | `Character([Some("Variant")])` |
//! | newtype struct | transparent |
//! | newtype / tuple / struct enum variant | `List([("Variant", payload)])` |
//! | sequence of same-kind scalars | one atomic vector |
//! | other sequences, tuples | unnamed `List` |
//! | struct, map with string keys | named `List` |
//!
//! `None` becomes `Null`, not a typed `NA`: the serializer never sees the
//! inner type of an absent `Option`. A `Vec<Option<T>>` therefore becomes a
//! list with `NULL` holes, exactly as [`RSerializer::to_sexp`] renders it.
//!
//! [`RSerializer::to_sexp`]: super::ser::RSerializer::to_sexp

use super::error::RSerdeError;
use crate::rvalue::RValue;
use serde::ser::{self, Serialize};

// region: Public entry points

/// Serializer that converts Rust values to an owned [`RValue`] tree.
///
/// ```
/// use miniextendr_api::RValue;
/// use miniextendr_api::serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point { x: f64, tag: String }
///
/// let v = RValue::from_serde(&Point { x: 1.5, tag: "a".into() }).unwrap();
/// match v {
///     RValue::List(fields) => {
///         assert_eq!(fields[0].0.as_deref(), Some("x"));
///         assert!(matches!(&fields[0].1, RValue::Double(d) if d == &[1.5]));
///     }
///     other => panic!("expected a named list, got {other:?}"),
/// }
/// ```
pub struct RValueSerializer;

/// Serialize any `Serialize` value into an [`RValue`].
pub fn to_rvalue<T: ?Sized + Serialize>(value: &T) -> Result<RValue, RSerdeError> {
    value.serialize(RValueSerializer)
}

impl RValue {
    /// Build an [`RValue`] from any `Serialize` value; see [`to_rvalue`] and
    /// the [module docs](self) for the type mapping.
    pub fn from_serde<T: ?Sized + Serialize>(value: &T) -> Result<Self, RSerdeError> {
        to_rvalue(value)
    }
}

// endregion

// region: Scalar and container helpers

/// `i64` policy shared with `IntoR for i64`: `Integer` when the value fits an
/// `i32` other than `NA_integer_`, otherwise `Double` (lossy past 2^53, as R
/// itself is).
fn int_or_double(v: i64) -> RValue {
    match i32::try_from(v) {
        Ok(i) if i != i32::MIN => RValue::Integer(vec![Some(i)]),
        // Deliberate widening: R has no 64-bit integer; this mirrors the
        // REALSXP fallback of `i64::into_sexp`.
        _ => RValue::Double(vec![v as f64]),
    }
}

fn unnamed_list(elements: Vec<RValue>) -> RValue {
    RValue::List(elements.into_iter().map(|e| (None, e)).collect())
}

fn tagged(variant: &str, payload: RValue) -> RValue {
    RValue::List(vec![(Some(variant.to_string()), payload)])
}

/// Which atomic kind a length-1 `RValue` is, for sequence coalescing.
fn scalar_kind(v: &RValue) -> Option<u8> {
    match v {
        RValue::Logical(x) if x.len() == 1 => Some(0),
        RValue::Integer(x) if x.len() == 1 => Some(1),
        RValue::Double(x) if x.len() == 1 => Some(2),
        RValue::Complex(x) if x.len() == 1 => Some(3),
        RValue::Character(x) if x.len() == 1 => Some(4),
        RValue::Raw(x) if x.len() == 1 => Some(5),
        _ => None,
    }
}

/// Homogeneous length-1 scalars become one atomic vector (the `RValue` twin of
/// `List::from_scalars_or_list`); anything else stays an unnamed list.
fn coalesce(elements: Vec<RValue>) -> RValue {
    let Some(kind) = elements.first().and_then(scalar_kind) else {
        return unnamed_list(elements);
    };
    if !elements.iter().all(|e| scalar_kind(e) == Some(kind)) {
        return unnamed_list(elements);
    }
    macro_rules! concat {
        ($variant:path) => {
            $variant(
                elements
                    .into_iter()
                    .flat_map(|e| match e {
                        $variant(v) => v,
                        // `scalar_kind` proved every element is this variant.
                        _ => unreachable!("coalesce: kind checked above"),
                    })
                    .collect(),
            )
        };
    }
    match kind {
        0 => concat!(RValue::Logical),
        1 => concat!(RValue::Integer),
        2 => concat!(RValue::Double),
        3 => concat!(RValue::Complex),
        4 => concat!(RValue::Character),
        _ => concat!(RValue::Raw),
    }
}

/// A map key must serialize to exactly one string.
fn key_string(key: RValue) -> Result<String, RSerdeError> {
    match key {
        RValue::Character(mut v) if v.len() == 1 => {
            v.pop().flatten().ok_or(RSerdeError::NonStringKey)
        }
        _ => Err(RSerdeError::NonStringKey),
    }
}

// endregion

// region: serde::Serializer for RValueSerializer

impl ser::Serializer for RValueSerializer {
    type Ok = RValue;
    type Error = RSerdeError;

    type SerializeSeq = RValueSeq;
    type SerializeTuple = RValueSeq;
    type SerializeTupleStruct = RValueSeq;
    type SerializeTupleVariant = RValueTupleVariant;
    type SerializeMap = RValueMap;
    type SerializeStruct = RValueStruct;
    type SerializeStructVariant = RValueStructVariant;

    fn serialize_bool(self, v: bool) -> Result<RValue, RSerdeError> {
        Ok(RValue::Logical(vec![Some(v)]))
    }

    fn serialize_i8(self, v: i8) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_i16(self, v: i16) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_i32(self, v: i32) -> Result<RValue, RSerdeError> {
        Ok(int_or_double(i64::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<RValue, RSerdeError> {
        Ok(int_or_double(v))
    }

    fn serialize_u8(self, v: u8) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_u16(self, v: u16) -> Result<RValue, RSerdeError> {
        Ok(RValue::Integer(vec![Some(i32::from(v))]))
    }

    fn serialize_u32(self, v: u32) -> Result<RValue, RSerdeError> {
        Ok(int_or_double(i64::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<RValue, RSerdeError> {
        Ok(match i64::try_from(v) {
            Ok(i) => int_or_double(i),
            // Same deliberate widening as `int_or_double`.
            Err(_) => RValue::Double(vec![v as f64]),
        })
    }

    fn serialize_f32(self, v: f32) -> Result<RValue, RSerdeError> {
        Ok(RValue::Double(vec![f64::from(v)]))
    }

    fn serialize_f64(self, v: f64) -> Result<RValue, RSerdeError> {
        Ok(RValue::Double(vec![v]))
    }

    fn serialize_char(self, v: char) -> Result<RValue, RSerdeError> {
        Ok(RValue::Character(vec![Some(v.to_string())]))
    }

    fn serialize_str(self, v: &str) -> Result<RValue, RSerdeError> {
        Ok(RValue::Character(vec![Some(v.to_string())]))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<RValue, RSerdeError> {
        Ok(RValue::Raw(v.to_vec()))
    }

    fn serialize_none(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<RValue, RSerdeError> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<RValue, RSerdeError> {
        Ok(RValue::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<RValue, RSerdeError> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<RValue, RSerdeError> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<RValue, RSerdeError> {
        Ok(tagged(variant, to_rvalue(value)?))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<RValueSeq, RSerdeError> {
        Ok(RValueSeq::with_capacity(len.unwrap_or(0)))
    }

    fn serialize_tuple(self, len: usize) -> Result<RValueSeq, RSerdeError> {
        Ok(RValueSeq::with_capacity(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<RValueSeq, RSerdeError> {
        Ok(RValueSeq::with_capacity(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<RValueTupleVariant, RSerdeError> {
        Ok(RValueTupleVariant {
            variant,
            elements: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<RValueMap, RSerdeError> {
        Ok(RValueMap {
            pending_key: None,
            fields: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<RValueStruct, RSerdeError> {
        Ok(RValueStruct {
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<RValueStructVariant, RSerdeError> {
        Ok(RValueStructVariant {
            variant,
            inner: RValueStruct {
                fields: Vec::with_capacity(len),
            },
        })
    }
}

/// Sequence / tuple accumulator: sequences coalesce homogeneous scalars, tuples
/// always stay lists.
pub struct RValueSeq {
    elements: Vec<RValue>,
}

impl RValueSeq {
    fn with_capacity(n: usize) -> Self {
        RValueSeq {
            elements: Vec::with_capacity(n),
        }
    }
}

impl ser::SerializeSeq for RValueSeq {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(coalesce(self.elements))
    }
}

impl ser::SerializeTuple for RValueSeq {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(unnamed_list(self.elements))
    }
}

impl ser::SerializeTupleStruct for RValueSeq {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(unnamed_list(self.elements))
    }
}

/// `Enum::Variant(a, b)` → `List([("Variant", List([a, b]))])`.
pub struct RValueTupleVariant {
    variant: &'static str,
    elements: Vec<RValue>,
}

impl ser::SerializeTupleVariant for RValueTupleVariant {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(tagged(self.variant, unnamed_list(self.elements)))
    }
}

/// Map accumulator; keys must serialize to a single string.
pub struct RValueMap {
    pending_key: Option<String>,
    fields: Vec<(Option<String>, RValue)>,
}

impl ser::SerializeMap for RValueMap {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        self.pending_key = Some(key_string(to_rvalue(key)?)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| ser::Error::custom("serialize_value called before serialize_key"))?;
        self.fields.push((Some(key), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::List(self.fields))
    }
}

/// Struct accumulator → named list in field order.
pub struct RValueStruct {
    fields: Vec<(Option<String>, RValue)>,
}

impl ser::SerializeStruct for RValueStruct {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fields.push((Some(key.to_string()), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(RValue::List(self.fields))
    }
}

/// `Enum::Variant { a, b }` → `List([("Variant", List([("a", ..), ("b", ..)]))])`.
pub struct RValueStructVariant {
    variant: &'static str,
    inner: RValueStruct,
}

impl ser::SerializeStructVariant for RValueStructVariant {
    type Ok = RValue;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        ser::SerializeStruct::serialize_field(&mut self.inner, key, value)
    }

    fn end(self) -> Result<RValue, RSerdeError> {
        Ok(tagged(self.variant, ser::SerializeStruct::end(self.inner)?))
    }
}

// endregion

// region: Tagged error parts — variant name + payload fields for serde_error

/// Outcome of [`tagged_parts`]: the variant (if the value carried one) and the
/// payload fields in declaration order.
pub(crate) type TaggedParts = (Option<String>, Vec<(String, RValue)>);

/// Split a serialized error into its variant name and its fields.
///
/// - Externally tagged enums (serde's default) report the variant from the
///   `serialize_*_variant` call: a struct variant's fields are the payload, a
///   newtype variant with a struct payload contributes that struct's fields,
///   any other newtype or tuple payload lands under one field named `value`.
/// - Internally tagged enums (`#[serde(tag = "kind")]`) serialize as a struct
///   whose `tag` field holds the variant; that field is consumed, the others
///   are the payload.
/// - Anything else (a plain struct, a map, a string, a sequence) has no variant;
///   struct and map fields become the payload, scalars and sequences contribute
///   nothing.
pub(crate) fn tagged_parts<E: ?Sized + Serialize>(
    e: &E,
    tag: &str,
) -> Result<TaggedParts, RSerdeError> {
    e.serialize(TaggedSerializer { tag })
}

fn payload_fields(inner: RValue) -> Vec<(String, RValue)> {
    match inner {
        RValue::List(pairs)
            if !pairs.is_empty()
                && pairs
                    .iter()
                    .all(|(n, _)| n.as_deref().is_some_and(|n| !n.is_empty())) =>
        {
            pairs
                .into_iter()
                .map(|(name, value)| (name.unwrap_or_default(), value))
                .collect()
        }
        other => vec![("value".to_string(), other)],
    }
}

fn split_tag(tag: &str, fields: Vec<(String, RValue)>) -> TaggedParts {
    let mut variant = None;
    let mut rest = Vec::with_capacity(fields.len());
    for (name, value) in fields {
        match value {
            RValue::Character(mut v) if variant.is_none() && name == tag && v.len() == 1 => {
                variant = v.pop().flatten();
            }
            value => rest.push((name, value)),
        }
    }
    (variant, rest)
}

struct TaggedSerializer<'a> {
    tag: &'a str,
}

/// Compound accumulator for shapes that carry no variant information.
pub(crate) struct TaggedSwallow;

/// `Enum::Variant(a, b)` → variant plus one `value` field holding the tuple.
pub(crate) struct TaggedTupleVariant {
    variant: &'static str,
    elements: Vec<RValue>,
}

/// A struct or map at the top level: fields, with the tag field split off.
pub(crate) struct TaggedFields<'a> {
    tag: &'a str,
    pending_key: Option<String>,
    fields: Vec<(String, RValue)>,
}

/// `Enum::Variant { a, b }` → variant plus its fields.
pub(crate) struct TaggedStructVariant {
    variant: &'static str,
    fields: Vec<(String, RValue)>,
}

const NO_VARIANT: TaggedParts = (None, Vec::new());

impl<'a> ser::Serializer for TaggedSerializer<'a> {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    type SerializeSeq = TaggedSwallow;
    type SerializeTuple = TaggedSwallow;
    type SerializeTupleStruct = TaggedSwallow;
    type SerializeTupleVariant = TaggedTupleVariant;
    type SerializeMap = TaggedFields<'a>;
    type SerializeStruct = TaggedFields<'a>;
    type SerializeStructVariant = TaggedStructVariant;

    fn serialize_bool(self, _: bool) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i8(self, _: i8) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i16(self, _: i16) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i32(self, _: i32) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_i64(self, _: i64) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u8(self, _: u8) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u16(self, _: u16) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u32(self, _: u32) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_u64(self, _: u64) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_f32(self, _: f32) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_f64(self, _: f64) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_char(self, _: char) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_str(self, _: &str) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_none(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<TaggedParts, RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<TaggedParts, RSerdeError> {
        Ok((Some(variant.to_string()), Vec::new()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<TaggedParts, RSerdeError> {
        // Transparent, like the value serializer: keep intercepting.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<TaggedParts, RSerdeError> {
        Ok((Some(variant.to_string()), payload_fields(to_rvalue(value)?)))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<TaggedSwallow, RSerdeError> {
        Ok(TaggedSwallow)
    }
    fn serialize_tuple(self, _len: usize) -> Result<TaggedSwallow, RSerdeError> {
        Ok(TaggedSwallow)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<TaggedSwallow, RSerdeError> {
        Ok(TaggedSwallow)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TaggedTupleVariant, RSerdeError> {
        Ok(TaggedTupleVariant {
            variant,
            elements: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<TaggedFields<'a>, RSerdeError> {
        Ok(TaggedFields {
            tag: self.tag,
            pending_key: None,
            fields: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<TaggedFields<'a>, RSerdeError> {
        Ok(TaggedFields {
            tag: self.tag,
            pending_key: None,
            fields: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TaggedStructVariant, RSerdeError> {
        Ok(TaggedStructVariant {
            variant,
            fields: Vec::with_capacity(len),
        })
    }
}

impl ser::SerializeSeq for TaggedSwallow {
    type Ok = TaggedParts;
    type Error = RSerdeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
}

impl ser::SerializeTuple for TaggedSwallow {
    type Ok = TaggedParts;
    type Error = RSerdeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
}

impl ser::SerializeTupleStruct for TaggedSwallow {
    type Ok = TaggedParts;
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(NO_VARIANT)
    }
}

impl ser::SerializeTupleVariant for TaggedTupleVariant {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        self.elements.push(to_rvalue(value)?);
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok((
            Some(self.variant.to_string()),
            vec![("value".to_string(), unnamed_list(self.elements))],
        ))
    }
}

impl ser::SerializeMap for TaggedFields<'_> {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        self.pending_key = Some(key_string(to_rvalue(key)?)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| ser::Error::custom("serialize_value called before serialize_key"))?;
        self.fields.push((key, to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(split_tag(self.tag, self.fields))
    }
}

impl ser::SerializeStruct for TaggedFields<'_> {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fields.push((key.to_string(), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok(split_tag(self.tag, self.fields))
    }
}

impl ser::SerializeStructVariant for TaggedStructVariant {
    type Ok = TaggedParts;
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fields.push((key.to_string(), to_rvalue(value)?));
        Ok(())
    }

    fn end(self) -> Result<TaggedParts, RSerdeError> {
        Ok((Some(self.variant.to_string()), self.fields))
    }
}

// endregion

// region: Tests (pure Rust, no R runtime)

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::collections::BTreeMap;

    fn named(v: &RValue) -> Vec<(&str, &RValue)> {
        match v {
            RValue::List(pairs) => pairs
                .iter()
                .map(|(n, v)| (n.as_deref().unwrap_or(""), v))
                .collect(),
            other => panic!("expected a list, got {other:?}"),
        }
    }

    #[test]
    fn scalars_map_like_the_sexp_serializer() {
        assert!(matches!(to_rvalue(&true).unwrap(), RValue::Logical(v) if v == vec![Some(true)]));
        assert!(matches!(to_rvalue(&7u8).unwrap(), RValue::Integer(v) if v == vec![Some(7)]));
        assert!(matches!(to_rvalue(&1.5f32).unwrap(), RValue::Double(v) if v == vec![1.5]));
        assert!(
            matches!(to_rvalue(&'x').unwrap(), RValue::Character(v) if v == vec![Some("x".to_string())])
        );
        assert!(matches!(to_rvalue(&()).unwrap(), RValue::Null));
        assert!(matches!(to_rvalue(&None::<i32>).unwrap(), RValue::Null));
        assert!(
            matches!(to_rvalue(&Some(3i32)).unwrap(), RValue::Integer(v) if v == vec![Some(3)])
        );
    }

    #[test]
    fn wide_integers_follow_the_i64_policy() {
        assert!(matches!(to_rvalue(&5i64).unwrap(), RValue::Integer(v) if v == vec![Some(5)]));
        assert!(
            matches!(to_rvalue(&(1i64 << 40)).unwrap(), RValue::Double(v) if v == vec![(1i64 << 40) as f64])
        );
        // NA_integer_ must not be produced from a real value.
        assert!(matches!(
            to_rvalue(&i64::from(i32::MIN)).unwrap(),
            RValue::Double(_)
        ));
        assert!(matches!(to_rvalue(&u64::MAX).unwrap(), RValue::Double(_)));
        assert!(matches!(to_rvalue(&u32::MAX).unwrap(), RValue::Double(_)));
    }

    #[test]
    fn homogeneous_sequences_coalesce_mixed_ones_stay_lists() {
        assert!(
            matches!(to_rvalue(&vec![1i32, 2, 3]).unwrap(), RValue::Integer(v) if v == vec![Some(1), Some(2), Some(3)])
        );
        assert!(
            matches!(to_rvalue(&["a", "b"][..]).unwrap(), RValue::Character(v) if v.len() == 2)
        );
        // Fixed-size arrays are tuples to serde, so they stay lists.
        assert!(matches!(to_rvalue(&["a", "b"]).unwrap(), RValue::List(v) if v.len() == 2));
        match to_rvalue(&vec![Some(1i32), None, Some(3)]).unwrap() {
            RValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(items.iter().all(|(n, _)| n.is_none()));
                assert!(matches!(items[1].1, RValue::Null));
            }
            other => panic!("expected a list, got {other:?}"),
        }
        // Tuples never coalesce.
        assert!(matches!(to_rvalue(&(1i32, 2i32)).unwrap(), RValue::List(v) if v.len() == 2));
        assert!(matches!(to_rvalue(&Vec::<i32>::new()).unwrap(), RValue::List(v) if v.is_empty()));
    }

    #[derive(Serialize)]
    struct Inner {
        flag: bool,
    }

    #[derive(Serialize)]
    struct Outer {
        id: i32,
        name: String,
        inner: Inner,
        weights: Vec<f64>,
    }

    #[test]
    fn structs_become_named_lists_in_field_order() {
        let v = to_rvalue(&Outer {
            id: 1,
            name: "n".into(),
            inner: Inner { flag: false },
            weights: vec![0.5, 1.5],
        })
        .unwrap();
        let fields = named(&v);
        assert_eq!(
            fields.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            ["id", "name", "inner", "weights"]
        );
        assert!(
            matches!(fields[2].1, RValue::List(inner) if inner[0].0.as_deref() == Some("flag"))
        );
        assert!(matches!(fields[3].1, RValue::Double(w) if w == &[0.5, 1.5]));
    }

    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    enum Shape {
        Unit,
        Newtype(i32),
        Tuple(i32, String),
        Struct { w: f64 },
    }

    #[test]
    fn enum_variants_follow_the_external_tagging_shapes() {
        assert!(
            matches!(to_rvalue(&Shape::Unit).unwrap(), RValue::Character(v) if v == vec![Some("unit".to_string())])
        );
        let nt = to_rvalue(&Shape::Newtype(4)).unwrap();
        assert!(
            matches!(&named(&nt)[..], [("newtype", RValue::Integer(v))] if v == &vec![Some(4)])
        );
        let tup = to_rvalue(&Shape::Tuple(1, "a".into())).unwrap();
        assert!(matches!(&named(&tup)[..], [("tuple", RValue::List(items))] if items.len() == 2));
        let st = to_rvalue(&Shape::Struct { w: 2.0 }).unwrap();
        assert!(
            matches!(&named(&st)[..], [("struct", RValue::List(f))] if f[0].0.as_deref() == Some("w"))
        );
    }

    #[test]
    fn maps_need_string_keys() {
        let mut m = BTreeMap::new();
        m.insert("b".to_string(), 2i32);
        m.insert("a".to_string(), 1i32);
        let v = to_rvalue(&m).unwrap();
        assert_eq!(
            named(&v).iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            ["a", "b"]
        );

        let mut bad = BTreeMap::new();
        bad.insert(1i32, "x");
        assert!(matches!(to_rvalue(&bad), Err(RSerdeError::NonStringKey)));
    }

    #[derive(Serialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Tagged {
        MissingField { field: String },
        Io,
    }

    #[derive(Serialize)]
    enum External {
        Bad { code: i32 },
        Plain(String),
        Wrapped(Inner),
        Unit,
    }

    #[test]
    fn tagged_parts_internal_tag_is_consumed() {
        let (variant, fields) =
            tagged_parts(&Tagged::MissingField { field: "id".into() }, "kind").unwrap();
        assert_eq!(variant.as_deref(), Some("missing_field"));
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "field");
        assert!(matches!(&fields[0].1, RValue::Character(v) if v == &vec![Some("id".to_string())]));

        let (variant, fields) = tagged_parts(&Tagged::Io, "kind").unwrap();
        assert_eq!(variant.as_deref(), Some("io"));
        assert!(fields.is_empty());
    }

    #[test]
    fn tagged_parts_external_variants() {
        let (variant, fields) = tagged_parts(&External::Bad { code: 7 }, "kind").unwrap();
        assert_eq!(variant.as_deref(), Some("Bad"));
        assert_eq!(fields[0].0, "code");

        let (variant, fields) = tagged_parts(&External::Plain("x".into()), "kind").unwrap();
        assert_eq!(variant.as_deref(), Some("Plain"));
        assert_eq!(fields[0].0, "value");

        let (variant, fields) =
            tagged_parts(&External::Wrapped(Inner { flag: true }), "kind").unwrap();
        assert_eq!(variant.as_deref(), Some("Wrapped"));
        assert_eq!(fields[0].0, "flag");

        let (variant, fields) = tagged_parts(&External::Unit, "kind").unwrap();
        assert_eq!(variant.as_deref(), Some("Unit"));
        assert!(fields.is_empty());
    }

    #[test]
    fn tagged_parts_without_variant_information() {
        let (variant, fields) = tagged_parts(&Inner { flag: true }, "kind").unwrap();
        assert!(variant.is_none());
        assert_eq!(fields[0].0, "flag");

        let (variant, fields) = tagged_parts(&String::from("boom"), "kind").unwrap();
        assert!(variant.is_none());
        assert!(fields.is_empty());

        // A different tag name leaves the `kind` field as data.
        let (variant, fields) = tagged_parts(&Tagged::Io, "type").unwrap();
        assert!(variant.is_none());
        assert_eq!(fields[0].0, "kind");
    }
}

// endregion
