use crate::prg::PrgSeed;
use crate::prg::FixedKeyPrgStream;
use crate::bits_to_u32;

use crate::{ring, Group};

use super::RingElm;
// use serde::ser::{Serialize, Serializer, SerializeStruct};
// use std::fmt;
use serde::Deserialize;
use serde::Serialize;

const NUMERIC_LEN:usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeaverTuple{
    pub a: RingElm,
    pub b: RingElm,
    pub ab: RingElm,
    pub delta_a: RingElm,
    pub delta_b: RingElm,
}

impl BeaverTuple{
    fn new(ra: RingElm, rb: RingElm, rc: RingElm) -> Self{
        BeaverTuple { a: ra, b: rb, ab: rc, delta_a:RingElm::zero(), delta_b:RingElm::zero(), }
    }

    pub fn genBeaver(beavertuples0: &mut Vec<BeaverTuple>, beavertuples1: &mut Vec<BeaverTuple>, seed: &PrgSeed, size:usize) {
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        for i in 0..size{
            let rd_bits = stream.next_bits(NUMERIC_LEN*5);
            let a0 = RingElm::from( bits_to_u32(&rd_bits[..NUMERIC_LEN]) );
            let b0 = RingElm::from( bits_to_u32(&rd_bits[NUMERIC_LEN..2*NUMERIC_LEN]) );

            let a1 = RingElm::from( bits_to_u32(&rd_bits[2*NUMERIC_LEN..3*NUMERIC_LEN]) );
            let b1 = RingElm::from( bits_to_u32(&rd_bits[3*NUMERIC_LEN..4*NUMERIC_LEN]));

            let ab0 = RingElm::from( bits_to_u32(&rd_bits[4*NUMERIC_LEN..5*NUMERIC_LEN]) );

            let mut a = RingElm::zero();
            a.add(&a0);
            a.add(&a1);

            let mut b = RingElm::zero();
            b.add(&b0);
            b.add(&b1);

            let mut ab = RingElm::one();
            ab.mul(&a);
            ab.mul(&b);

            ab.sub(&ab0);

            let beaver0 = BeaverTuple{
                a: a0,
                b: b0,
                ab: ab0,
                delta_a:RingElm::zero(),
                delta_b:RingElm::zero(),
            };

            let beaver1 = BeaverTuple{
                a: a1,
                b: b1,
                ab: ab,
                delta_a:RingElm::zero(),
                delta_b:RingElm::zero(),
            };
            beavertuples0.push(beaver0);
            beavertuples1.push(beaver1);
            
        }
    }
    
    pub fn beaver_mul0(&mut self, alpha: RingElm, beta: RingElm)-> Vec<u8>{
        self.delta_a = alpha - self.a;
        self.delta_b = beta - self.b;

        let mut container  = Vec::<u8>::new();
        container.append(&mut self.delta_a.to_u32().unwrap().to_be_bytes().to_vec());
        container.append(&mut self.delta_b.to_u32().unwrap().to_be_bytes().to_vec());
        container
    }

    /*The multiplication of [alpha] x [beta], the values of beaver_share are [a], [b], and [ab], d and e are the reconstructed values of alpha-a, beta-b*/
    pub fn beaver_mul1(&mut self, is_server: bool, otherHalf:&Vec<u8> ) -> RingElm{
        assert_eq!(otherHalf.len(),8usize);
        for i in 0..2{
            let mut ybuf: [u8; 4]= [0; 4];
            for j in 0..4{
                ybuf[j] = otherHalf[i*4+j];
            }
            if i==0{
                self.delta_a.add(&RingElm::from(u32::from_be_bytes(ybuf)));
            }
            else{
                self.delta_b.add(&RingElm::from(u32::from_be_bytes(ybuf)));
            }
        }
        let mut result= RingElm::zero();
        if is_server{
            result.add(&(self.delta_a*self.delta_b) );
        }
        result.add(&(self.delta_a*self.b) );
        result.add(&(self.delta_b*self.a) );
        result.add(& self.ab);
        result
    }

}


// /*The multiplication of [alpha] x [beta], the values of beaver_share are [a], [b], and [ab], d and e are the reconstructed values of alpha-a, beta-b*/
// fn beaver_mul(is_server: bool, beaver_share: &BeaverTuple, d: &RingElm, e: &RingElm) -> RingElm{
    
//     let mut r;
//     if is_server{
//         r = beaver_share.ab.clone();
//     }
//     else{
//         r = RingElm::zero();
//     }

//     let mut r0 = d.clone();
//     r0.mul(&e); //d*e

//     let mut r1 = d.clone();
//     r1.mul(&beaver_share.b); //d*[b]

//     let mut r2 = e.clone();
//     r2.mul(&beaver_share.a); //e*[a]
    
//     r.add(&r0);
//     r.add(&r1);
//     r.add(&r2);

//     r
// }

// impl Serialize for BeaverTuple {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("beavertuple", 3)?;
//         state.serialize_field("r", &self.a)?;
//         state.serialize_field("g", &self.b)?;
//         state.serialize_field("b", &self.ab)?;
//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for BeaverTuple {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where D: Deserializer<'de>,
//     {
//         enum Field { A, B, C }
//         //type Field = RingElm;
//         // This part could also be generated independently by:
//         //
//         //    #[derive(Deserialize)]
//         //    #[serde(field_identifier, rename_all = "lowercase")]
//         //    enum Field { Secs, Nanos }
//         impl<'de> Deserialize<'de> for Field {
//             fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//             where D: Deserializer<'de>,
//             {
//                 struct FieldVisitor;

//                 impl<'de> Visitor<'de> for FieldVisitor {
//                     type Value = Field;

//                     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                         formatter.write_str("`a` or `b` or `c`")
//                     }

//                     fn visit_str<E>(self, value: &str) -> Result<Field, E>
//                     where E: de::Error,
//                     {
//                         match value {
//                             "a" => Ok(Field::A),
//                             "b" => Ok(Field::B),
//                             "c" => Ok(Field::C),
//                             _ => Err(de::Error::unknown_field(value, FIELDS)),
//                         }
//                     }
//                 }

//                 deserializer.deserialize_identifier(FieldVisitor)
//             }
//         }

//         struct BeaverVisitor;

//         impl<'de> Visitor<'de> for BeaverVisitor {
//             type Value = BeaverTuple;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("struct BeaverTuple")
//             }

//             fn visit_seq<V>(self, mut seq: V) -> Result<BeaverTuple, V::Error>
//             where V: SeqAccess<'de>,
//             {
//                 let a = seq.next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(0, &self))?;
//                 let b = seq.next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(1, &self))?;
//                 let c = seq.next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(2, &self))?;
//                 Ok(BeaverTuple::new(a, b, c))
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<BeaverTuple, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut a = None;
//                 let mut b = None;
//                 let mut c = None;
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::A => {
//                             if a.is_some() {
//                                 return Err(de::Error::duplicate_field("secs"));
//                             }
//                             a = Some(map.next_value()?);
//                         }
//                         Field::B => {
//                             if b.is_some() {
//                                 return Err(de::Error::duplicate_field("nanos"));
//                             }
//                             b = Some(map.next_value()?);
//                         }
//                         Field::C => {
//                             if c.is_some() {
//                                 return Err(de::Error::duplicate_field("nanos"));
//                             }
//                             c = Some(map.next_value()?);
//                         }
//                     }
//                 }
//                 let a = a.ok_or_else(|| de::Error::missing_field("a"))?;
//                 let b = b.ok_or_else(|| de::Error::missing_field("b"))?;
//                 let c = c.ok_or_else(|| de::Error::missing_field("c"))?;
//                 Ok(BeaverTuple::new(a, b, c))
//             }
//         }

//         const FIELDS: &'static [&'static str] = &["a", "b", "c"];
//         deserializer.deserialize_struct("BeaverTuple", FIELDS, BeaverVisitor)
//     }
// }


