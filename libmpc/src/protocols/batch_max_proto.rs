use fss::*;
use fss::idpf::*;
use fss::RingElm;
use fss::BinElm;
use fss::mbeaver::MBeaver;
use fss::mbeaver::product;

use crate::offline_data::offline_batch_max::BatchMaxOffline;
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
pub async fn batch_max(p: &mut MPCParty<BatchMaxOffline>, x_bits: &Vec<bool>, batch_size: usize) ->Vec<bool>{
    let m: usize = p.m;
    let n = p.n;
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
        let mut V = Vec::<RingElm>::new();
        let mut tmp_state = Vec::<Vec::<EvalState>>::new();
        for i in 0..m{
            let tmp_state_i = Vec::<EvalState>::new();
            //tmp_state_i.push(idpf_state[i].clone()); // initialize the j-th idpf's state 
            tmp_state.push(tmp_state_i);
        }   //prepare for m state vectors, each of which contains  {\tao} state

        let index_start = block_order * batch_size;
        let index_end: usize = (block_order+1) * batch_size; //change them out from the loop
        for i in 0..every_batch_num{ // for every \{tao} - j
            let mut v_item = RingElm::from(0);
            for j in 0..m{
                let x_idpf = bits_Xor(&const_bdc_bits[i*batch_size..(i+1)*batch_size].to_vec(), &t[(j*n+index_start)..(j*n+index_end)].to_vec());//line8
                //let (state_new, beta) = p.offlinedata.base.k_share[j].eval_bit(&old_state[j], &x_idpf[0..batch_size]);
                let old_state = idpf_state[j].clone(); // The initial state of the m-th idpf is idpf_state[i]
                let (state_new, beta) = batch_eval_of_idpf(&p.offlinedata.base.k_share[j], &old_state, &x_idpf, batch_size);
                tmp_state[j].push(state_new); //the j-th state for the k-th idpf
                v_item.add(&beta);
                //let (state, beta) = p.offlinedata.base.k_share[k].eval_bit(state, dir)
            }
            V.push(v_item);
        }
        /********************************************************  END  : Line6-14: Compute vector V   *******************************/
        /*****************************************************************************************************************************/
        
        /*****************************************************************************************************************************/
        /********************************************************  START: F_BatchMax  Line15 *****************************************/
        let f_batch_max = {
            /***********************************   START: Line3-5 Compute idpf-s    **********************************/
            let q_share = &p.offlinedata.base.qb_share[block_order*batch_size..(block_order+1)*batch_size];
            let zc_a_share = &p.offlinedata.zc_a_share[block_order*every_batch_num..(block_order+1)*every_batch_num];
            let zc_k_share = &p.offlinedata.zc_k_share[block_order*every_batch_num..(block_order+1)*every_batch_num]; 
            //update0817: it needs to call f_znc {\tao} times in every block  
            let M = &p.offlinedata.qmatrix_share[block_order];
            let mbs = &p.offlinedata.mbeavers[block_order].mbs;
            let mut it_bbeaver = p.offlinedata.binary_beavers.iter();
            M.print();
            let mut x_f_nzc_share = Vec::<RingElm>::new();
            
            for i in 0..every_batch_num{
                let mut x_item = V[i].clone();
                x_item.add(&zc_a_share[i]);
                x_f_nzc_share.push(x_item);
            } // Prepare exchange 2^batch ring;

            //exchange message
            let x_f_nzc = p.netlayer.exchange_ring_vec(x_f_nzc_share).await;
        
            let mut cmp = Vec::<bool>::new();
            for i in 0..every_batch_num{
                let mut vec_eval = vec![false;32usize];
                let num_eval = x_f_nzc[i].to_u32();
                match num_eval {
                    Some(numeric) => vec_eval = u32_to_bits(32usize,numeric),
                    None      => println!( "u32 Conversion failed!!" ),
                }
        
                let y_fnzc: BinElm = zc_k_share[i].eval(&vec_eval);
                // println!("y_fnzc={:?}", y_fnzc);
                let mut cmp_i = y_fnzc.to_Bool();
                if is_server{
                    cmp_i = !cmp_i;
                }
                cmp.push(cmp_i);
            } //Line3-5, compute idpf-s
            /***********************************   END:   Line3-5 Compute idpf-s    **********************************/
            println!("pre-ordered cmp[{}]={:?}", block_order, cmp);
            /********************************   START:  Line2 order cmp  *********************************************/

            let mut exchange_msg_1 = Vec::<bool>::new();
            let mut consumed_beavers = Vec::<MBeaver>::new(); //store every consumed beaver tuple 
            //every item in cmp and M will take part in {\tao} times multilication
             
            for i in 0..every_batch_num{ // for every item in cmp and every line in M 
                for j in 0..every_batch_num{ //prepare for cmp[j] * M[i][j]
                    let beaver_new = it_bbeaver.next().unwrap();
                    let delta_0 = cmp[j] ^ beaver_new[0]; //d_share =  v_{alpha} - a
                    let delta_1 = M.locate(i, j) ^ beaver_new[1]; //e_share = v_{beta} - b
                    exchange_msg_1.push(delta_0);  //push d_share
                    exchange_msg_1.push(delta_1);  //push e_share
                    consumed_beavers.push(beaver_new.clone()); //store the consumed beaver
                }
            }
            
            let delta = p.netlayer.exchange_bool_vec(exchange_msg_1).await;     
            let mut index_delta = 0;
            for i in 0..cmp.len(){
                cmp[i] = false;
            }
          
            for i in 0..every_batch_num{ // for every item in cmp and every line in M 
                for j in 0..every_batch_num{ //prepare for cmp[j] * M[i][j]
                    let beaver_new = consumed_beavers[index_delta].clone();
                    let delta_0 = delta[index_delta*2];     //obtain d
                    let delta_1 = delta[index_delta*2+1];   //obtain e
                    let mut delta = Vec::<bool>::new();
                    delta.push(delta_0);
                    delta.push(delta_1);
                    index_delta += 1;
                    let bool_product = product(&delta, &beaver_new, is_server).unwrap();
                    cmp[i] = cmp[i] ^ bool_product;
                }
            } //finished test already
            println!("ordered cmp[{}]={:?}", block_order, cmp);
            /******************************** END:    Line2 order cmp  *********************************************/

            /******************************** START:  Line11 Compute bt  *******************************************/
            //let mbs = &p.offlinedata.mbeavers[block_order].mbs;
            let mut exchange_msg_2 = Vec::<bool>::new();

            for i in 1..every_batch_num{ // {\tao} - 2 mulitiplications
                let mut item = cmp[i] ^ mbs[i-1][0]; //the index of mbs is i-1
                exchange_msg_2.push(item);
                for j in 0..i{ //1-cmp[j]
                    if is_server{
                        item = cmp[j] ^ mbs[i-1][1+j];
                    }
                    else{
                        item = !cmp[j] ^ mbs[i-1][1+j];
                    }
                    exchange_msg_2.push(item);
                }
            }

            let delta = p.netlayer.exchange_bool_vec(exchange_msg_2).await;

            let mut bt = Vec::<bool>::new();
            bt.push(cmp[0]);
            
            index_delta = 0;
            for i in 1..every_batch_num{ // in the i-th round, obtain (i+1) elements 
                let items = delta[index_delta..=index_delta+i].to_vec();
                let bt_i = product(&items, &mbs[i-1], is_server).unwrap(); //change the index of mbs from i to i-1
                bt.push(bt_i);
                index_delta += i;
                index_delta += 1;
            }// finished test already
            /******************************** END:    Line11 Compute bt  *******************************************/

            /******************************** START:  Line14 Compute y_i  ******************************************/
            // Here, there is only one zero in bt
            let mut y = Vec::<bool>::new();
            for i in 0..batch_size{
                y.push(q_share[i]);
            }

            for i in 0..every_batch_num{
                let const_i_bits = &const_bdc_bits[i*batch_size..(i+1)*batch_size];
                for j in 0..batch_size{
                    y[j] = y[j] ^ (bt[i] && const_i_bits[j]); //changed index-s
                }
            }
            /******************************** END:    Line14 Compute y_i  ******************************************/
            println!("Pre-y[{}]={:?}", block_order, y);
            let result = p.netlayer.exchange_bool_vec(y).await;
            result
        };       
        /********************************************************  END:   F_BatchMax Line15  *****************************************/
        /*****************************************************************************************************************************/
        println!("Reveal-y[{}]={:?}", block_order, f_batch_max);
        let mut path_eval = 0; // define which path is used
        for i in 0..batch_size{
            path_eval = path_eval << 1;
            if f_batch_max[i]{
                path_eval += 1;
            }
        }  //line 17: rebuild the numerical max num

        for i in 0..m{
            idpf_state[i] = tmp_state[i][path_eval].clone(); 
        } // Line18-20: update the idpf-s

        if is_server{ //Here, I fixed a big bug, update:0819
            for i in 0..batch_size{
                cmp_bits[i+block_order*batch_size] = f_batch_max[i] ^ p.offlinedata.base.qb_share[i+block_order*batch_size]; // A big change here, last version forgot xor the q_share 
            }
        }
        else{
            for i in 0..batch_size{
                cmp_bits[i+block_order*batch_size] = p.offlinedata.base.qb_share[i+block_order*batch_size]; // A big change here, last version forgot xor the q_share 
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

    use crate::offline_data::offline_batch_max::{f_conv_matrix, QMatrix, MBeaverBlock};
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