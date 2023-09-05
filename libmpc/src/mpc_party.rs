use fss::*;
use fss::beavertuple::BeaverTuple;
use fss::idpf::*;
use fss::dpf::*;
use fss::RingElm;
use fss::BinElm;
use crate::mpc_platform::*;
use crate::offline_data::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

// enum OfflineDataType {
//     BitwiseMax,
//     BatchwiseMax,
//     BitwiseKre,
//     BatchwiseKre,
// }

// fn main() {
//     let current_direction = Direction::North;

//     match current_direction {
//         Direction::North => println!("We are heading north!"),
//         Direction::South => println!("We are heading south!"),
//         Direction::East => println!("We are heading east!"),
//         Direction::West => println!("We are heading west!"),
//     }
// }

// trait OfflineTrait {BitMaxOffline,BitKreOffline};
pub struct MPCParty<T>{
    // offlinedata: BitMaxOffline,
    pub offlinedata: T,
    pub m: usize, //The number of share numbers
    pub n: usize, //The length of a shared element
    pub netlayer: NetInterface
}

impl<T>  MPCParty<T>{
    pub fn new(data: T, netinterface: NetInterface)->Self{
        MPCParty { offlinedata: data, m: 0, n: 0, netlayer: netinterface}
    }
    pub fn setup(&mut self, input_size: usize, input_bits: usize){
        self.m = input_size;
        self.n = input_bits;
    }
}

pub async fn bitwise_max(p: &mut MPCParty<BitMaxOffline>, x_bits: &Vec<bool>)->Vec<bool>{
    let m: usize = p.m;
    let n = p.n;

    let is_server = p.netlayer.is_server;
    let mut mask_bits = Vec::<bool>::new();//t in the paper, it is a bit vector of length n
    let mut cmp_bits = vec![false;n]; // the current prefix that has been checked
    let mut old_state = Vec::<EvalState>::new();
    let mut new_state = Arc::new(Mutex::new(Vec::<EvalState>::new()));

    /*Line 2-5: This step compute n mask bits that will be used as the input of IDPF*/
    for i in 0..m{
        let init_state = p.offlinedata.base.k_share[i].eval_init();
        old_state.push(init_state.clone()); // Line2
        new_state.lock().unwrap().push(init_state.clone()); 
        for j in 0..n{
            let t_share = x_bits[i*n + j] ^ p.offlinedata.base.a_share[i*n + j] ^ p.offlinedata.base.qb_share[j]; //x[i][j]^qb[j]
            mask_bits.push(t_share);
        }
    }
   
    /*Line 3: The reveal function for a bunch of bool data*/ 
    let t = p.netlayer.exchange_bool_vec(mask_bits.clone()).await; 
    /*Line5-6: v is the number of elements whose prefix is p_{i-1} */
    let mut v_share= RingElm::zero();
    let mut one_share= RingElm::zero();
    if is_server{
        v_share = RingElm::from(m as u32);
        one_share = RingElm::from(1);
    }//line5

    let mut omega_share = {
        let ring_m = RingElm::from(m as u32);
        one_share.sub(&p.offlinedata.base.qa_share[0]);
        one_share.mul(&ring_m);
        one_share   
    }; // Line6
    println!("v = {}, omega = {}", v_share.to_u32().unwrap(), omega_share.to_u32().unwrap());
    let beavers = &mut p.offlinedata.base.beavers;
    let mut beavers_ctr = 0;
    
    //Online-step-3. Start bit-by-bit prefix query, from Line7
    for i in 0..n{
        // println!("***************start the {} iteration***************", i);
        println!("qb[{}]={}", i, p.offlinedata.base.qb_share[i]);
        let mut mu_share = Arc::new(Mutex::new(RingElm::zero()));
        (0..m).into_par_iter().for_each(|j| {
            let new_bit = t[j*n+i]; //x[j][i]
            let (state_new, beta) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], new_bit);
            mu_share.lock().unwrap().add(&beta);
            new_state.lock().unwrap()[j] = state_new; 
        });

        /*mu is the number of elements having the prefix p_{i-1} || q[i] */
        println!("mu={:?}", mu_share);

        let v0_share = mu_share.lock().unwrap().clone(); //Line 13, the number of elements having the prerix p_{i-1} || q[i]
        let mut v1_share = v_share.clone();
        v1_share.sub(&mu_share.lock().unwrap()); // Line 14, the number of elements having prefix p_{i-1} || ~q[i]
        let v_share_t = (v0_share.clone(), v1_share.clone());
        println!("v0={:?}, v1={:?}", v0_share, v1_share);
        
        /*Exchange five ring_elements in parallel: u_i-w_i-alpha[i], (d_share, e_share) tuples for the two multiplication operation */
        let mut msg0  = Vec::<RingElm>::new();        // the send message
        let mut x_fnzc_share = RingElm::from(0);  //
        x_fnzc_share.add(&mu_share.lock().unwrap());
        x_fnzc_share.sub(&omega_share); //compute u_i-w_i, the x value of f_{NonZeroCheck}
        x_fnzc_share.add(&p.offlinedata.zc_a_share[i]); //mask the x value by alpha 
        println!("{:?} x_fznc_share={:?}",i, x_fnzc_share);
        msg0.push(x_fnzc_share);
        let rv = p.netlayer.exchange_ring_vec(msg0.clone()).await;
        let x_fznc = rv[0].clone();

        let mut msg1  = Vec::<u8>::new();        // the send message
        /*Obtain two beaver tuples and assure the beaver tuples are existing*/
        let d1_share= v0_share.clone(); //the fisrt v_alpha = v0_share 
        let d2_share= v1_share.clone(); //the second v_alpha = v0_share 
        let mut e1_share = if is_server{ RingElm::one() } else{ RingElm::zero() }; //the fisrt v_beta = 1-q[i+1] 
        let mut e2_share = if is_server{ RingElm::one() } else{ RingElm::zero() }; //the second v_beta = 1-q[i+1]

        let omega_t;
        if i < n-1{
            //println!("beaver{} {:?} {:?}",i, beavers[beavers_ctr], beavers[beavers_ctr+1]);
            e1_share.sub(&p.offlinedata.base.qa_share[i+1]);
            e2_share.sub(&p.offlinedata.base.qa_share[i+1]);

            msg1.append(&mut beavers[beavers_ctr].beaver_mul0(d1_share, e1_share));
            msg1.append(&mut beavers[beavers_ctr+1].beaver_mul0(d2_share, e2_share));

            let otherMsg1 = p.netlayer.exchange_byte_vec(&msg1.clone()).await;//Perform Network communication
            
            let omega0 = beavers[beavers_ctr].beaver_mul1(is_server, &otherMsg1[0..8].to_vec());
            let omega1 = beavers[beavers_ctr+1].beaver_mul1(is_server, &otherMsg1[8..16].to_vec());
            beavers_ctr += 2;

            println!("wo={:?}, w1={:?}", omega0, omega1);
            omega_t = (omega0, omega1);
        }
        else{
            omega_t = (RingElm::from(0), RingElm::from(0));    
        } //end else if i < n-1
    
        //start Line 12, calculate the f_{NonZeroCheck}(x_fnzc)
        let mut vec_eval = vec![false;32usize];
        let num_eval = x_fznc.to_u32();
        match num_eval {
            Some(numeric) => vec_eval = u32_to_bits(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }

        println!("{:?} x_fznc={:?}",i, x_fznc);
        println!("{:?} vec_eval={:?}",i, vec_eval);
        let y_fnzc: BinElm = p.offlinedata.zc_k_share[i].eval(&vec_eval);
        println!("y_fnzc={:?}", y_fnzc);
        cmp_bits[i] = y_fnzc.to_Bool();
        if is_server{
            cmp_bits[i] = !cmp_bits[i]
        }
        /*
        println!("cmp_share={}", cmp_bits[i]);*/
        //end Line 12 

        /*Line 19 */
        let simga_share = cmp_bits[i] ^ p.offlinedata.base.qb_share[i];
        let sigma = p.netlayer.exchange_a_bool(simga_share).await;
        // println!("End Reveal sigma {}", i);
        println!("sigma_{}={}", i, sigma);
        /*Line 20-21 */
        if sigma {
            v_share = v_share_t.1;
            omega_share = omega_t.1;
        }
        else {
            v_share = v_share_t.0;
            omega_share = omega_t.0;
        }

         /*Line 22 update the m idpf if sigma == 1, it means a wrong q[i] is choosed*/
        if sigma {
            (0..m).into_par_iter().for_each(|j| {
                let eval_bit = !t[j*n+i];//choose the oppsite value x[j][i]
                let (state_new, _) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], eval_bit);
                new_state.lock().unwrap()[j] = state_new; 
            });
        }
        old_state = new_state.lock().unwrap().clone(); //update the state
        // println!("***************end the {} iteration***************", i);
    
    }        
    cmp_bits     
}

pub async fn bitwise_kre(p: &mut MPCParty<BitKreOffline>, x_bits: &Vec<bool>, kValue: &RingElm) ->Vec<bool>{
    let m: usize = p.m;
    let n = p.n;

    let is_server = p.netlayer.is_server;
    let mut mask_bits = Vec::<bool>::new();//t in the paper, it is a bit vector of length n
    let mut cmp_bits = vec![false;n]; // the current prefix that has been checked
    let mut old_state = Vec::<EvalState>::new();
    let mut new_state = Arc::new(Mutex::new(Vec::<EvalState>::new()));
    let mut start_index = 0usize;
    /*This step compute n mask bits that will be used as the input of IDPF*/
    for i in 0..m{
        let init_state = p.offlinedata.base.k_share[i].eval_init();
        old_state.push(init_state.clone()); // Line2
        new_state.lock().unwrap().push(init_state.clone());  
        for j in 0..n{
            let t_share = x_bits[i*n + j] ^ p.offlinedata.base.a_share[i*n + j] ^ p.offlinedata.base.qb_share[j]; //x[i][j]^qb[j]
            mask_bits.push(t_share);
        }
    }
   
    /*Line 3: The reveal function for a bunch of bool data*/ 
    let t = p.netlayer.exchange_bool_vec(mask_bits.clone()).await; 
    /*Line5-6: v is the number of elements whose prefix is p_{i-1} */
    let mut vi_share= if is_server{ RingElm::from(m as u32) } else {RingElm::zero()};
    let mut k_share = kValue.clone();
    let mut beavers = p.offlinedata.base.beavers.iter_mut();
    //Online-step-3. Start bit-by-bit prefix query
    for i in 0..n{
        // println!("***************start the {} iteration***************", i);
        let mut mu_share: RingElm = RingElm::zero();
        println!("qb[{}]={}", i, p.offlinedata.base.qb_share[i]);
        let mut mu_share = Arc::new(Mutex::new(RingElm::zero()));
        (0..m).into_par_iter().for_each(|j| {
            let new_bit = t[j*n+i]; //x[j][i]
            let (state_new, beta) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], new_bit);
            mu_share.lock().unwrap().add(&beta);
            new_state.lock().unwrap()[j] = state_new; 
        });

        /*Round-1: CondEval & two multiplications*/
        let ri0_share = vi_share - mu_share.lock().unwrap().clone();
        let ri1_share = mu_share.lock().unwrap().clone();
        let mut msg0  = Vec::<u8>::new();
        let mut cond_Alpha0 = p.offlinedata.condeval_k_share[2*i].alpha + ri1_share - k_share;
        let mut cond_Alpha1 = p.offlinedata.condeval_k_share[2*i+1].alpha + ri0_share - k_share;
        //The decryption of two CondEval keys
        for j in 0..2{
            if j==0{
                msg0.append(&mut cond_Alpha0.to_u8_vec());
            }else{
                msg0.append(&mut cond_Alpha1.to_u8_vec());
            }
            let mut qb_share = p.offlinedata.base.qb_share[i];
            if is_server && j==1{
                    qb_share ^= true;
            }
            let pointer = p.offlinedata.condeval_k_share[2*i+j].pi ^ qb_share;
            msg0.push(if pointer{1u8}else{0u8});
            msg0.append(if qb_share{ &mut p.offlinedata.condeval_k_share[2*i+j].sk_1 } else {&mut p.offlinedata.condeval_k_share[2*i+j].sk_0});
        }

        let (beaver0, beaver1) = (beavers.next().unwrap(), beavers.next().unwrap());
        let ne_qa_share = {if is_server{RingElm::one()} else {RingElm::zero()}} - p.offlinedata.base.qa_share[i];
        msg0.append(&mut beaver0.beaver_mul0(p.offlinedata.base.qa_share[i], ri1_share));
        msg0.append(&mut beaver1.beaver_mul0(ne_qa_share, ri0_share));

        //Msg-format be: alpha0-4||condEvalDecrypt0||alpha1-4||condEvalDecrypt1||4+4(Mul)||4+4(Mul)
        let mut condEvalLen:usize = (msg0.len() - 4*2 - 8*2)/2;
        let otherMsg0 = p.netlayer.exchange_byte_vec(&msg0.clone()).await;//Perform Network communication
        //CondEval evaluation part:
        cond_Alpha0.add(&RingElm::from(otherMsg0[..4].to_vec()));
        cond_Alpha1.add(&RingElm::from(otherMsg0[condEvalLen+4..condEvalLen+8].to_vec()));
        let mut ci_0: BinElm = p.offlinedata.condeval_k_share[2*i].eval1(&cond_Alpha0, &otherMsg0[4..condEvalLen+4].to_vec());
        let ci_1 = p.offlinedata.condeval_k_share[2*i+1].eval1(&cond_Alpha1, &otherMsg0[8+condEvalLen..8+2*condEvalLen].to_vec());

        // println!("ci_0: {:?}", ci_0);
        // println!("ci_1: {:?} \n", ci_1);

        let mut ri_share = beaver0.beaver_mul1(is_server,&otherMsg0[2*(condEvalLen+4)..2*(condEvalLen+4)+8].to_vec());
        ri_share.add(& beaver1.beaver_mul1(is_server,&otherMsg0[2*(condEvalLen+4)+8..].to_vec()));
        /*End: Round-1: CondEval & two multiplications*/

        /*Round-2: Multiplications & Reveal*/
        let mut msg1  = Vec::<u8>::new();
        //Step Round-2-1: reveal delta_i
        ci_0.add(&ci_1);
        cmp_bits[i] = ci_0.to_Bool();
        let mut sigma_i = cmp_bits[i] ^ p.offlinedata.base.qb_share[i];
        msg1.push(if sigma_i{1u8}else{0u8});
        let mut li_share = vi_share - ri_share;
        let ( beaver2nd1, beaver2nd2) = (beavers.next().unwrap(),beavers.next().unwrap());
        msg1.append(&mut beaver2nd1.beaver_mul0(p.offlinedata.base.qa_share[i], li_share));//compute t1
        msg1.append(&mut beaver2nd2.beaver_mul0(p.offlinedata.base.qa_share[i], ri_share));//compute t2

        let otherMsg1 = p.netlayer.exchange_byte_vec(&msg1.clone()).await;
        sigma_i ^= otherMsg1[0] == 1;

        start_index=1;
        let t1_share = beaver2nd1.beaver_mul1(is_server,&otherMsg1[start_index..{start_index+=8; start_index}].to_vec());
        let t2_share = beaver2nd2.beaver_mul1(is_server,&otherMsg1[start_index..].to_vec());
        /*End: Round-2: Multiplications & Reveal*/

        // println!("sigma_i: {} ", sigma_i);
        //Refresh secret sharing values. 
        if sigma_i{
            k_share = k_share -  t2_share;
            vi_share = ri_share + t1_share - t2_share;
            // vi_share = t2_share - t1_share ;
        }else{
            k_share = k_share - ri_share + t2_share;
            // vi_share = li_share +  t1_share + t2_share;
            vi_share = li_share -  t1_share + t2_share;
        }

        if sigma_i{/*If sigma == 1, it means a wrong q[i] is choosed, then re-run evaluation to get the correct evaluation state*/
            (0..m).into_par_iter().for_each(|j| {
                let eval_bit = !t[j*n+i];//choose the oppsite value x[j][i]
                let (state_new, _) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], eval_bit);
                new_state.lock().unwrap()[j] = state_new; 
            });
        }
        old_state = new_state.lock().unwrap().clone(); //update the state
        // println!("***************end the {} iteration***************", i);
    }
    cmp_bits     
}