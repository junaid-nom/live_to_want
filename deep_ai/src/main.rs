mod normal_nets;
use normal_nets::lolok;
extern crate tch;
use tch::Tensor;

pub fn main() {
    println!("{}", lolok());

    let t = Tensor::of_slice(&[3, 1, 4, 1, 5]);
    let t = t * 2;
    t.print();
}
