use crate::*;
use crate::prg::*;
use std::ops::{Index, IndexMut};
use std::error::Error;
use serde::{Serialize, Deserialize};

//use bincode::{Serializer, Deserializer};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MBeaver{
    pub a: Vec<bool>,
    pub n: usize
}

impl MBeaver{
    pub fn gen(n: usize) ->Self{
        let N: usize = (1<<n) - 1;
        let mut rv = Vec::<bool>::new();
        
        for _ in 0.. N{
            rv.push(false);
        }

        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let rd_bit = stream.next_bits(n);
        for i in 0..=n-1{
            let loc = (1<<i) - 1; // convert the loc to index by minus 1
            rv[loc] = rd_bit[i];
            
        }// initialize the n atom-s which is in the 2^i-1 location
                
        // assign the other locations
        
        for i in 2..N{
            let mut j = i+1; //j is the location

            /*Decomposite j to a binary stream, and if the bit is equal to 1, then operate rv */
            let mut bits = Vec::<usize>::new();
            while j != 0{
                bits.push(j%2);
                j = j / 2;
            }
            //println!("bits={:?}", bits);
            let mut e = true;
            let mut bit_loc = bits.len();
            while !bits.is_empty(){
                if bits.pop().unwrap() == 1{
                    e = e && rv[(1<<(bit_loc-1)) - 1];
                }
                bit_loc -= 1;
            }
            rv[i] = e;
        } 
        //println!("{:?}", rv);
        MBeaver { a: rv, n: n}
    }

    pub fn split(&self) -> (MBeaver, MBeaver){
        let len = self.a.len();
        let mut rv1 = self.a.clone();
        //let mut rv2 = Vec::<bool>::new();
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let rv2 = stream.next_bits(len);
        for i in 0..=len-1{
           rv1[i] = rv1[i] ^ rv2[i];
        }

        let bv1 = MBeaver{a: rv1, n: self.n};
        let bv2 = MBeaver{a: rv2, n: self.n};
        (bv1, bv2)
    }

    pub fn extendfrom(v: Vec<bool>)->Self{
        let n = v.len();
        let N: usize = (1<<n) - 1;
        let mut rv = Vec::<bool>::new();
        //let result = false;
        for _ in 0.. N{
            rv.push(false);
        }
        for i in 0..=n-1{
            let loc = (1<<i) - 1; // convert the loc to index by minus 1
            rv[loc] = v[i];
            //rv[loc] = bool::from((i+1) as u32);
        }// initialize the n atom-s which is in the 2^i-1 location

        rv[2] = rv[0] && rv[1];

        for i in 2..n{
            let loc = (1<<i) - 1; //the i-th atom item.
            let e = rv[loc]; 
        
            for j in 1..=loc{
                //rv[loc+j] = rv[j-1].clone(); 
                rv[loc+j] = rv[j-1] && e;
            }
        }  
        //println!("rv = {:?}", rv);
        MBeaver { a: rv, n: n }
    }

}

impl Index<usize> for MBeaver {
    type Output = bool;

    fn index(& self, index: usize) -> & bool {
        let loc = 1<<index;
        & self.a[loc-1]
    }
}

impl IndexMut<usize> for MBeaver {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let loc = 1<<index;
        &mut self.a[loc-1]
    }
}

//delta is extend from the opened ring values, and b is the beaver tuples shared by the two parites in offline phase
pub fn Muls(delta: &MBeaver, b: &MBeaver, is_server: bool)->Result<bool, Box<dyn Error>>{
    if delta.a.len() != b.a.len() || delta.n != b.n{
        //print!("{}={}={}={}", delta.a.len(), b.a.len(), delta.n, b.n);
        return Err("the two beaver tuples don't match".into());
    }
    //println!("delta="); print_bool_vec(&delta.a);
    //println!("b="); print_bool_vec(&b.a);
    let n = delta.n;
    let N = (1<<n) - 1; //0x0..01..1(n 1-s)
    //println!("n={}, N={}", n, N);
    let mut r: bool = false;
    if is_server{
        r = r ^ delta.a[N-1];
    }
    r = r ^ b.a[N-1];
    //println!("r = {}", r);
    for i in 1..N{
        // unit denotes a term of the + 
        let mut unit = delta.a[i-1];
        let index = (!i) & N;
        //println!("i={}, j={}", i, index);
        //unit.mul(&b.a[index-1]);
        unit = unit && b.a[index-1];
        //r.add(&unit);
        r = r ^ unit;
        //println!("r{} = {}", i, r);
    }

    Ok(r)
}

pub fn product(delta: &Vec<bool>, b: &MBeaver, is_server: bool)->Result<bool, Box<dyn Error>>{
    let n = delta.len();
    let N = b.a.len();

    if N != (1<<n) -1{
        return Err("the two beaver tuples don't match".into());
    }
    let delta_cy = delta.clone();
    let d = MBeaver::extendfrom(delta_cy);
    let r = Muls(&d, b, is_server);
    r
}

fn print_bool_vec(bv: &Vec<bool>){
    print!("(");
    for i in 0..bv.len(){
        if bv[i]{
            print!("1,");
        }
        else{
            print!("0,");
        }
    }
    println!(")");
}
mod test{
    use crate::mbeaver::*;    
    
    #[test]
    fn beavers_gen_works(){
        let beaver = MBeaver::gen(5);
    }
   
    #[test]
    fn beaver_split_works()
    {
        let n = 4;
        let b = MBeaver::gen(n);
        let (b1, b2) = b.split();

        let mut r1 = b1.a.clone();
        let r2 = b2.a.clone();

        for i in 0..r1.len(){
            r1[i] = r1[i] ^ r2[i];
        }

        for i in 0..r1.len(){
            assert_eq!(r1[i], b.a[i])
        }
    }

   
       
  
    #[test]
    fn Muls_works(){
        let n: usize = 16;
        let seed = PrgSeed::random();
        let mut stream = FixedKeyPrgStream::new();
        stream.set_key(&seed.key);
        let v1 = stream.next_bits(n);
        let v2 = stream.next_bits(n);
        print!("v1="); print_bool_vec(&v1);
        print!("v2="); print_bool_vec(&v2);
 
        let beaver = MBeaver::gen(n);
    
        let (b1, b2) = beaver.split();
        //print!("b1="); print_bool_vec(&b1.a);
        //print!("b2="); print_bool_vec(&b2.a);
        let mut d1 = v1.clone();
        let mut d2 = v2.clone();
        
        for i in 0..n{
            d1[i] = v1[i] ^ b1[i];
            d2[i] = v2[i] ^ b2[i];
        }
        print!("d1="); print_bool_vec(&d1);
        print!("d2="); print_bool_vec(&d2);
        let mut d = d1.clone();
        for i in 0..n{
            d[i] = d1[i] ^ d2[i];
        }
        print!("d="); print_bool_vec(&d);
 
        let delta = MBeaver::extendfrom(d.clone());
        //print!("d.extend="); print_bool_vec(&delta.a);
        //let r1 = Muls(&delta, &b1, true).unwrap();
        //let r2 = Muls(&delta, &b2, false).unwrap();
        let r1 = product(&d, &b1, true).unwrap();
        let r2 = product(&d, &b2, false).unwrap();
    
        let r = r1 ^ r2;
    
        let mut v_mul = true;
        for i in 0..v1.len(){
            let unit = v1[i] ^ v2[i];
            v_mul = v_mul && unit;
        } 
        assert_eq!(v_mul, r);
    }

  
}
