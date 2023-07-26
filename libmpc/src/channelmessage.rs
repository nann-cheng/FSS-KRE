use std::error::Error;
use std::fmt;
use idpf::{RingElm, Group};
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelMessage{
    BoolData(bool),
    RingData(RingElm),
    BoolVec(Vec<bool>),
    RingVec(Vec<RingElm>)
}
impl fmt::Display for ChannelMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelMessage::BoolData(b) => write!(f, "{}", b),
            ChannelMessage::RingData(value) => write!(f, "{}", value.to_u32().unwrap()),
            ChannelMessage::BoolVec(v) => write!(f, "v is a vector of bool {}", v.len()),
            ChannelMessage::RingVec(v) => write!(f, "v is a vector of RingElm {}", v.len())
        }
    }
}
impl ChannelMessage{
    pub fn to_bytes(&self) -> Vec<u8>{
        match self{
            ChannelMessage::BoolData(false) => {vec![0]},
            ChannelMessage::BoolData(true) => {vec![1]},
            ChannelMessage::RingData(v) => {v.to_u32().unwrap().to_be_bytes().to_vec()},
            ChannelMessage::BoolVec(v) => {let t = v.iter().map(|x| if *x==true{1u8} else {0u8}).collect(); t}
            ChannelMessage::RingVec(v) => {
                let mut t =  Vec::<u8>::new(); 
                for e in v{
                    t.append(&mut e.to_u32().unwrap().to_be_bytes().to_vec());
                }
                t
            }
        }
    }
    pub fn to_bool_type(v: Vec<u8>) ->Result<Self, Box<dyn Error>>{
        if v.len() != 1 || v[0] != 0 && v[0] != 1{
            Err("Channel Message Convert Error.".into())
        }
        else{
            Ok(ChannelMessage::BoolData(v[0] == 1))
        }
    }

    pub fn to_ring_type(v: Vec<u8>) ->Result<Self, Box<dyn Error>>{
        if v.len() != std::mem::size_of::<RingElm>(){
            Err("Channel Message Convert Error.".into())
        }
        else{
            let mut buf: [u8; 4]= [0; 4];
            for i in 0..4{
                buf[i] = v[i]
            }
            let e = RingElm::from(u32::from_be_bytes(buf));
            Ok(ChannelMessage::RingData(e))
        }
    }

    pub fn to_boolvec_type(v: Vec<u8>) ->Result<Self, Box<dyn Error>>{
        let mut buf = Vec::<bool>::new();
        for e in v{
            if e != 0 && e != 1{
                return Err("Channel Message Convert Error.".into());
            }
            buf.push(e == 1)
        }
        Ok(ChannelMessage::BoolVec(buf))
    }

    pub fn to_ringvec_type(v: Vec<u8>) -> Result<Self, Box<dyn Error>>{
        if v.len() % std::mem::size_of::<RingElm>() != 0{
            Err("Channel Message Convert Error.".into())
        }
        else{
            let mut r = Vec::<RingElm>::new();
            for i in 0..v.len()/4{
                let mut buf: [u8; 4]= [0; 4];
                for j in 0..4{
                    buf[j] = v[i*4+j];
                }
                let e = RingElm::from(u32::from_be_bytes(buf));
                r.push(e);
            }
            Ok(ChannelMessage::RingVec(r))
        }
    }
}

pub fn f_reconstrct(msg1: &ChannelMessage, msg2: &ChannelMessage) -> Result<ChannelMessage, Box<dyn Error>>{
    match (msg1, msg2){
        (ChannelMessage::BoolData(e1), ChannelMessage::BoolData(e2)) =>{    
            let e = *e1 as u8 ^ *e2 as u8;
            Ok(ChannelMessage::BoolData(e == 1))
        },
        (ChannelMessage::RingData(e1), ChannelMessage::RingData(e2)) =>{
            let mut e = RingElm::zero();
            e.add(&e1);
            e.add(&e2);
            Ok(ChannelMessage::RingData(e))
        },
        (ChannelMessage::BoolVec(e1), ChannelMessage::BoolVec(e2)) =>{
            if e1.len() != e2.len(){
                Err("The bool vectors' lengthes are not same ".into())
            }
            else{
                let v_len = e1.len();
                let mut v = Vec::<bool>::new();
                for i in 0..v_len{
                    let u1 = e1[i];
                    let u2 = e2[i];
                    let u = u1 as u8 ^ u2 as u8;
                    v.push(u == 1);
                }
                Ok(ChannelMessage::BoolVec(v))
            }
        }
        (ChannelMessage::RingVec(e1), ChannelMessage::RingVec(e2)) =>{
            if e1.len() != e2.len(){
                Err("The bool vectors' lengthes are not same ".into())
            }
            else{
                let v_len = e1.len();
                let mut v = Vec::<RingElm>::new();
                for i in 0..v_len{
                    let mut u = RingElm::from(0);
                    u.add(&e1[i]);
                    u.add(&e2[i]);
                    v.push(u);
                }
                Ok(ChannelMessage::RingVec(v))
            }
        }
        _ =>{
            Err("Mismatched message type for channel message".into())
        }
    }
}
