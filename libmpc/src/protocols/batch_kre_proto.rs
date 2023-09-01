use fss::*;
use fss::idpf::*;
use fss::RingElm;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use crate::offline_data::offline_batch_kre::BatchKreOffline;
use super::super::mpc_party::*;

pub fn batch_eval_of_idpf(idpf: &IDPFKey<RingElm>, old_state: &EvalState, x_batch: &[bool], batch_size: usize) ->(EvalState, RingElm){
    let mut cur_state: EvalState = old_state.clone();
    let mut y_idpf = RingElm::from(0);
    for i in 0..batch_size{
        let (new_state, y_eval) =  idpf.eval_bit(&cur_state, x_batch[i]);
        cur_state = new_state;
        y_idpf = y_eval;
    }

    (cur_state, y_idpf)
}

//Assume n % batchsize == 0
pub async fn batch_kre(p: &mut MPCParty<BatchKreOffline>, x_bits: &Vec<bool>, batch_size: usize, kValue: &RingElm) ->Vec<bool>{
    let mut k_star = kValue.clone();
    println!("k inital {:?}", k_star);

    let m: usize = p.m;
    let n = p.n;
    println!("m={}, n={}", m, n);
    let is_server = p.netlayer.is_server;

    let every_batch_num = 1 << batch_size;
    let block_num = n / batch_size;
    //println!("m={}, n={}", m, n);
    /***********************************************************************************************************************************************/
    /******************************************************  START: Line8 Comute \tao F_{BDC} i in 0..{\tao} ***************************************/
    let mut const_bdc_bits = Vec::<bool>::new();
    for i in 0..every_batch_num{
        let cur_bits = u32_to_bits_BE(batch_size, (every_batch_num-1-i).try_into().unwrap()); 
        //convert int to {omega}-bits. q[0..{omega}]
        const_bdc_bits.extend(cur_bits);
    }
    /******************************************************  END:   Line8 Comute \tao F_{BDC} i in 0..{\tao} ***************************************/
    /***********************************************************************************************************************************************/
    //println!("const_bdc {:?}", const_bdc_bits);
    /***********************************************************************************************************************************************/
    /******************************************************  START: Line2-5: Reveal t = x^q^{\alpha}  **********************************************/
    let mut mask_bits = Vec::<bool>::new();//t in the paper, it is a bit vector of length n
    let mut cmp_bits = vec![false;n]; // the current prefix that has been checked
    let mut idpf_state = Vec::<EvalState>::new();
    //let mut new_state = Vec::<EvalState>::new();

    //println!("k_share_len:{}", p.offlinedata.base.k_share.len());
    //println!("a_share_len:{}", p.offlinedata.base.a_share.len());
    /*Line 2-5: This step compute n mask bits that will be used as the input of IDPF*/
    for i in 0..m{
        let init_state = p.offlinedata.base.k_share[i].eval_init();
        idpf_state.push(init_state.clone()); 
        //new_state.push(init_state.clone()); 
        for j in 0..n{
            let t_share = x_bits[i*n + j] ^ p.offlinedata.base.a_share[i*n + j] ^ p.offlinedata.base.qb_share[j]; //x[i][j]^qb[j]
            mask_bits.push(t_share);
        }
    }
   
    /*Line 4: The reveal function for a bunch of bool data*/ 
    let t = p.netlayer.exchange_bool_vec(mask_bits.clone()).await; 
    /******************************************************  END: Line2-5: Reveal t = x^q^{\alpha} **************************************************/
    /***********************************************************************************************************************************************/
    println!("q={:?}", p.offlinedata.base.qb_share);
    /***********************************************************************************************************************************************/
    /********************************************************  START: Line6-25: Compute vector V   *************************************************/
    for block_order in 0..block_num{ // for every block
        /*****************************************************************************************************************************/
        /********************************************************  START: Line6-14: Compute vector V   *******************************/
        let mut V_c = Vec::<RingElm>::new();
        let mut tmp_state = Arc::new(Mutex::new(Vec::<Vec::<EvalState>>::new()));
        for i in 0..m{
            let tmp_state_i = Vec::<EvalState>::new();
            //tmp_state_i.push(idpf_state[i].clone()); // initialize the j-th idpf's state 
            tmp_state.lock().unwrap().push(tmp_state_i);
        }   //prepare for m state vectors, each of which contains  {\tao} state

        let index_start = block_order * batch_size;
        let index_end: usize = (block_order+1) * batch_size; //change them out from the loop
        for i in 0..every_batch_num{ // for every \{tao} - j
            let mut v_item = Arc::new(Mutex::new(RingElm::from(0)));
            (0..m).into_par_iter().for_each(|j| {
                let x_idpf = bits_Xor(&const_bdc_bits[i * batch_size..(i + 1) * batch_size].to_vec(), &t[(j * n + index_start)..(j * n + index_end)].to_vec());
                let old_state = idpf_state[j].clone(); // The initial state of the m-th idpf is idpf_state[i]
                let (state_new, beta) = batch_eval_of_idpf(&p.offlinedata.base.k_share[j], &old_state, &x_idpf, batch_size);
                tmp_state.lock().unwrap()[j].push(state_new); // The j-th state for the k-th idpf
                v_item.lock().unwrap().add(&beta);
            });
            V_c.push(*v_item.lock().unwrap());
        }
        /********************************************************  END  : Line6-14: Compute vector V   *******************************/
        /*****************************************************************************************************************************/
        //println!("V[{}]={:?}", block_order, V_c);
        /*****************************************************************************************************************************/
        /********************************************************  START: F_BatchMax  Line15 *****************************************/
        let f_batch_kre = {
            /***********************************   START: Line3-5 Compute idpf-s    **********************************/
            let q_share = &p.offlinedata.base.qb_share[block_order*batch_size..(block_order+1)*batch_size];
            let let_a_share = &p.offlinedata.let_a_share[block_order*every_batch_num..(block_order+1)*every_batch_num];
            let let_k_share = &p.offlinedata.let_k_share[block_order*every_batch_num..(block_order+1)*every_batch_num]; 
            //update0817: it needs to call f_znc {\tao} times in every block  
            let M_qel = &p.offlinedata.qelmmatrix_share[block_order];
            //M_qel.print();
            let qb = &mut p.offlinedata.qbeavers[block_order];
            let cb = &mut p.offlinedata.cbeavers[block_order];
            let kb = &mut p.offlinedata.kbeavers[block_order];
            let mut v_star_v = Vec::<RingElm>::new();
            let mut x_f_let_share = Vec::<RingElm>::new();

            /***********************************   START: Line1 Compute v * M    **********************************/
            let mut msg0  = Vec::<u8>::new();

            for i in 0..every_batch_num{
                for j in 0..every_batch_num{
                    msg0.append(&mut qb[i * every_batch_num + j].beaver_mul0(V_c[j], M_qel.locate(i, j)));
                }
            }

            let otherMsg0 = p.netlayer.exchange_byte_vec(&msg0.clone()).await;//Perform Network communication
        
            let mut V_M = Vec::<RingElm>::new();
            for i in 0..every_batch_num{
                V_M.push(RingElm::zero());
            }

            for i in 0..every_batch_num{
                for j in 0..every_batch_num{
                    V_M[i].add(& qb[i * every_batch_num + j].beaver_mul1(is_server, &otherMsg0[8 * (i *  every_batch_num + j)..8 * (i *  every_batch_num + j + 1)].to_vec()));
                }
            }

            /***********************************   END:   Line1 Compute v * M    **********************************/
            //println!("V * M[{}]={:?}", block_order, V_M);
            /***********************************   START: Line2-5 Compute LessEqualThan   **********************************/
            for t in 1..every_batch_num+1{
                let mut v_star = RingElm::zero();
                for j in 0..t{
                    v_star.add(&V_M[j].clone());
                }
                v_star_v.push(v_star);
                
                v_star.sub(&k_star);
                v_star.add(&let_a_share[t-1]); 
                
                x_f_let_share.push(v_star);  
            }

            let mut x_f_let = p.netlayer.exchange_ring_vec(x_f_let_share.clone()).await;

            let mut cmp = Vec::<RingElm>::new();
            for i in 0..every_batch_num{ 
                cmp.push(let_k_share[i].eval(&x_f_let[i]));
            } 

            /***********************************   END:   Line2-5 Compute LessEqualThan    **********************************/
            //println!("pre-ordered cmp[{}]={:?}", block_order, cmp);
            /******************************** START:  Line9-19 Compute bt  ******************************************/
            let mut bt = Vec::<RingElm>::new();

            bt.push(cmp[0].clone());
            
            let mut msg1  = Vec::<u8>::new();

            for t in 1..every_batch_num{
                let mut cmp_t = RingElm::zero();
                if is_server{
                    cmp_t.add(&RingElm::one());
                } 

                cmp_t.sub(&cmp[t-1]);
                //println!("cmp_t after sub[{}]={:?}", t, cmp_t);
                msg1.append(&mut cb[t-1].beaver_mul0(cmp[t], cmp_t));     
            }

            let otherMsg1 = p.netlayer.exchange_byte_vec(&msg1.clone()).await;//Perform Network communication
        
            for t in 1..every_batch_num{
                bt.push(cb[t-1].beaver_mul1(is_server, &otherMsg1[8 * (t-1)..8 * (t)].to_vec()));
            }
            /******************************** END:    Line9-19 Compute bt  ******************************************/
            //println!("bt[{}]={:?}", block_order, bt);
            /******************************** START:  Line17 Compute y_i  ******************************************/
            // Here, there is only one zero in bt
            let mut y = Vec::<bool>::new();
            for i in 0..batch_size{
                y.push(q_share[i]);
                //y.push(false);
            }

            for i in 0..every_batch_num{
                let const_i_bits = &const_bdc_bits[i*batch_size..(i+1)*batch_size];
                for j in 0..batch_size{
                    let bt_b;

                    if (RingElm::to_u32(&bt[i]).unwrap() & 1) == 1 {
                        bt_b = true;
                    }else{
                        bt_b = false;
                    }
                    y[j] = y[j] ^ (bt_b && const_i_bits[j]); //changed index-s
                }
            }
            /******************************** END:    Line17 Compute y_i  ******************************************/
            //println!("Pre-y[{}]={:?}", block_order, y);
            let result = p.netlayer.exchange_bool_vec(y).await;
            println!("Pre-y[{}]={:?}", block_order, result);
            /******************************** START:  Line23 Compute k_star  ******************************************/
            let mut msg2  = Vec::<u8>::new();

            msg2.append(&mut kb[0].beaver_mul0(bt[0], k_star));

            for t in 1..every_batch_num{
                let mut k_v = k_star.clone();
                k_v.sub(&v_star_v[t-1]);
                msg2.append(&mut kb[t].beaver_mul0(bt[t], k_v));
            }

            let otherMsg2 = p.netlayer.exchange_byte_vec(&msg2.clone()).await;//Perform Network communication
        
            k_star = RingElm::zero();
            for t in 0..every_batch_num{
                k_star.add(&kb[t].beaver_mul1(is_server, &otherMsg2[8 * t..8 * (t + 1)].to_vec()));
            }
            //println!("k_star[{}] {:?}", block_order, k_star);
            /******************************** END:    Line23 Compute k_star  ******************************************/
            
            result
        };       
        /********************************************************  END:   F_BatchMax Line15  *****************************************/
        /*****************************************************************************************************************************/
        println!("Reveal-y[{}]={:?}", block_order, f_batch_kre);
        let mut path_eval = 0; // define which path is used
        for i in 0..batch_size{
            path_eval = path_eval << 1;
            if f_batch_kre[i]{
                path_eval += 1;
            }
        }  //line 17: rebuild the numerical max num
        println!("path_eval={}", path_eval);
        for i in 0..m{
            idpf_state[i] = tmp_state.lock().unwrap()[i][every_batch_num-path_eval-1].clone();  //update 0819: a big bug fixed here
        } // Line18-20: update the idpf-s

        if is_server{ //Here, I fixed a big bug, update:0819
            for i in 0..batch_size{
                cmp_bits[i+block_order*batch_size] = f_batch_kre[i] ^ p.offlinedata.base.qb_share[i+block_order*batch_size]; // A big change here, last version forgot xor the q_share 
                //cmp_bits[i+block_order*batch_size] = f_batch_kre[i] 
            }
        }
        else{
            for i in 0..batch_size{
                cmp_bits[i+block_order*batch_size] = p.offlinedata.base.qb_share[i+block_order*batch_size]; // A big change here, last version forgot xor the q_share 
                //cmp_bits[i+block_order*batch_size] = false;
            }
        }
    }
     /********************************************************  END:   Line6-25*****************   *************************************************/
    /***********************************************************************************************************************************************/  
    cmp_bits     
}




#[cfg(test)]
mod tests {
    //use crate::mpc_party::f_max_batch;
    //use crate::offline_data::BitMaxOffline;
    //use crate::offline_data::f_conv_matrix;
    //use fss::RingElm;
    use fss::prg::{PrgSeed, FixedKeyPrgStream};
    //use rand::Rng;
    use tokio::sync::mpsc;
    use fss::mbeaver::*;
    use fss::u32_to_bits_BE;

    use fss::qmatrix::*;
    use fss::mbeaver::MBeaverBlock;
    #[tokio::test]
    async fn f_max_batch_works(){
    }

    #[test]
    fn const_bits_works(){
        let batch_size: usize = 3;
        let every_batch_num = 1 << batch_size;
        let mut const_bdc_bits = Vec::<bool>::new();
        for i in 0..every_batch_num{
            let cur_bits = u32_to_bits_BE(batch_size, (every_batch_num-1-i).try_into().unwrap()); 
            //convert int to {omega}-bits. q[0..{omega}]
            print!("{}=", every_batch_num-i);
            for j in 0..batch_size{
                if cur_bits[j]{
                    print!("1");
                }
                else{
                    print!("0");
                }
            }
            println!("");
            const_bdc_bits.extend(cur_bits);
        }
        for j in 0..batch_size * every_batch_num{
            if const_bdc_bits[j]{
                print!("1");
            }
            else{
                print!("0");
            }
        }
        println!("");
    }
   
    async fn times_operation(mut cmp: Vec<bool>, batch_size: usize, M: QMatrix, binarr_beavers: Vec<MBeaver>, is_server: bool, writer: mpsc::Sender<Vec<bool>>, mut reader: mpsc::Receiver<Vec<bool>>) -> Vec<bool>{
        let every_batch_num  = 1 << batch_size;
        let mut it_bbeaver = binarr_beavers.iter();
        let mut exchange_msg_1 = Vec::<bool>::new();
        let mut consumed_beavers = Vec::<MBeaver>::new(); //store every consumed beaver tuple 
        //every item in cmp and M will take part in {\tao} times multilication
       
         
        for i in 0..every_batch_num{ // for every item in cmp and every line in M 
            for j in 0..every_batch_num{ //prepare for cmp[j] * M[i][j]
                let beaver_new = it_bbeaver.next().unwrap();
                let delta_0 = cmp[j] ^ beaver_new[0];
                let delta_1 = M.locate(i, j) ^ beaver_new[1];
                exchange_msg_1.push(delta_0); 
                exchange_msg_1.push(delta_1);
                consumed_beavers.push(beaver_new.clone());
            }
        }
        
        //let delta = p.netlayer.exchange_bool_vec(exchange_msg_1).await;
        writer.send(exchange_msg_1.clone()).await.unwrap();
        let exchange_msg_2 = reader.recv().await.unwrap();
        assert_eq!(exchange_msg_1.len(), exchange_msg_2.len());
        let mut exchange_msg = Vec::<bool>::new();
        for i in 0..exchange_msg_1.len(){
            let exchange_msg_i = exchange_msg_1[i] ^ exchange_msg_2[i];
            exchange_msg.push(exchange_msg_i);
        }
        let delta = exchange_msg;

        let mut index_delta = 0;
        
        for i in 0..cmp.len(){
            cmp[i] = false;
        }
      
        for i in 0..every_batch_num{ // for every item in cmp and every line in M 
            for j in 0..every_batch_num{ //prepare for cmp[j] * M[i][j]
                let beaver_new = consumed_beavers[index_delta].clone();
                let delta_0 = delta[index_delta*2];
                let delta_1 = delta[index_delta*2+1];
                let mut delta = Vec::<bool>::new();
                delta.push(delta_0);
                delta.push(delta_1);
                index_delta += 1;
                let bool_product = product(&delta, &beaver_new, is_server).unwrap();
                cmp[i] = cmp[i] ^ bool_product;
            }
        }
        println!("cm.len={}", cmp.len());
        cmp
        
    }

    async fn  v_times_matrix(v: &Vec<bool>, M: &QMatrix, q: &Vec<bool>, batch_size: usize, stream: &mut FixedKeyPrgStream)->(Vec<bool>, Vec<bool>){
        let every_batch_num = 1 << batch_size;
    
        let q1 = stream.next_bits(batch_size);
        let mut q2 = Vec::<bool>::new();
        for i in 0..batch_size{
            q2.push(q[i] ^ q1[i]);
        }
        let (M1, M2) = M.split();

        let mut binary_beavers1 = Vec::<MBeaver>::new();
        let mut binary_beavers2 = Vec::<MBeaver>::new();
        for i in 0..100{
            let binary_beaver_item = MBeaver::gen(2);
            let (binary_beaver_item1, binary_beaver_item2) = binary_beaver_item.split();
            binary_beavers1.push(binary_beaver_item1);
            binary_beavers2.push(binary_beaver_item2);
        }

        // c = [0, 0, 1, 0]
        let cmp1 = stream.next_bits(every_batch_num);
        let mut cmp2 = v.clone();
        for i in 0..every_batch_num{
            cmp2[i] = cmp1[i] ^ v[i];
        }
        let mut cmp = Vec::<bool>::new();
        for i in 0..every_batch_num{
            let cmp_i = cmp1[i] ^ cmp2[i];
            cmp.push(cmp_i);
        }

        //simulate the network's action
        let (tx, mut rx) = mpsc::channel::<Vec<bool>>(4096);
        let (ty, mut ry) = mpsc::channel::<Vec<bool>>(4096);
        let task1 = times_operation(cmp1, batch_size, M1, binary_beavers1, false, tx, ry);
        let task2 = times_operation(cmp2, batch_size, M2, binary_beavers2, true, ty, rx);
        let (c1, c2) = futures::join!(task1, task2);
        (c1, c2)
    }

    #[tokio::test]
    async fn v_times_matrix_works(){
        let batch_size = 3;
        let every_batch_num = 1 << batch_size;
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let q = stream.next_bits(batch_size);
        let M = f_conv_matrix(&q, batch_size);
        let v = stream.next_bits(every_batch_num);
        let (r1, r2) = v_times_matrix(&v, &M, &q, batch_size, &mut stream).await;
        assert_eq!(r1.len(), v.len());
        assert_eq!(r2.len(), v.len());

        for i in 0..every_batch_num{
            let mut item = false;
            for j in 0..every_batch_num{
                item ^= v[j] && M.locate(i, j);
            }
            assert_eq!(item, r1[i] ^ r2[i]);
        }
    }

    async fn party_extract(cmp: &Vec<bool>, q_share: &Vec<bool>, mbs: &Vec<MBeaver>, batch_size: usize, is_server: bool, const_bdc_bits: &Vec<bool>, writer: mpsc::Sender<Vec<bool>>, mut reader: mpsc::Receiver<Vec<bool>>) -> Vec<bool>{
        let every_batch_num = 1 << batch_size;
        let mut exchange_msg_1 = Vec::<bool>::new();

        for i in 1..every_batch_num{
            let mut item = cmp[i] ^ mbs[i-1][0];
            exchange_msg_1.push(item);
            for j in 0..i{ //1-cmp[j]
                if is_server{
                    item = cmp[j] ^ mbs[i-1][1+j];
                }
                else{
                    item = !cmp[j] ^ mbs[i-1][1+j];
                }
                exchange_msg_1.push(item);
            }
        }

        //simulate the action of exchange messages
        writer.send(exchange_msg_1.clone()).await.unwrap();
        let exchange_msg_2 = reader.recv().await.unwrap();
        let mut c = exchange_msg_2;
        for i in 0..c.len(){
            c[i] = c[i] ^ exchange_msg_1[i];
        }

        let mut bt = Vec::<bool>::new();
        bt.push(cmp[0]);
            
        let mut index_delta = 0;
        for i in 1..every_batch_num{
            let items = c[index_delta..=index_delta+i].to_vec();
            let bt_i = product(&items, &mbs[i-1], is_server).unwrap();
            bt.push(bt_i);
            index_delta += i;
            index_delta += 1;
        }// finished test already
            /******************************** END:    Line11 Compute bt  *******************************************/

        /******************************** START:  Line14 Compute y_i  ******************************************/
        // Here, there is only one zero in bt
        println!("bt={:?}", bt);
        let mut y = Vec::<bool>::new();
        for i in 0..batch_size{
            y.push(q_share[i]);
        }

        for i in 0..every_batch_num{
            let const_i_bits = &const_bdc_bits[i*batch_size..(i+1)*batch_size];
            for j in 0..batch_size{
                y[j] = y[j] ^ (bt[i] && const_i_bits[j]);
            }
        }

        y
    }

    async fn do_extract(v: &Vec<bool>, q: &Vec<bool>, stream: &mut FixedKeyPrgStream)->(Vec<bool>, Vec<bool>){
        let batch_size = q.len();
        let every_batch_num = v.len();
        let mut const_bdc_bits = Vec::<bool>::new();
        for i in 0..every_batch_num{
            let cur_bits = u32_to_bits_BE(batch_size, (every_batch_num-1-i).try_into().unwrap()); 
            //convert int to {omega}-bits. q[0..{omega}]
            const_bdc_bits.extend(cur_bits);
        }
        let mut v1 = v.clone();
        let v2 = stream.next_bits(every_batch_num);
        for i in 0..v1.len(){
            v1[i] = v1[i] ^ v2[i];
        }

        let mut q1 = q.clone();
        let q2 = stream.next_bits(every_batch_num);
        for i in 0..q.len(){
            q1[i] = q1[i] ^ q2[i];
        }

        let mbs = MBeaverBlock::gen(every_batch_num);
        let (mbs1, mbs2) = mbs.split();

        let (tx, mut rx) = mpsc::channel::<Vec<bool>>(4096);
        let (ty, mut ry) = mpsc::channel::<Vec<bool>>(4096);

        let task1 = party_extract(&v1, &q1, &mbs1.mbs, batch_size, true, &const_bdc_bits, tx, ry);
        let task2 = party_extract(&v2, &q2, &mbs2.mbs, batch_size, false, &const_bdc_bits, ty, rx);
        let (r1, r2) = futures::join!(task1, task2);
        (r1, r2)
    }

    #[tokio::test]
    async fn extract_works(){
        let batch_size = 3;
        let every_batch_num = 1 << batch_size;
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let q = stream.next_bits(batch_size);
        println!("q={:?}", q);
        let v = stream.next_bits(every_batch_num);
        println!("v={:?}", v);
        let (r1, r2) = do_extract(&v, &q, &mut stream).await;
        println!("r1={:?}", r1);
        println!("r2={:?}", r2);
        let mut pos: usize = 0;
        for i in 0..every_batch_num{
            if v[i]{
                pos = i;
                break;
            }
        }
        pos = every_batch_num - 1 - pos;
        println!("pos={}", pos);
        let mut result = q.clone();

        for i in 0..batch_size{
            let bit_i = ((pos>>i) & 0x1usize) == 1;
            result[i] = bit_i;
        }
        result.reverse();
        for i in 0..batch_size{
            assert_eq!(result[i] ^ q[i], r1[i] ^ r2[i]);
        }
    }

}