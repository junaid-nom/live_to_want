extern crate tch;
extern crate rust_bert;

use self::tch::kind;

use self::tch::data::Iter2;

use self::tch::{nn, nn::ModuleT, nn::LinearConfig, nn::OptimizerConfig, Device, Tensor, no_grad_guard};
extern crate plotters;
use self::plotters::prelude::*;
use self::rust_bert::bert::BertEncoder;


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
impl ModuleT for ConvMNISTNet {
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
pub struct LongNet {
    pub hidden_linears: Vec<nn::Linear>,
    pub in_layer: nn::Linear,
    pub out_layer: nn::Linear,
    pub in_size: i64
}
impl LongNet {
    pub fn new(vs: &nn::Path, hidden_layers_count: usize, hidden_layer_dim: i64, in_dim :i64, out_dim: i64) -> LongNet {
        let lconfig = LinearConfig{
            ..Default::default()
        };
        
        let mut net = LongNet{ 
            hidden_linears: vec![], 
            //fc1, fc2, fc3, fc4, fc5, fc6, fc7, fc8, fc9, fc10,
            in_layer: nn::linear(vs, in_dim, hidden_layer_dim, Default::default()),
            out_layer: nn::linear(vs, hidden_layer_dim, out_dim, Default::default()),
            in_size: in_dim,
        };

        for lay in 0..hidden_layers_count{
            net.hidden_linears.push(nn::linear(vs, hidden_layer_dim, hidden_layer_dim, Default::default()));
        }
        //no_grad_guard();
        //net.in_layer.ws = net.in_layer.ws.fill(1).requires_grad_(false);
        net
    }
}
impl ModuleT for LongNet {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        let a = xs.view([-1, self.in_size]);
        let mut a = a.apply(&self.in_layer).leaky_relu();
        for layer in &self.hidden_linears {
            a = a.apply(layer).leaky_relu();
        }
        a = a.apply(&self.out_layer);
        a
    }
}

#[test]
pub fn test_net_set_weights() {
    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = LongNet::new(&vs.root(), 0, 10, 1, 1);
}

pub fn draw_graph() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("plotters-doc-data/0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
            &RED,
        ))?
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("done");
    Ok(())
}

pub fn get_net_accuracy(net: &dyn ModuleT, x: &Tensor, true_y: &Tensor, batch_size: i64) -> f64 {
    return get_net_accuracy_inner(net, x, true_y, batch_size, false);
}

pub fn get_net_accuracy_inner(net: &dyn ModuleT, x: &Tensor, true_y: &Tensor, batch_size: i64, print_outs: bool) -> f64 {
    let _no_grad = no_grad_guard();
    let mut sum_accuracy = 0f64;
    let mut sample_count = 0f64;
    for (xs, ys) in Iter2::new(x, true_y, batch_size).to_device(Device::cuda_if_available()).return_smaller_last_batch() {
        let y_out = net.forward_t(&xs, false);
        if print_outs {
            y_out.print();
        }
        let size = xs.size()[0] as f64;
        sum_accuracy += get_accuracy_tensors(&y_out, &ys);
        sample_count += size;
    }
    println!("Sum acc: {} Sample Count: {}", sum_accuracy,sample_count);
    (1. - (sum_accuracy / sample_count)).abs()
}

pub fn get_accuracy_tensors(y: &Tensor, true_y: &Tensor) -> f64 {
    Vec::<f64>::from((y.f_add_scalar(0.001).unwrap().true_divide(&true_y.f_add_scalar(0.001).unwrap()).f_add_scalar(-1)).unwrap().abs().sum(tch::Kind::Double))[0]
}
#[test]
fn test_get_net_accuracy() {
    assert_eq!(get_accuracy_tensors(
        &Tensor::of_slice(&[1,2,3,4,5]),
        &Tensor::of_slice(&[1,2,3,4,5])), 0.);

    assert_eq!(get_accuracy_tensors(
        &Tensor::of_slice(&[2,4,6,8,10]),
        &Tensor::of_slice(&[1,2,3,4,5])
    ) / 5., 1.);

    assert_eq!(get_accuracy_tensors(
        &Tensor::of_slice(&[1,4,6,8,5, 6]),
        &Tensor::of_slice(&[1,2,3,4,5, 6])
    ) / 6., 0.5);

    assert_eq!(get_accuracy_tensors(
        &Tensor::of_slice(&[1,2,3,4,5, 6]),
        &Tensor::of_slice(&[1,4,6,8,5, 6])
    ) / 6., 0.25);
}


pub fn run_net_on_cos_func() {
    // Easily make a tensor:
    // let vec = [3.0, 1.0, 4.0, 1.0, 5.0].to_vec();
    // let t1 = Tensor::of_slice(&vec);

    let root = BitMapBackend::new("plotters-doc-data/0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("cos(sin(10*(x^2))^3)", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-1f32..1f32, -1f32..1f32).unwrap();

    chart.configure_mesh().draw().unwrap();

    // Make a dataset using the cosine func: cos(sin(10*(x^2))^3)
    // from -1 to 1 as x input. 1/10000 - 1/10000, 20k data points
    let mut x = Tensor::range_step(-10000,10000, kind::FLOAT_CUDA);
    x = x / 10000;
    let x = x.reshape(&[20001, 1]);
    //x.print();
    println!("x: {:#?}", x.size());
    let y = (x.square() * 10).sin().pow_(3.0).cos();
    let y = y.reshape(&[20001, 1]);
    //y.print();
    println!("y: {:#?}", y.size());
    //println!("x, 15000: {:#?}", x.view([20001]).double_value(&[15000]));
    //println!("vec x {:#?}", Vec::<f64>::from(&x));

    //let og_iter = (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x));
    //let true_iter = Vec::<f64>::from(&x).into_iter().zip(Vec::<f64>::from(&y));
    let true_iter2= Vec::<f32>::from(&x).into_iter().zip(Vec::<f32>::from(&y));

    //println!("example {:#?}", og_iter);
    //println!("v {:#?}", true_iter);
    chart
        .draw_series(LineSeries::new(
            //true_iter.clone(),
            true_iter2.clone(),
            &RED,
        )).unwrap()
        .label("real")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    //println!("collected: {:#?}",true_iter.collect::<Vec<(f64,f64)>>());
    //println!("vec from y: {:#?}",Vec::<f64>::from(&y));
    //println!("y: -3 {:#?} -4 {:#?} -5 {:#?}", y.view([20001]).double_value(&[20001-3]), y.view([20001]).double_value(&[20001-4]), y.view([20001]).double_value(&[20001-5]));
    //println!("collected: {:#?}", true_iter2.collect::<Vec<(f32,f32)>>());
    //y.print();

    // TODONEXT: Why does the function result for y KEEP CHANGING???
    // and why is the graph totally fucked

    let vs = nn::VarStore::new(Device::cuda_if_available());
    for _ in 0..10 {
        for layer_count in 0..9 {
            let hidden_layers_count = layer_count;
            let hidden_layer_neurons = 10;
            
            let net = LongNet::new(&vs.root(), hidden_layers_count, hidden_layer_neurons, 1, 1);
            let mut opt = nn::Adam::default().build(&vs, 1e-4).unwrap();
            let batch_size = 20001;
            let epoch_default = 1000;
            let epochs = batch_size / 256 / 1 * epoch_default;
            //let epochs = 1;
            let mut epochs_occured = 0;
            let mut test_accuracy = 0.;
            for _ in 1..epochs+1 {
                epochs_occured += 1;
                let mut data = Iter2::new(&x,&y,batch_size);
                let data = data.to_device(Device::cuda_if_available());
                for (batch_xs, batch_ys) in data.shuffle() {
                    //println!("xs: {:#?}", batch_xs.size());
                    let loss = net.forward_t(&batch_xs, true);
                    let loss = loss.mse_loss(&batch_ys, tch::Reduction::Mean);
                    //loss.print();
                    opt.backward_step(&loss);
                }
                
                test_accuracy = get_net_accuracy(&net, &x, &y, batch_size);
                // println!("epoch: {:4} test acc: {:5.2}%", epoch, 100. * test_accuracy,);
                if test_accuracy > 0.99 {
                    break;
                }
            }
    
            println!("batch_size: {} epochs {} LayerCount: {}, Nuerons per layer: {} accuracy final: {}", batch_size, epochs_occured, hidden_layers_count + 2, hidden_layer_neurons, test_accuracy);
    
            let mut data = Iter2::new(&x,&y,256);
            let data = data.to_device(Device::cuda_if_available());
            let mut total_vec = Vec::<(f32,f32)>::new();
            for (batch_xs, _) in data {
                //println!("xs: {:#?}", batch_xs.size());
                let out = net.forward_t(&batch_xs, false);
                //out.print();
                let part_iter= Vec::<f32>::from(&batch_xs).into_iter().zip(Vec::<f32>::from(&out));
                total_vec.extend(part_iter);
            }
    
            chart
                .draw_series(LineSeries::new(
                    //true_iter.clone(),
                    total_vec.clone(),
                    &BLUE,
                )).unwrap()
                .label(String::from(format!("net L: {} N: {}", hidden_layers_count, hidden_layer_neurons)))
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        }
    }
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw().unwrap();

    root.present().unwrap();
    
}
