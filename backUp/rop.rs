use idpf::RingElm;
use idpf::Group;
use std::io;

fn main(){
    let mut input = String::new();
    let stdin=io::stdin();
    let mut x: RingElm;
    let mut y: RingElm;
    //let mut r: RingElm;
    let mut ct = true;
    while ct{
          println!("Please choose your operation: 1.+, 2.- 3.*");
          stdin.read_line(&mut input).expect("Failed to read line");
          let choice: u8 = input.trim().parse().expect("Input not a integer");
          println!("Please input two integers:");
          input.clear();
          stdin.read_line(&mut input).expect("Failed to read line");
          let v0:u32 = input.trim().parse().expect("Input not a integer");
          input.clear();
          stdin.read_line(&mut input).expect("Failed to read line");
          let v1:u32 = input.trim().parse().expect("Input not a integer");

          x = RingElm::from(v0);
          y = RingElm::from(v1);
          match choice{
                1 => {x.add(&y);}
                2 => {x.sub(&y);}
                3 => {x.mul(&y);}
                _ => {println!("Error input");}
          }
          println!("Continue?y/n");
          stdin.read_line(&mut input).expect("Failed to read line");
          if input == "y" || input == "Y"{
                ct = true;
          }
          else{
            ct = false;
          }
    }
}