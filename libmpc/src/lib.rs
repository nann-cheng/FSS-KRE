pub mod channelmessage;
pub mod mpc_party;
pub mod mpc_platform;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use futures::join;
    use idpf::prg::*;
    use idpf::RingElm;   
    use crate::channelmessage::*;    
    use crate::mpc_party::*;
    #[tokio::test]
    async fn reconstruction_works(){
        let b1 = ChannelMessage::BoolData(true);
        let b2 = ChannelMessage::BoolData(true);
        let b = f_reconstrct(&b1, &b2);
        assert_eq!(b.unwrap(), ChannelMessage::BoolData(false));
        let r1 = ChannelMessage::RingData(RingElm::from(33));
        let r2 = ChannelMessage::RingData(RingElm::from(54));
        let r = f_reconstrct(&r1, &r2);
        assert_eq!(r.unwrap(), ChannelMessage::RingData(RingElm::from(87)));
        let mut v1 = Vec::<bool>::new();
        v1.push(true); v1.push(true); v1.push(false); v1.push(false); 
        let mut v2 = Vec::<bool>::new();
        v2.push(true); v2.push(false); v2.push(false); v2.push(true); 
        let vb1 = ChannelMessage::BoolVec(v1);
        let vb2 = ChannelMessage::BoolVec(v2);

        let vb = f_reconstrct(&vb1, &vb2);
        
        //assert_eq!(vb.unwrap().to_bytes(), [false, true, false, true])
        let buf = vb.unwrap().to_bytes();
        assert_eq!(buf.len(), 4);
        /*assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 1);
        assert_eq!(buf[2], 0);
        assert_eq!(buf[3], 1);*/
        let mut rv1 = Vec::<RingElm>::new();
        let mut rv2: Vec<RingElm> = Vec::<RingElm>::new();
        //let mut rv: Vec<RingElm> = Vec::<RingElm>::new();

        for i in 0..10{
            let e1 = RingElm::from(i);
            let e2: RingElm = RingElm::from(1234-i);
            rv1.push(e1);
            rv2.push(e2);
        }

        let rv = f_reconstrct(&ChannelMessage::RingVec(rv1), &&ChannelMessage::RingVec(rv2)).unwrap();
        match rv{
            ChannelMessage::RingVec(real_rv) =>{
                assert_eq!(real_rv.len(), 10);
                for i in 0..10{
                    assert_eq!(real_rv[i].to_u32().unwrap(), 1234);
                }
            }
            _ =>{panic!("ChannelMessage::RingVec reconstruction error!");}
        }
    }

    #[tokio::test]
    async fn test_work_of_channel(){
        let mut v = Vec::<bool>::new();
        v.push(true); v.push(true); v.push(false); v.push(false);
        let vc = v.clone();  
        let vb = ChannelMessage::BoolVec(v);
        //let mut buf= [0; 10];
        let (msg_tx, mut msg_rx) = mpsc::channel::<Vec<u8>>(100);
        let _= msg_tx.send(vb.to_bytes()).await;
        let t = msg_rx.recv().await.unwrap();

        for i in 0..4{
            assert_eq!(vc[i] as u8, t[i]);
        }

        let mut rv1 = Vec::<RingElm>::new();
        for i in 0..10{
            let e: RingElm = RingElm::from(1234-i);
            rv1.push(e);
        }

        let msg1 = ChannelMessage::RingVec(rv1);
        let _= msg_tx.send(msg1.to_bytes()).await;
        let msg2 = msg_rx.recv().await.unwrap();
        assert_eq!(msg2.len(), 40);
        println!("msg2={:?}", msg2);
        let rv2 = ChannelMessage::to_ringvec_type(msg2).unwrap();
        println!("rv2={:?}", rv2);
        match rv2{
            ChannelMessage::RingVec(real_rv) =>{
                assert_eq!(real_rv.len(), 10);
                for i in 0..10{
                    assert_eq!(real_rv[i].to_u32().unwrap(), (1234-i) as u32);
                }
            }
            _ =>{panic!("ChannelMessage::RingVec reconstruction error!");}
        }
    }
}
