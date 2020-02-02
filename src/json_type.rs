use crate::fragment::fragment_components_from_fragment;
use std::{fmt::Debug, ops::Deref};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, EnumIter, Eq, Debug, Display, PartialEq)]
pub enum EnumJsonType {
    // We assume that all the drafts will have the same primitive types
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}

impl EnumJsonType {
    #[must_use]
    pub fn from_type(type_string: &str) -> Option<Self>
    where
        Self: Sized,
    {
        match type_string {
            "array" => Some(Self::Array),
            "boolean" => Some(Self::Boolean),
            "integer" => Some(Self::Integer),
            "null" => Some(Self::Null),
            "number" => Some(Self::Number),
            "object" => Some(Self::Object),
            "string" => Some(Self::String),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_type(&self) -> &str {
        match self {
            Self::Array => "array",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Null => "null",
            Self::Number => "number",
            Self::Object => "object",
            Self::String => "string",
        }
    }
}

pub trait JsonMapTrait<'json, T>
where
    T: 'json + JsonType<T>,
{
    #[must_use]
    fn keys(&'json self) -> Box<dyn Iterator<Item = &str> + 'json> {
        Box::new(self.items().map(|(key, _)| key))
    }

    #[must_use]
    fn values(&'json self) -> Box<dyn Iterator<Item = &T> + 'json> {
        Box::new(self.items().map(|(_, value)| value))
    }

    #[must_use]
    fn items(&'json self) -> Box<dyn Iterator<Item = (&str, &T)> + 'json>;
}

// This trait allows us to have a 1:1 mapping with serde_json, generally used by rust libraries
// but gives us the power to use different objects from serde_json. This gives us the ability
// to support usage of different data-types like PyObject from pyo3 in case of python bindings
pub trait JsonType<T>: Debug
where
    T: JsonType<T>,
{
    fn as_array<'json>(&'json self) -> Option<Box<dyn ExactSizeIterator<Item = &T> + 'json>>;
    fn as_boolean(&self) -> Option<bool>;
    fn as_integer(&self) -> Option<i128>;
    fn as_null(&self) -> Option<()>;
    fn as_number(&self) -> Option<f64>;
    fn as_object(&self) -> Option<JsonMap<T>>
    where
        for<'json> JsonMap<'json, T>: JsonMapTrait<'json, T>;
    fn as_string(&self) -> Option<&str>;

    fn get_attribute(&self, attribute_name: &str) -> Option<&T>;
    fn get_index(&self, index: usize) -> Option<&T>;

    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    fn is_object(&self) -> bool
    where
        for<'json> JsonMap<'json, T>: JsonMapTrait<'json, T>,
    {
        self.as_object().is_some()
    }

    fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    fn has_attribute(&self, attribute_name: &str) -> bool {
        self.get_attribute(attribute_name).is_some()
    }

    fn primitive_type(&self) -> EnumJsonType
    where
        for<'json> JsonMap<'json, T>: JsonMapTrait<'json, T>,
    {
        // This might not be efficient, but it could be comfortable to quickly extract the type especially while debugging
        if self.is_array() {
            EnumJsonType::Array
        } else if self.is_boolean() {
            EnumJsonType::Boolean
        } else if self.is_integer() {
            EnumJsonType::Integer
        } else if self.is_null() {
            EnumJsonType::Null
        } else if self.is_number() {
            EnumJsonType::Number
        } else if self.is_object() {
            EnumJsonType::Object
        } else if self.is_string() {
            EnumJsonType::String
        } else {
            #[allow(unsafe_code)]
            unsafe {
                unreachable::unreachable()
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait ThreadSafeJsonType<T>: JsonType<T> + Sync + Send
where
    T: JsonType<T>,
{
}

#[derive(Debug)]
pub struct JsonMap<'json, T>(&'json T)
where
    T: JsonType<T>;

impl<'json, T> JsonMap<'json, T>
where
    T: JsonType<T>,
{
    pub fn new(object: &'json T) -> Self {
        Self(object)
    }
}

impl<'json, T> Deref for JsonMap<'json, T>
where
    T: JsonType<T>,
{
    type Target = T;

    #[must_use]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn get_fragment<'json, T>(json_object: &'json T, fragment: &str) -> Option<&'json T>
where
    T: JsonType<T>,
    for<'_json_map> JsonMap<'_json_map, T>: JsonMapTrait<'_json_map, T>,
{
    let mut result: Option<&T> = Some(json_object);
    for fragment_part in fragment_components_from_fragment(fragment) {
        if let Some(value) = result {
            result = match value.primitive_type() {
                EnumJsonType::Object => value.get_attribute(fragment_part.as_str()),
                EnumJsonType::Array => fragment_part.parse::<usize>().map(|index| value.get_index(index)).ok().unwrap_or(None),
                _ => None,
            };
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{get_fragment, EnumJsonType, JsonType};
    use crate::rust_type::RustType;
    use test_case::test_case;

    #[test_case("array", Some(EnumJsonType::Array))]
    #[test_case("integer", Some(EnumJsonType::Integer))]
    #[test_case("number", Some(EnumJsonType::Number))]
    #[test_case("null", Some(EnumJsonType::Null))]
    #[test_case("object", Some(EnumJsonType::Object))]
    #[test_case("string", Some(EnumJsonType::String))]
    #[test_case("an invalid type", None)]
    fn test_enum_primitive_type_from_type(type_str: &str, expected_option_enum_primitive_type: Option<EnumJsonType>) {
        assert_eq!(EnumJsonType::from_type(type_str), expected_option_enum_primitive_type);
    }

    #[test_case(EnumJsonType::Array, "array")]
    #[test_case(EnumJsonType::Integer, "integer")]
    #[test_case(EnumJsonType::Number, "number")]
    #[test_case(EnumJsonType::Null, "null")]
    #[test_case(EnumJsonType::Object, "object")]
    #[test_case(EnumJsonType::String, "string")]
    fn test_enum_primitive_type_to_type(enum_primitive_type: EnumJsonType, expected_type_str: &str) {
        assert_eq!(enum_primitive_type.to_type(), expected_type_str);
    }

    #[test]
    fn test_ensure_that_trait_can_be_made_into_an_object() {
        let _: Option<Box<dyn JsonType<RustType>>> = None;
    }

    #[test_case("", &Some(rust_type_map!["key" => rust_type_map!["inner_key" => rust_type_vec![1, "2"]]]))]
    #[test_case("/key", &Some(rust_type_map!["inner_key" => rust_type_vec![1, "2"]]))]
    #[test_case("/key/inner_key", &Some(rust_type_vec![1,"2"]))]
    #[test_case("/key/inner_key/0", &Some(RustType::from(1)))]
    #[test_case("/key/inner_key/1", &Some(RustType::from("2")))]
    #[test_case("/not_present", &None)]
    #[test_case("/key/inner_key/a", &None)]
    #[test_case("/key/inner_key/2", &None)]
    fn test_get_fragment(fragment: &str, expected_value: &Option<RustType>) {
        let external_map = rust_type_map![
            "key" => rust_type_map![
                "inner_key" => rust_type_vec![
                    1,
                    "2"
                ],
            ],
        ];
        assert_eq!(get_fragment(&external_map, fragment), expected_value.as_ref());
    }
}
