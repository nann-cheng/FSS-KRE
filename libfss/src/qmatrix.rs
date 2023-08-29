use crate::*;
use crate::prg::*;
use serde::Deserialize;
use serde::Serialize;
use crate::ring;

pub fn f_conv_matrix(q: &Vec<bool>, batch_size: usize) -> QMatrix {
    let every_batch_num: usize = 1 << batch_size;
    /*let q_num = bits_to_u8_BE(q); //indicate the location to get 1..1
    println!("q_num={}", q_num);
    let all_one_pos = q_num as usize;
    let  mut v = vec![false; (every_batch_num * every_batch_num) as usize];
    for i in 0..every_batch_num{
        v[i*every_batch_num + ((all_one_pos + i) % every_batch_num)] = true;
    }*/
    let mut const_bdc_bits = Vec::<bool>::new();
    for i in 0..every_batch_num {
        let cur_bits = u32_to_bits_BE(batch_size, (every_batch_num - 1 - i).try_into().unwrap());
        //convert int to {omega}-bits. q[0..{omega}]
        const_bdc_bits.extend(cur_bits);
    }

    let mut v = vec![false; every_batch_num * every_batch_num];
    for i in 0..every_batch_num {
        let mut pos_bits = vec![false; batch_size];
        for j in 0..batch_size {
            pos_bits[j] = q[j] ^ const_bdc_bits[i * batch_size + j];
        }
        let pos: usize = bits_to_u8_BE(&pos_bits).into();
        println!("i= {},pos = {}", i, pos);
        v[i * every_batch_num + (every_batch_num - pos - 1)] = true;
    }
    QMatrix {
        v: v,
        n: every_batch_num,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QMatrix {
    //This structure is defined for the ConvMatrix
    pub v: Vec<bool>,
    pub n: usize,
} // The offline data used in every batch.

impl QMatrix {
    //type Output = bool;
    pub fn locate(&self, i: usize, j: usize) -> bool {
        self.v[i * self.n + j]
    }

    pub fn Mutlocate(&mut self, i: usize, j: usize) -> &mut bool {
        &mut self.v[i * self.n + j]
    }

    pub fn split(&self) -> (Self, Self) {
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        let bv1 = stream.next_bits(self.n * self.n);
        let mut bv0 = self.v.clone();

        for i in 0..self.n * self.n {
            bv0[i] = bv0[i] ^ bv1[i];
        }

        (Self { v: bv0, n: self.n }, Self { v: bv1, n: self.n })
    }

    pub fn print(&self) {
        print!("[");
        for i in 0..self.n {
            for j in 0..self.n {
                if self.v[i * self.n + j] {
                    print!("1 ");
                } else {
                    print!("0 ");
                }
            }
            if i < self.n - 1 {
                print!("\n ");
            }
        }
        println!("]");
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QElmMatrix {
    //This structure is defined for the ConvMatrix
    pub v: Vec<RingElm>,
    pub n: usize,
} // The offline data used in every batch.

impl QElmMatrix {
    pub fn locate(&self, i: usize, j: usize) -> RingElm {
        self.v[i * self.n + j]
    }

    pub fn Mutlocate(&mut self, i: usize, j: usize) -> &mut RingElm {
        &mut self.v[i * self.n + j]
    }

    pub fn split(&self) -> (Self, Self) {
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);

        let mut v0 = Vec::<RingElm>::new();
        let mut v1 = Vec::<RingElm>::new();

        for i in 0..(self.n * self.n){
            let (el0, el1) = self.v[i].share();
            v0.push(el0);
            v1.push(el1);
        }
        

        (Self { v: v0, n: self.n }, Self { v: v1, n: self.n })
    }

    pub fn convertFromQMatrix(q: QMatrix) -> Self {
        let mut rv: Vec<RingElm> = Vec::new();

        for i in 0..q.n {
            for j in 0..q.n {
                if q.v[i * q.n + j] {
                    rv.push(RingElm::one());
                } else {
                    rv.push(RingElm::zero());
                }
            }
        }

        Self { v: rv, n: q.n }
    }

    pub fn print(&self) {
        print!("[");
        for i in 0..self.n {
            for j in 0..self.n {
                self.v[i * self.n + j].print();
            }
            if i < self.n - 1 {
                print!("\n ");
            }
        }
        println!("]");
    }
}