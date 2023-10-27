use crate::mpc_party::MPCParty;
use fss::*;
use fss::idpf::*;
use fss::RingElm;
use fss::BinElm;
use crate::offline_data::offline_bitwise_kre::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

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

    println!("Debug 0");

    //Online-step-3. Start bit-by-bit prefix query
    for i in 0..n{
        // println!("***************start the {} iteration***************", i);
        let mut mu_share: RingElm = RingElm::zero();
        // println!("qb[{}]={}", i, p.offlinedata.base.qb_share[i]);
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

        println!("Debug 1");
        let (beaver0, beaver1) = (beavers.next().unwrap(), beavers.next().unwrap());
        let ne_qa_share = {if is_server{RingElm::one()} else {RingElm::zero()}} - p.offlinedata.base.qa_share[i];
        msg0.append(&mut beaver0.beaver_mul0(p.offlinedata.base.qa_share[i], ri1_share));
        msg0.append(&mut beaver1.beaver_mul0(ne_qa_share, ri0_share));

        //Msg-format be: alpha0-4||condEvalDecrypt0||alpha1-4||condEvalDecrypt1||4+4(Mul)||4+4(Mul)
        let mut condEvalLen:usize = (msg0.len() - 4*2 - 8*2)/2;

        println!("Debug 1.4");

        let otherMsg0 = p.netlayer.exchange_byte_vec(&msg0.clone()).await;//Perform Network communication

        println!("Debug 1.5");

        //CondEval evaluation part:
        cond_Alpha0.add(&RingElm::from(otherMsg0[..4].to_vec()));
        cond_Alpha1.add(&RingElm::from(otherMsg0[condEvalLen+4..condEvalLen+8].to_vec()));
        let mut ci_0: BinElm = p.offlinedata.condeval_k_share[2*i].eval1(&cond_Alpha0, &otherMsg0[4..condEvalLen+4].to_vec());
        let ci_1 = p.offlinedata.condeval_k_share[2*i+1].eval1(&cond_Alpha1, &otherMsg0[8+condEvalLen..8+2*condEvalLen].to_vec());

        // println!("ci_0: {:?}", ci_0);
        // println!("ci_1: {:?} \n", ci_1);

        println!("Debug 2");

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

        println!("Debug 3");

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
    p.netlayer.print_benchmarking().await;
    cmp_bits     
}