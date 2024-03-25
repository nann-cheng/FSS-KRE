use crate::mpc_party::MPCParty;
use fss::*;
use fss::idpf::*;
use fss::RingElm;
use fss::BinElm;
use fss::dpf::DPFKey;
use fss::beavertuple::BeaverTuple;
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

    // println!("begin to exchange to get t");
    /*Line 3: The reveal function for a bunch of bool data*/ 
    let t = p.netlayer.exchange_bool_vec(mask_bits.clone()).await;
    // println!("succeeds to get t");


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
    };//Line6

    // println!("v = {}, omega = {}", v_share.to_u32().unwrap(), omega_share.to_u32().unwrap());
    let beavers = &mut p.offlinedata.base.beavers;
    let mut beavers_ctr = 0;
    

    //Online-step-3. Start bit-by-bit prefix query, from Line7
    for i in 0..n{
        // println!("***************start the {} iteration***************", i);
        // println!("qb[{}]={}", i, p.offlinedata.base.qb_share[i]);
        let mut mu_share = Arc::new(Mutex::new(RingElm::zero()));
        (0..m).into_par_iter().for_each(|j| {
            let new_bit = t[j*n+i]; //x[j][i]
            let (state_new, beta) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], new_bit);
            mu_share.lock().unwrap().add(&beta);
            new_state.lock().unwrap()[j] = state_new; 
        });

        /*mu is the number of elements having the prefix p_{i-1} || q[i] */
        // println!("mu={:?}", mu_share);

        let v0_share = mu_share.lock().unwrap().clone(); //Line 13, the number of elements having the prerix p_{i-1} || q[i]
        let mut v1_share = v_share.clone();
        v1_share.sub(&mu_share.lock().unwrap()); // Line 14, the number of elements having prefix p_{i-1} || ~q[i]
        let v_share_t = (v0_share.clone(), v1_share.clone());
        // println!("v0={:?}, v1={:?}", v0_share, v1_share);
        
        /*Exchange five ring_elements in parallel: u_i-w_i-alpha[i], (d_share, e_share) tuples for the two multiplication operation */
        let x_fznc;
        let omega_t;
        if i < n-1{//pack the msg used in NZCheck and multiplication together
            let mut msg  = Vec::<u8>::new();        // the send message

            let mut x_fnzc_share = RingElm::from(0);  //
            x_fnzc_share.add(&mu_share.lock().unwrap());
            x_fnzc_share.sub(&omega_share); //compute u_i-w_i, the x value of f_{NonZeroCheck}
            x_fnzc_share.add(&p.offlinedata.zc_a_share[i]); //mask the x value by alpha 
            msg.append(&mut x_fnzc_share.to_u8_vec());

            /*Obtain two beaver tuples and assure the beaver tuples are existing*/
            let d1_share= v0_share.clone(); //the fisrt v_alpha = v0_share 
            let d2_share= v1_share.clone(); //the second v_alpha = v0_share 
            let mut e1_share = if is_server{ RingElm::one() } else{ RingElm::zero() }; //the fisrt v_beta = 1-q[i+1] 
            let mut e2_share = if is_server{ RingElm::one() } else{ RingElm::zero() }; //the second v_beta = 1-q[i+1]
            //println!("beaver{} {:?} {:?}",i, beavers[beavers_ctr], beavers[beavers_ctr+1]);
            e1_share.sub(&p.offlinedata.base.qa_share[i+1]);
            e2_share.sub(&p.offlinedata.base.qa_share[i+1]);

            msg.append(&mut beavers[beavers_ctr].beaver_mul0(d1_share, e1_share));
            msg.append(&mut beavers[beavers_ctr+1].beaver_mul0(d2_share, e2_share));

            let otherMsg = p.netlayer.exchange_byte_vec(&msg.clone()).await;//Perform Network communication

            x_fznc = RingElm::from(otherMsg[0..4].to_vec().clone());
            let omega0 = beavers[beavers_ctr].beaver_mul1(is_server, &otherMsg[4..12].to_vec());
            let omega1 = beavers[beavers_ctr+1].beaver_mul1(is_server, &otherMsg[12..20].to_vec());
            beavers_ctr += 2;

            // println!("wo={:?}, w1={:?}", omega0, omega1);
            omega_t = (omega0, omega1);
        }
        else{//deal with only NZCheck
            let mut msg0  = Vec::<RingElm>::new(); // the send message
            let mut x_fnzc_share = RingElm::from(0);  //
            x_fnzc_share.add(&mu_share.lock().unwrap());
            x_fnzc_share.sub(&omega_share); //compute u_i-w_i, the x value of f_{NonZeroCheck}
            x_fnzc_share.add(&p.offlinedata.zc_a_share[i]); //mask the x value by alpha 
            // println!("{:?} x_fznc_share={:?}",i, x_fnzc_share);
            msg0.push(x_fnzc_share);
            let rv = p.netlayer.exchange_ring_vec(msg0.clone()).await;
            x_fznc = rv[0].clone();


            omega_t = (RingElm::from(0), RingElm::from(0));
        } //end else if i < n-1
    
        //start Line 12, calculate the f_{NonZeroCheck}(x_fnzc)
        let mut vec_eval = vec![false;32usize];
        let num_eval = x_fznc.to_u32();
        match num_eval {
            Some(numeric) => vec_eval = u32_to_bits(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }

        // println!("{:?} x_fznc={:?}",i, x_fznc);
        // println!("{:?} vec_eval={:?}",i, vec_eval);
        let y_fnzc: BinElm = p.offlinedata.zc_k_share[i].eval(&vec_eval);
        // println!("y_fnzc={:?}", y_fnzc);
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
        // println!("sigma_{}={}", i, sigma);
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
    p.netlayer.print_benchmarking().await;

    cmp_bits     
}


pub async fn bitwise_max_opt(p: &mut MPCParty<BitMaxOffline>, x_bits: &Vec<bool>) -> Vec<bool> {
    let m: usize = p.m;
    let n = p.n;
    let is_server = p.netlayer.is_server;
    // let x_bits_debug = p.netlayer.exchange_bool_vec(x_bits.clone()).await; // debug purpose
   // debug purpose
    // for i in 0..m{
    //     for j in 0..n{
    //         if x_bits_debug[i*n + j] {print!("1");}
    //         else {print!("0");}
    //     }
    //     println!("");
    // }

    // let qb_debug = p.netlayer.exchange_bool_vec(p.offlinedata.base.qb_share.clone()).await;
    // println!("qb_share={:?}", qb_debug);

    // debug end
    let mut max_bits = vec![false; n];  //The return value that is equal to the cmp_bits computed in every round
    let mut mask_bits = Vec::<bool>::new();//t in the paper, it is a bit-vector of length n

    let mut old_state = Vec::<EvalState>::with_capacity(m);
    
    //let mut state_idpf = [[Vec::<EvalState>::with_capacity(m), Vec::<EvalState>::with_capacity(m)], [Vec::<EvalState>::with_capacity(m),Vec::<EvalState>::with_capacity(m)]];

    let mut state_idpf = Vec::<EvalState>::with_capacity(m*4);
    let mut mu_share = [[RingElm::from(0), RingElm::from(0)], [RingElm::from(0), RingElm::from(0)]];
    
    // Assume omega0 is omega[0][0] and omega1 is omega[1][0] in round 0
    let mut omega_share = [[RingElm::from(0), RingElm::from(0)], [RingElm::from(0), RingElm::from(0)]];
    
    let mut zc_key_it = p.offlinedata.zc_k_share.iter();
    let mut zc_alpha_it = p.offlinedata.zc_a_share.iter();
    let mut beaver_it = p.offlinedata.base.beavers.iter();

    
    let mut alpha_share_for_beaver: RingElm;
    let mut beta_share_for_beaver: RingElm;

    let mut cmp = [false, false];
    let mut sigma: bool;

    let one_share = if is_server {RingElm::from(1)} else {RingElm::zero()};
    //let m_share = if is_server {RingElm::from(m as u32)} else {RingElm::zero()};
    
    /****************************************Computation Framework 4-7: Start*******************************************************************/    
    for i in 0..m{
        let init_state = p.offlinedata.base.k_share[i].eval_init();
        old_state.push(init_state.clone()); // Line2

        for j in 0..n{
            let t_share = x_bits[i*n + j] ^ p.offlinedata.base.a_share[i*n + j] ^ p.offlinedata.base.qb_share[j]; //x[i][j]^qb[j]
            mask_bits.push(t_share);
        }
    }

    let t = p.netlayer.exchange_bool_vec(mask_bits.clone()).await; 
    /****************************************Computation Framework 4-7:   End*******************************************************************/ 
    

    /********************************************BitWise Round0: Start**************************************************************************/ 
    let _ = {
        /********************************************IDPF_Eval: Start****************************************************************/
        //let mut v_share = [m_share.clone(), RingElm::from(0)];
        omega_share[0][0] = {
            let ring_m = RingElm::from(m as u32);
            ring_m * (one_share - p.offlinedata.base.qa_share[0])
        }; // PI-protocol Line3: w = m(1-qa[0])
    
       for j in 0..m{
            let new_bit = t[j*n];
            let next_bit = t[j*n+1];
            
            let (state_new00, beta00) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], new_bit); // idpf(q[0])
            state_idpf.push(state_new00.clone());
            mu_share[0][0].add(&beta00);  // count the number of elements indexed by q[0]

            let (state_new01, beta01) = p.offlinedata.base.k_share[j].eval_bit(&state_new00, next_bit); // idpf(q[0 || q[1])])
            state_idpf.push(state_new01);
            mu_share[0][1].add(&beta01);  // count the number of elements indexed by q[0]||[q1]

            let (state_new10, beta10) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], !new_bit); // idpf(!q[0])
            state_idpf.push(state_new10.clone());
            mu_share[1][0].add(&beta10);  // count the number of elements indexed by !q[0]
        
            let (state_new11, beta11) = p.offlinedata.base.k_share[j].eval_bit(&state_new10, next_bit); // idpf(!q[0])
            state_idpf.push(state_new11);
            mu_share[1][1].add(&beta11);  // count the number of elements indexed by !q[0]||[q1]
        }; // Line 9-12 in Computation Framework 



        // let mu_share_debug = [mu_share[0][0].clone(), mu_share[0][1].clone(),mu_share[1][0].clone(),mu_share[1][1].clone()];
        // let mu_debug = p.netlayer.exchange_ring_vec(mu_share_debug.to_vec()).await;
        // println!("mu_debug0 = {:?}", mu_debug);

        /********************************************IDPF_Eval:   End****************************************************************/
        /********************************************Prepare for PI-protocol: Start**************************************************/
    
        let mut msg_ring_share: Vec<RingElm> = vec![RingElm::from(0); 5];  // To prepare 5 RingElemes to exchange
        let mut my_beavers = Vec::<BeaverTuple>::new();  

        /********************************************Prepare for PI-protocol:   End**************************************************/
        
        /***************************************ZeroCheck and BeaverMultiply: Start**************************************************/
        
        let f_nzc_key_share = zc_key_it.next().unwrap();
        let f_nzc_alpha_share = zc_alpha_it.next().unwrap().clone();
        let f_nzc_x_share = mu_share[0][0] - omega_share[0][0];
        msg_ring_share[0] = f_nzc_x_share + f_nzc_alpha_share; // mu - omega + alpha
        
        let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
        alpha_share_for_beaver =  mu_share[0][0].clone(); //v_share[0]; // alpha = v0
        beta_share_for_beaver = one_share - p.offlinedata.base.qa_share[1]; // beta = 1 - q1 
        let mut open_value = beaver.mul_open(alpha_share_for_beaver, beta_share_for_beaver);
        my_beavers.push(beaver);
        msg_ring_share[1] = open_value.0;
        msg_ring_share[2] = open_value.1;

        beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
        alpha_share_for_beaver =  mu_share[1][0]; // alpha = m - v0
        beta_share_for_beaver = one_share - p.offlinedata.base.qa_share[1]; 
        open_value = beaver.mul_open(alpha_share_for_beaver, beta_share_for_beaver);
        my_beavers.push(beaver);
        msg_ring_share[3] = open_value.0;
        msg_ring_share[4] = open_value.1;

        let msg0_ring_exchanged = p.netlayer.exchange_ring_vec(msg_ring_share.clone()).await; //restruct the message

        // Start: Execute the beaver multiplication
        let mut alpha_open = msg0_ring_exchanged[1].clone();
        let mut beta_open = msg0_ring_exchanged[2].clone();
        omega_share[0][0] = my_beavers[0].mul_compute(is_server, &alpha_open, &beta_open);
        alpha_open = msg0_ring_exchanged[3].clone();
        beta_open = msg0_ring_exchanged[4].clone();
        omega_share[1][0] = my_beavers[1].mul_compute(is_server, &alpha_open, &beta_open);
        // End: Execute the beaver multiplication
        omega_share[0][1] = omega_share[0][0].clone();
        omega_share[1][1] = omega_share[1][0].clone();
        //Start: evaluate the non-zero-check function
        let f_nzc_x = msg0_ring_exchanged[0].clone();
        cmp[0] = non_zero_check_func(&f_nzc_key_share, f_nzc_x, is_server); //if the max_value is prefixed with 1 
        max_bits[0] = cmp[0];
        
        // let omega_share_debug = [omega_share[0][0].clone(), omega_share[0][1].clone() , omega_share[1][0].clone(),omega_share[1][1].clone()]; //debug
        // let omega_debug = p.netlayer.exchange_ring_vec(omega_share_debug.to_vec()).await; //debug
        // println!("omega = {:?}", omega_debug); //debug

        //ZeroCheck([u] - [omega]), evaluate the non-zero function at f_znc_x
        //End: evaluate the non-zero-check function
        /***************************************ZeroCheck and BeaverMultiply:   End**************************************************/
    };
    /********************************************BitWise Round0:   End**************************************************************************/ 
    

    /********************************************BitWise Middle Rounds: Start*******************************************************************/ 
    sigma = true;
    
    for i in 1..(n-1){
        let sigma_index = !sigma as usize; 
        let mut msg_ring_share = vec![RingElm::from(0); 11]; // exchange 11 ring elements
        /**************************************Step2-1: Reaval, ZeroCheck and BeaverMultiply: Start******************************/
        //for reveal
        let x_reveal_share = RingElm::from((cmp[sigma_index] ^ p.offlinedata.base.qb_share[i - 1]) as u32);
        msg_ring_share[0] = x_reveal_share;
        
        //for ZeroCheck
        let f_nzc_key_share0 = zc_key_it.next().unwrap();
        let f_nzc_alpha_share0 = zc_alpha_it.next().unwrap().clone();
        msg_ring_share[1] = mu_share[0][0]-omega_share[0][sigma_index] + f_nzc_alpha_share0; //u[0][1]-omega[0][sigma]
        //msg_ring_share[1] = *mu_share[0][0].lock().unwrap()-omega_share[sigma_index][0] + f_nzc_alpha_share0; //u[0][1]-omega[0][sigma]
        //let f_nzc_key_share1 = zc_key_it.next().unwrap();
        //let f_nzc_alpha_share1 = zc_alpha_it.next().unwrap().clone();
        msg_ring_share[2] = mu_share[1][0]-omega_share[1][sigma_index] + f_nzc_alpha_share0; //u[1][1]-omega[1][sigma]
        //msg_ring_share[2] = *mu_share[1][0].lock().unwrap()-omega_share[sigma_index][1] + f_nzc_alpha_share0; //u[1][1]-omega[1][sigma]
        //for 4 beaver multiplication 
        let mut my_beavers = Vec::<BeaverTuple>::new();  
        let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
        alpha_share_for_beaver = one_share - p.offlinedata.base.qa_share[i+1]; // alpha = 1-q[i+1]
        beta_share_for_beaver = mu_share[0][1]; // beta = mu_share[0][1]
        let mut open_value = beaver.mul_open(alpha_share_for_beaver, beta_share_for_beaver);
        my_beavers.push(beaver);
        msg_ring_share[3] = open_value.0;
        msg_ring_share[4] = open_value.1; // for compute omega[0][0]
        
        beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
        alpha_share_for_beaver = one_share - p.offlinedata.base.qa_share[i+1]; // alpha = 1-q[i+1]
        beta_share_for_beaver = mu_share[1][1]; // beta =  mu_share[1][1]
        open_value = beaver.mul_open(alpha_share_for_beaver, beta_share_for_beaver);
        my_beavers.push(beaver);
        msg_ring_share[5] = open_value.0;
        msg_ring_share[6] = open_value.1; // for compute omega[0][1]

        beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
        alpha_share_for_beaver = one_share - p.offlinedata.base.qa_share[i+1]; // alpha = 1-q[i+1]
        beta_share_for_beaver = mu_share[0][0] - mu_share[0][1]; // beta = mu[0][0]-mu[0][1] 
        open_value = beaver.mul_open(alpha_share_for_beaver, beta_share_for_beaver);
        my_beavers.push(beaver);
        msg_ring_share[7] = open_value.0;
        msg_ring_share[8] = open_value.1; // for compute omega[1][0]

        beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
        alpha_share_for_beaver = one_share - p.offlinedata.base.qa_share[i+1]; // alpha = 1-q[i+1]
        beta_share_for_beaver = mu_share[1][0] - mu_share[1][1]; // beta = mu[1][0] - mu[1][1]
        open_value = beaver.mul_open(alpha_share_for_beaver, beta_share_for_beaver);
        my_beavers.push(beaver);
        msg_ring_share[9] = open_value.0;
        msg_ring_share[10] = open_value.1; // for compute omega[1][1]
        
        let msg_ring_exchange = p.netlayer.exchange_ring_vec(msg_ring_share).await;

        //reveal 
        sigma = if msg_ring_exchange[0].to_u32().unwrap() == 1 {true} else { false};

        // println!("Round{}**************************",i);
        // println!("sigma = {}", msg_ring_exchange[0].to_u32().unwrap());

        // ZeroCheck
        cmp[0] = non_zero_check_func(&f_nzc_key_share0, msg_ring_exchange[1], is_server); // cmp[1] = ZeroCheck(u[0][1] - omega[0][sigma]) 
        cmp[1] = non_zero_check_func(&f_nzc_key_share0, msg_ring_exchange[2], is_server); // cmp[2] = ZeroCheck(u[1][1] - omega[1][sigma])
        
        // 4-Beaver Multiplications
        let mut alpha_open = msg_ring_exchange[3].clone();
        let mut beta_open = msg_ring_exchange[4].clone();
        omega_share[0][0] = my_beavers[0].mul_compute(is_server, &alpha_open, &beta_open); // omega[0][0]

        alpha_open = msg_ring_exchange[5].clone();
        beta_open = msg_ring_exchange[6].clone();
        omega_share[0][1] = my_beavers[1].mul_compute(is_server, &alpha_open, &beta_open); // omega[0][1]

        alpha_open = msg_ring_exchange[7].clone();
        beta_open = msg_ring_exchange[8].clone();
        omega_share[1][0] = my_beavers[2].mul_compute(is_server, &alpha_open, &beta_open); // omega[1][0]

        alpha_open = msg_ring_exchange[9].clone();
        beta_open = msg_ring_exchange[10].clone();
        omega_share[1][1] = my_beavers[3].mul_compute(is_server, &alpha_open, &beta_open); // omega[1][1] : fixed the index of omegam, 240313

        
        /******************************************Step2-1: Reaval, ZeroCheck and BeaverMultiply:   End********************************/

        /*********************************************Step2-2: IDPF Evaluation: Start*************************************************/
        if !sigma{
            max_bits[i] = cmp[0];
            mu_share[0][0] = mu_share[0][1];
        }
        else{
            max_bits[i] = cmp[1];
            mu_share[0][0] = mu_share[1][1]; 
        }
        
        mu_share[0][1] = RingElm::from(0);
        mu_share[1][0] = RingElm::from(0);
        mu_share[1][1] = RingElm::from(0);
        for j in 0..m{
            let new_bit = t[j*n + i]; 
            let next_bit = t[j*n + i + 1];

            if !sigma{
                old_state[j] = state_idpf[4*j].clone();
                state_idpf[4*j] = state_idpf[4*j+1].clone();
            }
            else {
                old_state[j] = state_idpf[4*j+2].clone();
                state_idpf[4*j] = state_idpf[4*j+3].clone();
            }// update the state of p[i] from p[i-1], which has been computed in last round  
                
            // update the state of p[i-1]||p[i]||p[i+1] from p[i]  
            let (state_new01, beta01) = p.offlinedata.base.k_share[j].eval_bit(&state_idpf[4*j], next_bit); 
            state_idpf[4*j+1] = state_new01;    
            mu_share[0][1].add(&beta01);  
    
            // update the state of p[i] from !p[i-1] 
            let (state_new10, beta10) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], !new_bit); 
            state_idpf[4*j+2] = state_new10.clone(); 
            mu_share[1][0].add(&beta10); 
                
            // update the state of !p[i-1]||p[i]||p[i+1] from p[i]  
            let (state_new11, beta01) = p.offlinedata.base.k_share[j].eval_bit(&state_new10, next_bit); 
            state_idpf[4*j+3] = state_new11.clone();    
            mu_share[1][1].add(&beta01);  
        }
        /*********************************************Step2-2: IDPF Evaluation:   End*************************************************/
        
        // let mu_share_debug = [mu_share[0][0].clone(),mu_share[0][1].clone(),mu_share[1][0].clone(),mu_share[1][1].clone()];
        // let mu_debug = p.netlayer.exchange_ring_vec(mu_share_debug.to_vec()).await;
        // println!("mu_debug = {:?}", mu_debug);
        // let omega_share_debug = [omega_share[0][0].clone(), omega_share[0][1].clone() , omega_share[1][0].clone(),omega_share[1][1].clone()]; //debug
        // let omega_debug = p.netlayer.exchange_ring_vec(omega_share_debug.to_vec()).await; //debug
        // println!("omega = {:?}", omega_debug); //debug
    }
    /********************************************BitWise Middle Rounds:   End*********************************************************/ 

     /********************************************BitWise The Last Rounds:   End******************************************************/
     let _ = {
        let sigma_index = !sigma as usize; 
        let mut msg_ring_share = vec![RingElm::from(0); 3];
        /**************************************Step2-1: Reaval, ZeroCheck and BeaverMultiply: Start******************************/
        //for reveal sigma[n-2]
        let x_reveal_share = RingElm::from((cmp[sigma_index] ^ p.offlinedata.base.qb_share[n - 2]) as u32);
        msg_ring_share[0] = x_reveal_share;
        //for 2-ZeroCheck-s
        let f_nzc_key_share0 = zc_key_it.next().unwrap();
        let f_nzc_alpha_share0 = zc_alpha_it.next().unwrap().clone();
        msg_ring_share[1] = mu_share[0][0]-omega_share[0][sigma_index] + f_nzc_alpha_share0; //u[0][1]-omega[0][sigma]
        //let f_nzc_key_share1 = zc_key_it.next().unwrap();
        //let f_nzc_alpha_share1 = zc_alpha_it.next().unwrap().clone();
        msg_ring_share[2] = mu_share[1][1]-omega_share[1][sigma_index] + f_nzc_alpha_share0; //u[1][1]-omega[1][sigma]
        
        //for reveal two possible values of sigma[n-1]
        //x_reveal_share = RingElm::from((cmp[0] ^ p.offlinedata.base.qb_share[n - 1]) as u32);
        //msg_ring_share[3] = x_reveal_share;

        //x_reveal_share = RingElm::from((cmp[1] ^ p.offlinedata.base.qb_share[n - 1]) as u32);
        //msg_ring_share[4] = x_reveal_share;

        let msg_ring_exchange = p.netlayer.exchange_ring_vec(msg_ring_share).await; 

        //reveal sigma[n-2]
        sigma = if msg_ring_exchange[0] == RingElm::one() {true} else {false}; 

        // 2-ZeroCheck-s
        cmp[0] = non_zero_check_func(&f_nzc_key_share0, msg_ring_exchange[1], is_server); // cmp[1] = ZeroCheck(u[0][1] - omega[0][sigma]) 
        cmp[1] = non_zero_check_func(&f_nzc_key_share0, msg_ring_exchange[2], is_server); // cmp[2] = ZeroCheck(u[1][1] - omega[1][sigma])
        
        if !sigma
        {
            max_bits[n-1] = cmp[0]; 
        }
        else{
            mask_bits[n-1] = cmp[1];
        }      

     }; // The last round

      /********************************************BitWise The Last Rounds:   End*****************************************************/ 

    // let result_bits = p.netlayer.exchange_bool_vec(max_bits.clone()).await;
    // println!("result_bits = {:?}", result_bits);
    p.netlayer.print_benchmarking().await;
    max_bits
}

fn non_zero_check_func(key_share: &DPFKey<BinElm>, x: RingElm, is_server: bool) -> bool{
    let mut vec_eval = vec![false;32usize];
    let num_eval = x.to_u32();
    match num_eval {
        Some(numeric) => vec_eval = u32_to_bits(32usize,numeric),
        None      => println!( "u32 Conversion failed!!" ),
    }

    let y = key_share.eval(&vec_eval);
    let cmp = y.to_Bool();
    if is_server{
       !cmp
    }
    else{
        cmp   
    }
}