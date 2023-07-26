fn KRE1(INPUT_SIZE:usize, INPUT_BITS:usize) -> String {
    let start = Instant::now();

    //I. Offline-phase: Generating random DPF keys & daBits
    let seed = PrgSeed::random();
    let mut stream = FixedKeyPrgStream::new();
    stream.set_key(&seed.key);

    //Offline-Step-1. Basic Parameters
    let plaintext:u32 = INPUT_SIZE as u32;
    let mut kth_index = FieldElm::from(plaintext/2);
    // let mut kth_index = FieldElm::from(500u32);
    // let mut kth_index = FieldElm::from(INPUT_SIZE as u32);
    let (kth_0,kth_1) = kth_index.share();
    let mut left_branch = FieldElm::zero();


    let fix_betas = FieldElm::from(1u32).to_vec(INPUT_BITS);
    let r_bits = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    
    //Offline-Step-2. Random I-DPFs
    let mut dpf_0: Vec<DPFKey<FieldElm>> = Vec::new();
    let mut dpf_1: Vec<DPFKey<FieldElm>> = Vec::new();
    for i in 0..INPUT_SIZE{
        let alpha = &r_bits[i*INPUT_BITS..(i+1)*INPUT_BITS];
        let (k0, k1) = DPFKey::gen(&alpha, &fix_betas);
        dpf_0.push(k0);
        dpf_1.push(k1);
    }

    let r_bits_0 = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let r_bits_1 = bits_Xor(&r_bits, &r_bits_0);

    //Offline-Step-3. Random daBits for masking
    let q_boolean = stream.next_bits(INPUT_BITS);
    // println!("q_boolean is: {} ",vec_bool_to_string(&q_boolean));
    let q_boolean_0 = stream.next_bits(INPUT_BITS);
    let q_boolean_1 = bits_Xor(&q_boolean, &q_boolean_0);


    let mut q_numeric = Vec::new();
    let mut q_numeric_0 = Vec::new();
    let mut q_numeric_1 = Vec::new();
    for i in 0..INPUT_BITS{
        let mut q_i = FieldElm::zero();
        if q_boolean[i]{
            q_i = FieldElm::from(1u32);
        }
        let (q_i_0,q_i_1) = q_i.share();
        q_numeric.push(q_i);
        q_numeric_0.push(q_i_0);
        q_numeric_1.push(q_i_1);
    }
    
    // println!("{:.5?} seconds for offline phase.", start.elapsed());

    let online_start = Instant::now();
    //II. Online-phase: prefix-querying
    //Online-step-1. Random number input with boolean sharing form
    let x_bits = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let x_bits_0 = stream.next_bits(INPUT_BITS*INPUT_SIZE);
    let x_bits_1 = bits_Xor(&x_bits, &x_bits_0);
    let mut v_val = FieldElm::from(INPUT_SIZE as u32);

    //Debug: print these random input u32 values
    // print_input_statistics(INPUT_SIZE,INPUT_BITS,&x_bits);

    //Online-step-2. Get masking random numbers r_i^q^x_i
    let mut mask_bits = Vec::new();
    let mut prefix_bits = vec![false;INPUT_SIZE*INPUT_BITS];

    for i in 0..INPUT_SIZE{
        let v =  bits_Xor(&r_bits[i*INPUT_BITS..(i+1)*INPUT_BITS].to_vec(), &q_boolean);
        let mut v1  = bits_Xor(&x_bits[i*INPUT_BITS..(i+1)*INPUT_BITS].to_vec(), &v);
        mask_bits.append(&mut v1);
    }

    //Online-step-3. Start bit-by-bit prefix query
    let mut old_state0 = Vec::new();
    let mut new_state0 = Vec::new();

    let mut old_state1 = Vec::new();
    let mut new_state1 = Vec::new();
    for j in 0..INPUT_SIZE{
        old_state0.push( dpf_0[j].eval_init() );
        old_state1.push( dpf_1[j].eval_init() );
    }


    let mut cmp_bits= Vec::new();
    for i in 0..INPUT_BITS{
        let mut mu_0 = FieldElm::zero();
        let mut mu_1 = FieldElm::zero();

            if i==0{
                for j in 0..INPUT_SIZE{
                    let new_bit = mask_bits[j*INPUT_BITS+i];

                    let (state_new0, word0) = dpf_0[j].eval_bit(&old_state0[j], new_bit);
                    mu_0.add(&word0);

                    let (state_new1, word1) = dpf_1[j].eval_bit(&old_state1[j], new_bit);
                    mu_1.add(&word1);


                    new_state0.push(state_new0);
                    new_state1.push(state_new1);
                }
            }else{
                if mask_bits[i-1] != prefix_bits[i-1]//do evaluation twice
                {
                    for j in 0..INPUT_SIZE{
                        let new_bit = mask_bits[j*INPUT_BITS+i];

                        let (state_new0, word0) = dpf_0[j].eval_bit(&old_state0[j], prefix_bits[j*INPUT_BITS+i-1]);
                        old_state0[j] = state_new0.clone();

                        let (state_new00, word00) = dpf_0[j].eval_bit(&state_new0, new_bit);
                        mu_0.add(&word00);

                        new_state0[j] = state_new00.clone();

                        let (state_new1, word1) = dpf_1[j].eval_bit(&old_state1[j], prefix_bits[j*INPUT_BITS+i-1]);
                        old_state1[j] = state_new1.clone();
                        let (state_new11, word11) = dpf_1[j].eval_bit(&state_new1, new_bit);
                        mu_1.add(&word11);
                        new_state1[j] = state_new11.clone();
                    }
                }
                else{//do evaluation once
                    for j in 0..INPUT_SIZE{
                        let new_bit = mask_bits[j*INPUT_BITS+i];

                        let (state_new0, word0) = dpf_0[j].eval_bit(&new_state0[j], new_bit);
                        mu_0.add(&word0);

                        let (state_new1, word1) = dpf_1[j].eval_bit(&new_state1[j], new_bit);
                        mu_1.add(&word1);

                        old_state0[j] = new_state0[j].clone();
                        old_state1[j] = new_state1[j].clone();

                        new_state0[j] = state_new0.clone();
                        new_state1[j] = state_new1.clone();
                    }
                }
            }

        mu_0.add(&mu_1);//communication
        // mu_0.print("mu value");


        //Selection-Protocol Line-5 update left branch values
        let mut left = q_numeric[i].clone();
        left.mul(&mu_0);

        let mut right = v_val.clone();
        right.sub(&mu_0);
        let mut right_0 = FieldElm::one();
        right_0.sub(&q_numeric[i]);
        right_0.mul(&right);
        left.add(&right_0);
        left_branch = left;

        
        //Online-step-4. Execuate LessThan check between L and T
        let cmp_i = left_branch < kth_index;

        for j in 0..INPUT_SIZE{
            prefix_bits[j*INPUT_BITS+i] = !cmp_i ^ x_bits[j*INPUT_BITS+i] ^ r_bits[j*INPUT_BITS+i];
        }
        cmp_bits.push(!cmp_i);//This is to represent the maximum value

        //Max-Protocol Line-8 refresh V value & kth value
        if cmp_i{
            kth_index.sub(&left_branch);
            v_val.sub(&left_branch);
        }else{
           v_val = left_branch; 
        }
    }

    // format!("{:.4?}",online_start.elapsed().as_millis())
    format!("{}",online_start.elapsed().as_millis())
    // println!{"K_th value: {}",u32_to_bits_BE(INPUT_BITS, bits_to_u32_BE(&cmp_bits))};
    // println!("{:.5?} for online phase.", online_start.elapsed());
}