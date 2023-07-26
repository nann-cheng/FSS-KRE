use super::RingElm;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use std::fmt;
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
pub struct BeaverTuple{
    pub a: RingElm,
    pub b: RingElm,
    pub ab: RingElm
}

impl BeaverTuple{
    fn new(ra: RingElm, rb: RingElm, rc: RingElm) -> Self{
        BeaverTuple { a: ra, b: rb, ab: rc}
    }
}

impl Serialize for BeaverTuple {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("beavertuple", 3)?;
        state.serialize_field("r", &self.a)?;
        state.serialize_field("g", &self.b)?;
        state.serialize_field("b", &self.ab)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for BeaverTuple {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        enum Field { A, B, C }
        //type Field = RingElm;
        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`a` or `b` or `c`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where E: de::Error,
                    {
                        match value {
                            "a" => Ok(Field::A),
                            "b" => Ok(Field::B),
                            "c" => Ok(Field::C),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct BeaverVisitor;

        impl<'de> Visitor<'de> for BeaverVisitor {
            type Value = BeaverTuple;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BeaverTuple")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<BeaverTuple, V::Error>
            where V: SeqAccess<'de>,
            {
                let a = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let b = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let c = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(BeaverTuple::new(a, b, c))
            }

            fn visit_map<V>(self, mut map: V) -> Result<BeaverTuple, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut a = None;
                let mut b = None;
                let mut c = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::A => {
                            if a.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            a = Some(map.next_value()?);
                        }
                        Field::B => {
                            if b.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            b = Some(map.next_value()?);
                        }
                        Field::C => {
                            if c.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            c = Some(map.next_value()?);
                        }
                    }
                }
                let a = a.ok_or_else(|| de::Error::missing_field("a"))?;
                let b = b.ok_or_else(|| de::Error::missing_field("b"))?;
                let c = c.ok_or_else(|| de::Error::missing_field("c"))?;
                Ok(BeaverTuple::new(a, b, c))
            }
        }

        const FIELDS: &'static [&'static str] = &["a", "b", "c"];
        deserializer.deserialize_struct("BeaverTuple", FIELDS, BeaverVisitor)
    }
}