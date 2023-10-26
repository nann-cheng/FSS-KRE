//use std::{f64::consts::LOG2_10, rc};
use fss::u32_to_bits;
use crate::{offline_data::offline_ic_max::MaxOffline_IC, mpc_platform::NetInterface};
use super::super::mpc_party::*;
use fss::{RingElm, beavertuple::BeaverTuple};

pub async fn max_ic(p: &mut MPCParty<MaxOffline_IC>, x: &Vec<RingElm>) ->RingElm{
    let mut x_share = x.clone();
    let mut x_len = x_share.len();  // a bug is fixed here. fzhang, 0921
    let mut t = Vec::<RingElm>::new();
    
    let is_server = p.netlayer.is_server;
    let mut ic_key_it = p.offlinedata.ic_key.iter();
    let mut alpha_it = p.offlinedata.ic_alpha.iter();
    let mut beaver_it = p.offlinedata.beavers.iter();
    /*Start: Debug info */
    let x_org = p.netlayer.exchange_ring_vec(x.clone()).await;
    println!("start:{:?}", x_org);
    /*End:   Debug info */
    while x_len > 1{
       
        t.clear();
        /**************************************START COMPUTE GREATERTHAN****************************/
        let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[i*2]-x[i*2+1]+alpha
        for i in 0..x_len/2{
            let x_diff = x_share[i*2] - x_share[i*2 + 1];
            let alpha = alpha_it.next().expect("No enough alpha to use.");
            let x_ic = x_diff +  alpha.clone();
            msg_share_x_ic.push(x_ic);
        } // prepare the message to exchange
        
        let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await; // exchange (x[i*2]-x[i*2+1] + alpha) to get n/2 points for GreaterThan function
        
        //compute n/2 y+b*(x-y), where b is the eval result of the GReaterThan function. It needs to exchange beaver tuples
        let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
        let mut msg_share_beaver = Vec::<RingElm>::new(); 
        for i in 0..x_len/2{
            let x_diff = x_share[i*2] - x_share[i*2 + 1];
            let ic_key = ic_key_it.next().expect("No enough ic_key.");
            let y_ic = ic_key.eval(&x_ics[i]); //GreaterThan 
            let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
            let open_value = beaver.mul_open(y_ic, x_diff);
            msg_share_beaver.push(open_value.0);
            msg_share_beaver.push(open_value.1);
            my_beavers.push(beaver);
        } 
        
        let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;

        for i in 0..x_len/2{
            let mul_result = my_beavers[i].mul_compute(is_server, &msg_beavers[i*2], &msg_beavers[i*2+1]);
            let max_of_two = x_share[i*2+1] + mul_result; // fix the way to compute greaterthan function. fzhang, 0921 update
            t.push(max_of_two); 
        }
        /**************************************END   COMPUTE GREATERTHAN****************************/    
        
        // deal with the last element if x_len is odd
        if x_len & 0x1usize == 1{
            t.push(x_share[x_len-1]);
        } 

        //update x_share
        x_share.clear();
        x_share.extend(t.clone());
        x_len = x_share.len();  //an important bug is fixed here. fzhang, 0921
        /*Start: Debug info */
        let x_layer = p.netlayer.exchange_ring_vec(x_share.clone()).await;
        println!("step:{:?}]", x_layer);
        /*End:   Debug info */
    }
    x_share[0]  
}

/****************************************************************************************************************************************************/
 /**We regard the array as a logic tree, where the i-th element's parent is the (i/2)-th element. By comparing every element to its parent,**********/ 
 /**We let the max value be the parent and the min value is the child.                                                                              */
 /****************************************************************************************************************************************************/
pub async fn heapify(p: &mut MPCParty<MaxOffline_IC>, x_share: &mut Vec<RingElm>) -> RingElm{
    let x_len = x_share.len();
    /*Start: Debug info */
    let x_org = p.netlayer.exchange_ring_vec(x_share.clone()).await;
    println!("start:{:?}", x_org);
    /*End:   Debug info */
    let is_server = p.netlayer.is_server;
    /*let mut ic_key_it = p.offlinedata.ic_key.iter();
    let mut alpha_it = p.offlinedata.ic_alpha.iter();
    let mut beaver_it = p.offlinedata.beavers.iter();*/
    let h = ((x_len+1) as f64).log(2 as f64).ceil() as usize; //the depth of the logic tree that has x_len nodes   
    let mut start_index = (1 << (h-1)) - 1; //the start index of the nodes to be handled
    let mut end_index = x_len - 1;  //the end index of the nodes to be handled
    
    for i in (1..h).rev(){
        //println!("h = {}, start_index = {}, end_index = {}", i, start_index, end_index);
        let mut lchildren = Vec::<usize>::new();
        let mut rchildren = Vec::<usize>::new();
        for j in start_index..=end_index{
            if j & 0x1 == 1{
                lchildren.push(j);
            } 
            else {
                rchildren.push(j);
            }
        }
      
        /*******************************START: COMPARE THE RIGHT CHILDREN TO THEIR PARENT****************************************/
        if !rchildren.is_empty() { //Here, fix a big bug that could lead the program exit early because the set is empty. update0922, fzhang
            let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[parent]-x[j]+alpha
            for j in rchildren.clone(){
                
                let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                let x_diff = x_share[parent] - x_share[j];
                //let alpha = alpha_it.next().expect("No enough alpha to use."); 
                let alpha = p.offlinedata.ic_alpha.pop().unwrap(); //update 0922
                let x_ic = x_diff +  alpha.clone();
                msg_share_x_ic.push(x_ic);
            }

            let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await;

            let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
            let mut msg_share_beaver = Vec::<RingElm>::new();
        
            let mut x_ics_it = x_ics.iter();  
            for j in  rchildren.clone(){
                let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                let x_diff = x_share[parent] - x_share[j];
                //let ic_key = ic_key_it.next().expect("No enough ic_key.");
                let ic_key = p.offlinedata.ic_key.pop().expect("No enough ic_key.");  //update0922,fzhang
                let x_ic = x_ics_it.next().expect("No enough x_ic.");
                let y_ic = ic_key.eval(x_ic);
                //let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
                let mut beaver = p.offlinedata.beavers.pop().expect("No enough beaver tuple."); //update0922,fzhang
                let half_beaver = beaver.mul_open(y_ic, x_diff);
                msg_share_beaver.push(half_beaver.0);
                msg_share_beaver.push(half_beaver.1);
                my_beavers.push(beaver);
            }

            let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;
            let mut mul_index= 0;
            for j in rchildren.clone(){
                let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                let mul_result = my_beavers[mul_index].mul_compute(is_server, &msg_beavers[mul_index*2] ,&msg_beavers[mul_index*2+1]);
                mul_index += 1;
                let max_of_two = x_share[j] + mul_result;
                let sum_of_two = x_share[parent] + x_share[j];
                x_share[parent] = max_of_two;
                x_share[j] = sum_of_two - max_of_two;
            }
        }
        /*******************************END: COMPARE THE RIGHT CHILDREN TO THEIR PARENT******************************************/

        /*******************************START: COMPARE THE LEFT CHILDREN TO THEIR PARENT*******************************************/
        if !lchildren.is_empty() {
            let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[parent]-x[j]+alpha
            for j in lchildren.clone(){
                let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                let x_diff = x_share[parent] - x_share[j];
                //let alpha = alpha_it.next().expect("No enough alpha to use.");
                let alpha = p.offlinedata.ic_alpha.pop().expect("No enough ic_alpha."); //update0922,fzhang
                let x_ic = x_diff +  alpha.clone();
                msg_share_x_ic.push(x_ic);
            }

            let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await;

            let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
            let mut msg_share_beaver = Vec::<RingElm>::new();
        
            let mut x_ics_it = x_ics.iter();  
            for j in  lchildren.clone(){
                let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                let x_diff = x_share[parent] - x_share[j];
                //let ic_key = ic_key_it.next().expect("No enough ic_key.");
                let ic_key = p.offlinedata.ic_key.pop().expect("No enough ic_key."); //update0922,fzhang
                let x_ic = x_ics_it.next().expect("No enough x_ic.");
                let y_ic = ic_key.eval(x_ic);
                //let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
                let mut beaver = p.offlinedata.beavers.pop().expect("No enough beaver tuple."); //update0922,fzhang
                let half_beaver = beaver.mul_open(y_ic, x_diff);
                msg_share_beaver.push(half_beaver.0);
                msg_share_beaver.push(half_beaver.1);
                my_beavers.push(beaver);
            }

            let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;
            let mut mul_index= 0;
            for j in lchildren.clone(){
                let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                let mul_result = my_beavers[mul_index].mul_compute(is_server, &msg_beavers[mul_index*2] ,&msg_beavers[mul_index*2+1]);
                mul_index += 1;
                let max_of_two = x_share[j] + mul_result;
                let sum_of_two = x_share[parent] + x_share[j]; //fixed a mistake
                x_share[parent] = max_of_two;
                x_share[j] = sum_of_two - max_of_two;
            }
        }
        /*******************************END: COMPARE THE LEFT CHILDREN TO THEIR PARENT*********************************************/
        end_index = start_index - 1;
        start_index = (1 << (i - 1)) - 1;

        /*Start: Debug info */
        let x_layer = p.netlayer.exchange_ring_vec(x_share.clone()).await;
        println!("layer{}:{:?}", i, x_layer);
        /*End:   Debug info */
    }
    return x_share[0];
}

pub async fn heap_sort(p: &mut MPCParty<MaxOffline_IC>, x_share: &mut Vec<RingElm>){
    let x_length = x_share.len();
    //let times_heapify = x_share.len() - 1;
    let is_server = p.netlayer.is_server;

    /*Start: Debug info */
    let x_org = p.netlayer.exchange_ring_vec(x_share.clone()).await;
    println!("start:{:?}", x_org);

    for heap_cnt in 0..x_length{
        let x_len = x_length - heap_cnt; 
        let h = ((x_len+1) as f64).log(2 as f64).ceil() as usize; //the depth of the logic tree that has x_len nodes   
        let mut start_index = (1 << (h-1)) - 1; //the start index of the nodes to be handled
        let mut end_index = x_len - 1;  //the end index of the nodes to be handled
        println!("****************start heapify**********************");
        //println!("start_index = {}, end_index = {}", start_index, end_index);
        for i in (1..h).rev(){
            //println!("h = {}, start_index = {}, end_index = {}", i, start_index, end_index);
            let mut lchildren = Vec::<usize>::new();
            let mut rchildren = Vec::<usize>::new();
            for j in start_index..=end_index{
                if j & 0x1 == 1{
                    lchildren.push(j);
                } 
                else {
                    rchildren.push(j);
                }
            }
        
            /*******************************START: COMPARE THE RIGHT CHILDREN TO THEIR PARENT****************************************/
            if !rchildren.is_empty(){
                //println!("handel {:?}", rchildren);
                let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[parent]-x[j]+alpha
                for j in rchildren.clone(){  //rchild from lchild
                    
                    let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                    let x_diff = x_share[parent] - x_share[j];
                    //let alpha = alpha_it.next().expect("No enough alpha to use."); 
                    let alpha = p.offlinedata.ic_alpha.pop().unwrap(); //update 0922
                    let x_ic = x_diff +  alpha.clone();
                    msg_share_x_ic.push(x_ic);
                }

                let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await;

                let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
                let mut msg_share_beaver = Vec::<RingElm>::new();
            
                let mut x_ics_it = x_ics.iter();  
                for j in  rchildren.clone(){
                    let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                    let x_diff = x_share[parent] - x_share[j];
                    //let ic_key = ic_key_it.next().expect("No enough ic_key.");
                    let ic_key = p.offlinedata.ic_key.pop().expect("No enough ic_key.");  //update0922,fzhang
                    
                    let x_ic = x_ics_it.next().expect("No enough x_ic.");
                    let y_ic = ic_key.eval(x_ic);
                    //let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
                    let mut beaver = p.offlinedata.beavers.pop().expect("No enough beaver tuple."); //update0922,fzhang
                    let half_beaver = beaver.mul_open(y_ic, x_diff);
                    msg_share_beaver.push(half_beaver.0);
                    msg_share_beaver.push(half_beaver.1);
                    my_beavers.push(beaver);
                }

                let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;
                let mut mul_index= 0;
                for j in rchildren.clone(){
                    let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                    let mul_result = my_beavers[mul_index].mul_compute(is_server, &msg_beavers[mul_index*2] ,&msg_beavers[mul_index*2+1]);
                    mul_index += 1;
                    let max_of_two = x_share[j] + mul_result;
                    let sum_of_two = x_share[parent] + x_share[j];
                    x_share[parent] = max_of_two;
                    x_share[j] = sum_of_two - max_of_two;
                }
            }
            /*******************************END: COMPARE THE RIGHT CHILDREN TO THEIR PARENT******************************************/

            /*******************************START: COMPARE THE LEFT CHILDREN TO THEIR PARENT*******************************************/
            if !lchildren.is_empty(){
                //println!("handel {:?}", lchildren);
                let mut msg_share_x_ic = Vec::<RingElm>::new(); // to store the masked value that is  x[parent]-x[j]+alpha
                for j in lchildren.clone(){
                    let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                    let x_diff = x_share[parent] - x_share[j];
                    //let alpha = alpha_it.next().expect("No enough alpha to use.");
                    let alpha = p.offlinedata.ic_alpha.pop().expect("No enough ic_alpha."); //update0922,fzhang
                    let x_ic = x_diff +  alpha.clone();
                    msg_share_x_ic.push(x_ic);
                }

                let x_ics = p.netlayer.exchange_ring_vec(msg_share_x_ic).await;

                let mut my_beavers = Vec::<BeaverTuple>::new(); // to store n/2 beaver tuples for multiplications
                let mut msg_share_beaver = Vec::<RingElm>::new();
            
                let mut x_ics_it = x_ics.iter();  
                for j in  lchildren.clone(){
                    let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                    let x_diff = x_share[parent] - x_share[j];
                    //let ic_key = ic_key_it.next().expect("No enough ic_key.");
                    let ic_key = p.offlinedata.ic_key.pop().expect("No enough ic_key."); //update0922,fzhang
                    
                    let x_ic = x_ics_it.next().expect("No enough x_ic.");
                    let y_ic = ic_key.eval(x_ic);
                    //let mut beaver = beaver_it.next().expect("No enough beaver tuple.").clone();
                    let mut beaver = p.offlinedata.beavers.pop().expect("No enough beaver tuple."); //update0922,fzhang
                    let half_beaver = beaver.mul_open(y_ic, x_diff);
                    msg_share_beaver.push(half_beaver.0);
                    msg_share_beaver.push(half_beaver.1);
                    my_beavers.push(beaver);
                }

                let msg_beavers = p.netlayer.exchange_ring_vec(msg_share_beaver).await;
                let mut mul_index= 0;
                for j in lchildren.clone(){
                    let parent = (j+1) / 2 - 1; //fixed the way to compute the parent's index. fzhang, 0921
                    let mul_result = my_beavers[mul_index].mul_compute(is_server, &msg_beavers[mul_index*2] ,&msg_beavers[mul_index*2+1]);
                    mul_index += 1;
                    let max_of_two = x_share[j] + mul_result;
                    let sum_of_two = x_share[parent] + x_share[j]; //fixed a mistake
                    x_share[parent] = max_of_two;
                    x_share[j] = sum_of_two - max_of_two;
                }
            }
            /*******************************END: COMPARE THE LEFT CHILDREN TO THEIR PARENT*********************************************/
            end_index = start_index - 1;
            start_index = (1 << (i - 1)) - 1;
        }
         
        /*****************************************************************************************************************************/
        // update after each heapify call
        //1. swap the x_share[0] and x_share[x_len], then put the max to the end of the list
            let temp_elem = x_share[0].clone();
            x_share[0] = x_share[x_len - 1];
            x_share[x_len-1] = temp_elem; 
        /*****************************************************************************************************************************/
        /*Start: Debug info */
        let x_layer = p.netlayer.exchange_ring_vec(x_share.clone()).await;
        println!("sort_i{:?}", x_layer);
        println!("****************end  heapify**********************");
        /*End:   Debug info */
    }
}

//assume x_share is in incremental order already. This function is to extract the k-th maxium value
pub async fn extract_kmax(p: &mut MPCParty<MaxOffline_IC>, x_share: &Vec<RingElm>, k_share: RingElm) -> RingElm{
    let is_server = p.netlayer.is_server;
    let x_len = x_share.len();
    /************************************START: step1 prepare auxiliary array y_share ************************************************/
    let mut y_share = vec![RingElm::from(0); x_len];
    if(is_server){
        for i in  0..x_len{
            y_share[i] = RingElm::from((x_len - i) as u32) - k_share;
        }
    }
    /************************************END:step1 prepare auxiliary array y_share ***************************************************/

    /************************************START: step2 z_share = x_share times y_share and check z ?= 0 *******************************/
    let mut xmsg_for_beaver_share = Vec::<RingElm>::new();
    let mut my_beavers = Vec::<BeaverTuple>::new();
    
    for i in 0..x_len{
        let mut beaver = p.offlinedata.beavers.pop().expect("No enough beavers.");
        let half_beaver = beaver.mul_open(x_share[i].clone(), y_share[i]);
        my_beavers.push(beaver);
        xmsg_for_beaver_share.push(half_beaver.0); //the 1st element
        xmsg_for_beaver_share.push(half_beaver.1); //the 2nd element
    }
    
    let xmsg_for_beaver = p.netlayer.exchange_ring_vec(xmsg_for_beaver_share).await;
    
    let mut z_share = Vec::<RingElm>::new();
    for i in 0..x_len{
        let z_share_i = my_beavers[i].mul_compute(is_server, &xmsg_for_beaver[2*i], &xmsg_for_beaver[2*i+1]);
        z_share.push(z_share_i);
    } 
    /************************************END:   step2 z_share = x_share times y_share and check z ?= 0 *******************************/
    
    /************************************START: step3 check z ?= 0 *******************************************************************/
    let mut xmsg_for_zc_share = Vec::<RingElm>::new();
    
    for i in 0..x_len{ 
        //let key_zc = p.offlinedata.zc_key.pop().expect("No enough zc_key.");
        let alpha_zc = p.offlinedata.zc_alpha.pop().expect("No enough zc_alpha");
        let x_zc = z_share[i] + alpha_zc;
        xmsg_for_zc_share.push(x_zc);  //the first element
    }

    let xmsg_for_zc = p.netlayer.exchange_ring_vec(xmsg_for_zc_share).await;

    let mut z_share = Vec::<RingElm>::new();
    for i in 0..x_len{
        let key_zc = p.offlinedata.zc_key.pop().expect("No enough zc_key.");
        let x_zc = xmsg_for_zc[i];
        let mut vec_eval = vec![false;32usize];
        let num_eval = x_zc.to_u32();
        match num_eval {
            Some(numeric) => vec_eval = u32_to_bits(32usize,numeric),
            None      => println!( "u32 Conversion failed!!" ),
        }
        let y_zc = key_zc.eval(&vec_eval);
        z_share.push(y_zc);
    }
    /************************************END:   step3: check z ?= 0*******************************************************************/

    /************************************START: step4 compute the inner product of x_share and z_share********************************/
    let mut my_beaver_for_inner_product = Vec::<BeaverTuple>::new();
    let mut xmsg_for_inner_product_share = Vec::<RingElm>::new();
    for i in 0..x_len{
        let mut beaver = p.offlinedata.beavers.pop().expect("No enough beaver tuples.");
        let half_beaver = beaver.mul_open(x_share[i].clone(), z_share[i]);
        xmsg_for_inner_product_share.push(half_beaver.0);
        xmsg_for_inner_product_share.push(half_beaver.1);
        my_beaver_for_inner_product.push(beaver);
    }

    let xmsg_for_inner_product = p.netlayer.exchange_ring_vec(xmsg_for_inner_product_share).await;

    let mut kmax_share = RingElm::from(0);
    for i in 0..x_len{
        let k_max_share_i = my_beaver_for_inner_product[i].mul_compute(is_server, &xmsg_for_inner_product[i*2], &xmsg_for_inner_product[i*2+1]);
        kmax_share = kmax_share + k_max_share_i;
    }
    /************************************START: step4 compute the inner product of x_share and z_share********************************/
    /***************************Debug Info ****************************/
    let msg_kmax_share = [kmax_share].to_vec();
    let msg_kmax = p.netlayer.exchange_ring_vec(msg_kmax_share).await;
    println!("kmax={:?}", msg_kmax);
    /***************************Debug Info ****************************/
    kmax_share
}