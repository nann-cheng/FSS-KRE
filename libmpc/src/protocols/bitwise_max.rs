use crate::mpc_party::MPCParty;
use fss::*;
use fss::idpf::*;
use fss::RingElm;
use fss::BinElm;
use crate::offline_data::offline_bitwise_max::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

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