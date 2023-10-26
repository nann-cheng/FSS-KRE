use fss::*;
use fss::idpf::*;
use fss::RingElm;

pub mod bitwise_max;
pub mod bitwise_kre;
pub mod batch_max_proto;
pub mod batch_kre_proto;
pub mod max_ic_proto;

pub fn tree_eval_of_idpf(idpf: &IDPFKey<RingElm>, old_state: &EvalState, t_batch: &Vec<bool>, batch_size: usize, 
    tree_ind: usize, msk: bool, new_state: &mut Vec<EvalState>, beta: &mut Vec<RingElm>) {
    
    let mut vec_msk = vec![false; batch_size];
    vec_msk[tree_ind] = msk;
    let x_batch = bits_Xor(&vec_msk, t_batch);

    let (tmp_state, y_eval) =  idpf.eval_bit(&old_state, x_batch[tree_ind]);
    if tree_ind < (batch_size-1){
        tree_eval_of_idpf(idpf, &tmp_state, t_batch, batch_size, 
            tree_ind+1, true, new_state, beta);
        tree_eval_of_idpf(idpf, &tmp_state, t_batch, batch_size, 
            tree_ind+1, false, new_state, beta);
    }
    else{
        new_state.push(tmp_state);
        beta.push(y_eval);
    }
}