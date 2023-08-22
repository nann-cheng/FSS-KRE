use crate::*;
use crate::prg::*;
use serde::Deserialize;
use serde::Serialize;

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