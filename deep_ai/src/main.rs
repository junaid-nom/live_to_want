mod normal_nets;
use normal_nets::*;


pub fn main() {
    println!("{}", lolok());
    tch::maybe_init_cuda();
    println!("Cuda available: {}", tch::Cuda::is_available());
    println!("Cudnn available: {}", tch::Cuda::cudnn_is_available());
    
    //RunMNISTConvNet();
    //draw_graph().unwrap();
    run_net_on_cos_func();
}
