extern crate tch;
use self::tch::kind;

use self::tch::data::Iter2;

use self::tch::{nn, nn::ModuleT, nn::OptimizerConfig, Device, Tensor};

pub fn lolok() -> String {
    return "Lol Okay".to_string();
}

#[derive(Debug)]
struct ConvMNISTNet {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}
impl ConvMNISTNet {
    fn new(vs: &nn::Path) -> ConvMNISTNet {
        let conv1 = nn::conv2d(vs, 1, 32, 5, Default::default());
        let conv2 = nn::conv2d(vs, 32, 64, 5, Default::default());
        let fc1 = nn::linear(vs, 1024, 1024, Default::default());
        let fc2 = nn::linear(vs, 1024, 10, Default::default());
        ConvMNISTNet { conv1, conv2, fc1, fc2 }
    }
}
impl nn::ModuleT for ConvMNISTNet {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        xs.view([-1, 1, 28, 28])
            .apply(&self.conv1)
            .max_pool2d_default(2)
            .apply(&self.conv2)
            .max_pool2d_default(2)
            .view([-1, 1024])
            .apply(&self.fc1)
            .leaky_relu()
            .dropout(0.5, train)
            .apply(&self.fc2)
    }
}

pub fn RunMNISTConvNet() {
    println!("{}", lolok());

    let m = tch::vision::mnist::load_dir("/home/junaid/Downloads/TensorFlow-MNIST-master/mnist/data").unwrap();
    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = ConvMNISTNet::new(&vs.root());
    let mut opt = nn::Adam::default().build(&vs, 1e-4).unwrap();
    println!("ts: {:#?}", m.test_images.size());
    for epoch in 1..2 {
        for (bimages, blabels) in m.train_iter(256).shuffle().to_device(vs.device()) {
            println!("xs: {:#?}", bimages.size());
            let loss = net.forward_t(&bimages, true).cross_entropy_for_logits(&blabels);
            opt.backward_step(&loss);
        }
        let test_accuracy =
            net.batch_accuracy_for_logits(&m.test_images, &m.test_labels, vs.device(), 1024);
        println!("epoch: {:4} test acc: {:5.2}%", epoch, 100. * test_accuracy,);
    }

}


#[derive(Debug)]
struct LongNet {
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc4: nn::Linear,
    fc5: nn::Linear,
    fc6: nn::Linear,
    fc7: nn::Linear,
    fc8: nn::Linear,
    fc9: nn::Linear,
    fc10: nn::Linear,
}
impl LongNet {
    fn new(vs: &nn::Path) -> LongNet {
        let fc1 = nn::linear(vs, 1, 200, Default::default());
        let fc2 = nn::linear(vs, 200, 200, Default::default());
        let fc3 = nn::linear(vs, 200, 200, Default::default());
        let fc4 = nn::linear(vs, 200, 200, Default::default());
        let fc5 = nn::linear(vs, 200, 200, Default::default());
        let fc6 = nn::linear(vs, 10, 10, Default::default());
        let fc7 = nn::linear(vs, 10, 10, Default::default());
        let fc8 = nn::linear(vs, 10, 10, Default::default());
        let fc9 = nn::linear(vs, 10, 10, Default::default());
        let fc10 = nn::linear(vs, 200, 1, Default::default());
        LongNet { fc1, fc2, fc3, fc4, fc5, fc6, fc7, fc8, fc9, fc10 }
    }
}
impl nn::ModuleT for LongNet {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        let a = xs.view([-1,1])
            .apply(&self.fc1)
            .leaky_relu()
            .apply(&self.fc2)
            .leaky_relu()
            .apply(&self.fc3)
            .leaky_relu()
            .apply(&self.fc4)
            .leaky_relu()
            .apply(&self.fc5)
            .leaky_relu()
            // .apply(&self.fc6)
            // .leaky_relu()
            // .apply(&self.fc7)
            // .leaky_relu()
            // .apply(&self.fc8)
            // .leaky_relu()
            // .apply(&self.fc9)
            // .leaky_relu()
            .apply(&self.fc10);
        //a.print();
        a
    }
}


pub fn run_net_on_cos_func() {
    let v8: Vec<u8> = vec![1,2];

    // Make a dataset using the cosine func: cos(sin(10*(x^2))^3)
    // from -1 to 1 as x input. 1/10000 - 1/10000, 20k data points
    let mut x = Tensor::range_step(-10000,10000, kind::FLOAT_CUDA);
    x = x / 10000;
    let x = x.reshape(&[20001, 1]);
    //x.print();
    println!("x: {:#?}", x.size());
    let y = (x.square() * 10).sin().exponential_(3.0).cos();
    let y = y.reshape(&[20001, 1]);
    //y.print();
    println!("y: {:#?}", y.size());
    
    

    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = LongNet::new(&vs.root());
    let mut opt = nn::Adam::default().build(&vs, 1e-4).unwrap();

    for epoch in 1..100 {
        let mut data = Iter2::new(&x,&y,256);
        let mut data = data.to_device(Device::cuda_if_available());
        for (batch_xs, batch_ys) in data.shuffle() {
            //println!("xs: {:#?}", batch_xs.size());
            let loss = net.forward_t(&batch_xs, true).mse_loss(&batch_ys, tch::Reduction::Mean);
            loss.print();
            opt.backward_step(&loss);
        }
        let test_accuracy = net.batch_accuracy_for_logits(&x, &y, vs.device(), 1024);
        println!("epoch: {:4} test acc: {:5.2}%", epoch, 100. * test_accuracy,);
    }
}
