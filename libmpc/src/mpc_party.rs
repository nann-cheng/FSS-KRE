use super::channelmessage::*;
use idpf::*;
use idpf::beavertuple::BeaverTuple;
use idpf::dpf::*;
use idpf::RingElm;
use std::path::PathBuf;
use futures::executor::block_on;
use tokio::sync::mpsc;
use bincode::Error;
use std::fs::File;
use std::io::prelude::*;

use super::channelmessage::f_reconstrct;
pub struct FileConfig<'a>{
    pub dir_path: &'a str,
    pub k_file: &'a str,
    pub a_file: &'a str,  //alpha
    pub qa_file: &'a str, //q arithmetical share
    pub qb_file: &'a str, //q bool share
    pub zc_k_file: &'a str,
    pub zc_a_file: &'a str,
    pub beavers_file: &'a str
} // This struct is configuare the file where the offline data is stored. 

pub struct OfflineInfomation{
    k_share: Vec<DPFKey<RingElm>>, //dpf
    a_share: Vec<bool>,  //alpha
    qa_share: Vec<RingElm>, //q arithmetical share
    qb_share: Vec<bool>, //q bool share
    zc_k_share: Vec<DPFKey<RingElm>>,
    zc_a_share: Vec<RingElm>,
    beavers: Vec<BeaverTuple>
}

impl OfflineInfomation{
    pub fn new() -> Self{
        Self{k_share: Vec::new(), a_share: Vec::new(), qa_share: Vec::new(), qb_share: Vec::new(), zc_k_share: Vec::new(), zc_a_share: Vec::new(), beavers: Vec::new()}
    }

    pub fn setup(&mut self, files: &FileConfig)
    {
        let mut path = PathBuf::from(files.dir_path);
        path.push(files.a_file);
        self.ReadAlphaShare(&path);
        path.pop();
        
        path.push(files.k_file);
        self.ReadKeyShare(&path);
        path.pop();

        path.push(files.qa_file);
        self.ReadQAShare(&path);
        path.pop();

        path.push(files.qb_file);
        self.ReadQBShare(&path);
        path.pop();

        path.push(files.zc_a_file);
        self.ReadZeroCheckAShare(&path);
        path.pop();

        path.push(files.zc_k_file);
        self.ReadZeroCheckKeyShare(&path);
        path.pop();

        path.push(files.beavers_file);
        self.ReadBeavers(&path);
        path.pop();
    }

    fn WriteKeyShare(&self, file: &PathBuf) ->Result<bool, Error> {
        let mut f = File::create(file).expect("create failed");
        f.write_all(&bincode::serialize(&self.k_share).expect("Serialize key error")).expect("Write key error.");
        Ok(true)
    }

    fn ReadKeyShare(&mut self, file: &PathBuf) ->Result<bool, Error> {
        self.k_share.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.k_share = bincode::deserialize(&buf).expect("Deserialize key-share Error");
        Ok(true)
    }

    fn WriteAlphaShare(&self, file: &PathBuf) ->Result<bool, Error> {
        let mut f = File::create(file).expect("create failed");
        f.write_all(&bincode::serialize(&self.a_share).expect("Serialize alpha error")).expect("Write alpha error.");
        Ok(true)
    }

    fn ReadAlphaShare(&mut self, file: &PathBuf) ->Result<bool, Error> {
        self.a_share.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.a_share = bincode::deserialize(&buf).expect("Deserialize key-share Error");
        Ok(true)
    }
    fn ReadQAShare(&mut self, file: &PathBuf) ->Result<bool, Error> {
        //self.qa_share.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.qa_share = bincode::deserialize(&buf).expect("Deserialize qa-share Error");
        Ok(true)
    }

    fn WriteQAShare(&self, file: &PathBuf) ->Result<bool, Error> {
        let mut f = File::create(file).expect("create failed");
        f.write_all(&bincode::serialize(&self.qa_share).expect("Serialize alpha error")).expect("Write alpha error.");
        Ok(true)
    }

    fn ReadQBShare(&mut self, file: &PathBuf) ->Result<bool, Error> {
        self.qb_share.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.qb_share = bincode::deserialize(&buf).expect("Deserialize key-share Error");
        Ok(true)
    }

    fn WriteQBShare(&self, file: &PathBuf) ->Result<bool, Error> {
        let mut f = File::create(file).expect("create failed");
        f.write_all(&bincode::serialize(&self.qb_share).expect("Serialize alpha error")).expect("Write alpha error.");
        Ok(true)
    }
    
    fn ReadZeroCheckAShare(&mut self, file: &PathBuf) ->Result<bool, Error> {
        self.zc_a_share.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.zc_a_share = bincode::deserialize(&buf).expect("Deserialize key-share Error");
        Ok(true)
    }

    fn ReadZeroCheckKeyShare(&mut self, file: &PathBuf) ->Result<bool, Error> {
        self.zc_k_share.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.zc_k_share = bincode::deserialize(&buf).expect("Deserialize key-share Error");
        Ok(true)
    }

    fn ReadBeavers(&mut self, file: &PathBuf) ->Result<bool, Error> {
        self.beavers.clear();
        let mut f = File::open(file).expect("Open file failed");
        let mut buf = Vec::<u8>::new();
        f.read_to_end(&mut buf).expect("Read file error!");
        self.beavers = bincode::deserialize(&buf).expect("Deserialize key-share Error");
        Ok(true)
    }
}

pub enum PartyRole{
    Active,
    Passitive
}
pub struct MPCParty{
    offlinedata: OfflineInfomation,
    //x_share: Vec<bool>,
    m: usize, //The number of share numbers
    n: usize, //The length of a shared element
    role: PartyRole
}

impl MPCParty{
    pub fn new(data: OfflineInfomation, m_role: PartyRole)->Self{
        MPCParty { offlinedata: data, m: 0, n: 0, role: m_role}
    }
    pub fn setup(&mut self, files: &FileConfig, input_size: usize, input_bits: usize){
        self.offlinedata.setup(files);
        self.m = input_size;
        self.n = input_bits;
    }
} 

/*async fn exchange_message(msg_ty: &mpsc::Sender<Vec<u8>>, msg_rx: &mut mpsc::Receiver<Vec<u8>>, msg: &Vec<u8>)->Vec<u8>  
{
    let t = msg.clone();
    //println!("Call exchange_message. send length {}", t.len());
    let f1 = msg_ty.send(t);
    let f2 = msg_rx.recv();
    //let msg_recv = f2.unwrap();
    //println!("Call exchange_message. recv length {}", msg_recv.len());
    //tokio::excutor::block_on(f1, f2);
    let (_, o_share) = futures::join!(f1, f2);
    o_share.unwrap()
    //msg_recv
}*/

pub async fn max(p: &MPCParty, x_bits: &Vec<bool>, msg_ty: &mpsc::Sender<Vec<u8>>, msg_rx: &mut mpsc::Receiver<Vec<u8>>)->Vec<bool>{
    println!("Start!");
    let m = p.m;
    let n = p.n;
    println!("m={}, n={}", m, n);
    let mut mask_bits = Vec::<bool>::new(); //t in the paper, it is a bit vector of length n
    //let mut prefix_bits = vec![false;m*n]; // m bit vector whose length is n
    let mut cmp_bits = vec![false;n]; // the current prefix that has been checked
    
    let mut old_state = Vec::<EvalState>::new();
    let mut new_state = Vec::<EvalState>::new();

    /*Line 2-5: This step compute n mask bits that will be used as the input of IDPF*/
    for i in 0..m{
        let init_state = p.offlinedata.k_share[i].eval_init();
        old_state.push(init_state.clone()); // Line2
        new_state.push(init_state.clone()); 
        for j in 0..n{
            let t_share = x_bits[i*n + j] ^ p.offlinedata.a_share[i*n + j] ^ p.offlinedata.qb_share[i]; //x[i][j]
            mask_bits.push(t_share);
        }        
    }
    println!("Step2!");
    /*Line 3: The reveal function for a bunch of bool data*/ 
    let t = {
        let vc_0:Vec<u8> = mask_bits.iter().map(|x| if *x == true {1} else {0}).collect(); // convert the bool vec to u8 vec such that the message can be convoyed in the channel
        println!("Start Exchange1");

        let f1 = msg_ty.send(vc_0.clone()).await;
        let f2 = msg_rx.recv().await;
        let vc_1 = f2.unwrap();
        //let vc_1 = block_on(exchange_message(msg_ty, msg_rx, &vc_0));
        //let vc_1 = exchange_message(msg_ty, msg_rx, &vc_0).await;
        println!("End Exchange1");
        let share0 = ChannelMessage::to_boolvec_type(vc_0).unwrap();
        let share1 = ChannelMessage::to_boolvec_type(vc_1).unwrap();
        let r = f_reconstrct(&share0, &share1);
        if let Ok(ChannelMessage::BoolVec(vc)) = r{
            vc
        }
        else{
            Vec::<bool>::new()
        }
     };
     println!("Step3: t.len = {}", t.len());
    /*Line5-6: v is the number of elements whose prefix is p_{i-1} */
    let mut v_share; 
    match p.role{
        PartyRole::Active => v_share = RingElm::from(m as u32),
        _ => v_share = RingElm::from(0)
    }
    let mut omega_share = {
        let mut ring_unit = RingElm::from(1);
        let ring_m = RingElm::from(m as u32);
        
        ring_unit.sub(&p.offlinedata.qa_share[0]);
        ring_unit.mul(&ring_m);
        ring_unit   
    }; // Line6
    
    let mut beavers = p.offlinedata.beavers.iter();
    
    //Online-step-3. Start bit-by-bit prefix query, from Line7
    for i in 0..n{
        println!("***************start the {} iteration***************", i);
        let mut mu_share = RingElm::zero();
        for j in 0..m{
            let new_bit = t[j*n+i]; //x[j][i]
            let (state_new, beta) = p.offlinedata.k_share[j].eval_bit(&old_state[j], new_bit); //compute the IDPF value at t[j][i]
            mu_share.add(&beta);
            new_state[j] = state_new; 
        }
        /*mu is the number of elements having the prefix p_{i-1} || q[i] */
        
        let v0_share = mu_share.clone(); //Line 13, the number of elements having the prerix p_{i-1} || q[i]
        let mut v1_share = v_share.clone();
        v1_share.sub(&mu_share); // Line 14, the number of elements having prefix p_{i-1} || ~q[i]
        let v_share_t = (v0_share.clone(), v1_share.clone());
        /*Exchange five ring_elements in parallel: u_i-w_i-alpha[i], (d_share, e_share) tuples for the two multiplication operation */
        let mut msg1 = Vec::<RingElm>::new(); // the send message
        let mut x_fnzc_share = RingElm::from(0);  //
        x_fnzc_share.add(&mu_share);
        x_fnzc_share.sub(&omega_share); //compute u_i-w_i, the x value of f_{NonZeroCheck}
        x_fnzc_share.add(&p.offlinedata.zc_a_share[i]); //mask the x value by alpha 
        msg1.push(x_fnzc_share);

        /*Obtain two beaver tuples and assure the beaver tuples are existing*/
        let beaver1 = beavers.next();
        let beaver2 = beavers.next();
        if beaver1.is_none() || beaver2.is_none(){
            eprintln!("Beaver tuples are not enough.");
            std::process::exit(-1);
        }
        let bt1 = beaver1.unwrap();
        let bt2 = beaver2.unwrap();
        let mut d1_share= v0_share.clone(); //the fisrt v_alpha = v0_share 
        let mut d2_share= v1_share.clone(); //the second v_alpha = v0_share 
        let mut e1_share = RingElm::from(1); //the fisrt v_beta = 1-q[i+1] 
        let mut e2_share = RingElm::from(1); //the second v_beta = 1-q[i+1]
        if i < n-1{
            /* local comupute v_alpha-a for the two multiple operatios */
            d1_share.sub(&bt1.a);
            d2_share.sub(&bt2.a);

            /* local comupute v_beta-b for the two multiple operatios */
            e1_share.sub(&p.offlinedata.qa_share[i+1]);
            e2_share.sub(&p.offlinedata.qa_share[i+1]);
            e1_share.sub(&bt1.b);
            e2_share.sub(&bt2.b);

            msg1.push(d1_share);
            msg1.push(d2_share);
            msg1.push(e1_share);
            msg1.push(e2_share);
        }
        
        /*open d and e for the two multiple in parallel */
        /* First encode the four ring elements to a binary stream, then decode the received binary strean to four ring elements */
        let x_msg1 = ChannelMessage::RingVec(msg1);
        let buf1 = x_msg1.to_bytes();
        //let buf2 = block_on(exchange_message(msg_ty, msg_rx, &buf1));
        println!("Start Exchange ring elememts {}", i);
        let _ = msg_ty.send(buf1.clone()).await;
        let buf2 = msg_rx.recv().await.unwrap();
        let x_msg2 = ChannelMessage::to_ringvec_type(buf2).unwrap();
        let x_msg = f_reconstrct(&x_msg1, &x_msg2).unwrap();
        println!("End Exchange ring elememts {}", i);
        let (x_fznc, d1, d2, e1, e2);
        let omega_t;
        println!("Step 4-1-{}", i);
        if i < n-1{ //Decode x_msg to 5 ring elements 
            match x_msg{
                ChannelMessage::RingVec(rv) => {
                    if rv.len() != 5{
                        eprintln!("Decode error!");
                        std::process::exit(-1);
                    }
                    else{
                        x_fznc = rv[0].clone();
                        d1 = rv[1].clone();
                        d2 = rv[2].clone();
                        e1 = rv[3].clone();
                        e2 = rv[4].clone();
                    }
                }
                _ =>{
                    eprintln!("Decode error!");
                    std::process::exit(-1);
                }
            }
            //Now, x_fnzc, d1, d2, e1, e2 have been reconstructed and decoded
            //Line 15-18: Calculate omega0 and omega1 
           
            let mut omega0 = bt1.ab.clone();
            let mut omega0_1;
            match p.role{
                PartyRole::Active => {omega0_1 = RingElm::from(0)},
                PartyRole::Passitive=>{omega0_1 = d1.clone(); omega0_1.mul(&e1);}
            }
            let mut omega0_2 = d1.clone();
            omega0_2.mul(&bt1.b);
            let mut omega0_3 = e1.clone();
            omega0_3.mul(&bt1.a);
            omega0.add(&omega0_1);
            omega0.add(&omega0_2);
            omega0.add(&omega0_3);

            let mut omega1 = bt2.ab.clone();
            let mut omega1_1;
            match p.role{
                PartyRole::Active => {omega1_1 = RingElm::from(0)},
                PartyRole::Passitive=>{omega1_1 = d1.clone(); omega1_1.mul(&e1);}
            }
            let mut omega1_2 = d2.clone();
            omega1_2.mul(&bt2.b);
            let mut omega1_3 = e2.clone();
            omega1_3.mul(&bt2.a);
            omega1.add(&omega1_1);
            omega1.add(&omega1_2);
            omega1.add(&omega1_3);
            
            omega_t = (omega0, omega1);  
        } //end if i < n
        else{
            match x_msg{
                ChannelMessage::RingVec(rv) => {
                    if rv.len() != 1{
                        eprintln!("Decode error!");
                        std::process::exit(-1);
                    }
                    else{
                        x_fznc = rv[0].clone();
                        //d1 = rv[1];
                        //d2 = rv[2];
                        //e1 = rv[3];
                        //e2 = rv[4];
                    }
                }
                _ =>{
                    eprintln!("Decode error!");
                    std::process::exit(-1);
                }
            }
            omega_t = (RingElm::from(0), RingElm::from(0));    
       } //end else if i < n-1
       println!("Step 4-2-{}", i);
        //start Line 12, calculate the f_{NonZeroCheck}(x_fnzc)
        let mut vec_eval = vec![false;NUMERIC_LEN];

        let num_eval = x_fznc.to_u32();
        match num_eval {
            Some(numeric) => vec_eval = u32_to_bits(NUMERIC_LEN,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }
        let y_fnzc = p.offlinedata.zc_k_share[i].eval(&vec_eval);
        cmp_bits[i] = y_fnzc.to_u32().unwrap() & 0x1u32 == 0x1u32; //mod 2, convert RingElem to u32, then to bool
        //end Line 12 

        /*Line 19 */
        let simga_share = cmp_bits[i] ^ p.offlinedata.qb_share[i];
        let msg1 = ChannelMessage::BoolData(simga_share);
        let buf1 = msg1.to_bytes();
        println!("Start Reveal sigma {}, need exchange 1 bool value", i);
        let _ = msg_ty.send(buf1.clone()).await;
        let buf2 = msg_rx.recv().await.unwrap();
        
        //let buf2 = block_on(exchange_message(msg_ty, msg_rx, &buf1));
        let msg2 = ChannelMessage::to_bool_type(buf2).unwrap();
        let r_msg = f_reconstrct(&msg1, &msg2).unwrap();
        let sigma = match r_msg{
            ChannelMessage::BoolData(b) => {b}
            _ =>{false}
        };
        println!("End Reveal sigma {}", i);
        println!("Step 4-3-{}", i);
        println!("sigma_{}={}", i, sigma);
        /*Line 20-21 */
        if sigma == true{
            v_share = v_share_t.1;
            omega_share = omega_t.1;
        }
        else {
            v_share = v_share_t.0;
            omega_share = omega_t.0;
        }

         /*Line 22 update the m idpf if sigma == 1, it means a wrong q[i] is choosed*/
        if sigma == true{
            for j in 0..m{
                let eval_bit = !t[j*m+i]; //choose the oppsite value x[j][i]
                let (state_new, _) = p.offlinedata.k_share[j].eval_bit(&old_state[j], eval_bit);
                new_state[j] = state_new;
            }
        }
        old_state = new_state.clone(); //update the state
        println!("***************end the {} iteration***************", i);
    }
    cmp_bits
       
}




