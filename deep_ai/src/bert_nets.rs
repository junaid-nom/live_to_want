extern crate tch;
extern crate rust_bert;

use std::iter::successors;

use self::rust_bert::bert::BertConfig;
use normal_nets::*;
use self::tch::IndexOp;


use self::tch::kind;

use self::tch::data::Iter2;

use self::tch::{nn, nn::ModuleT, nn::LinearConfig, nn::OptimizerConfig, Device, Tensor, no_grad_guard};
extern crate plotters;
use self::plotters::prelude::*;
use self::rust_bert::bert::BertEncoder;

/// 10 digits, -1 for negative, -2 for space
static EMBEDDING_LEN: usize = 10+2;


/// Takes in a number and returns a vector of digits. so 142 becomes [1,4,2]
fn number_to_vec(n: i32) -> Vec<i32> {
    let mut digits = Vec::new();
    let mut n = n;
    while n > 9 {
        digits.push(n % 10);
        n = n / 10
    }
    digits.push(n);
    digits.reverse();
    digits
}

/// Expects number from -2-9 inclusive. -1 encodes "negative" symbol.
/// -2 encodes "space" symbol
fn digit_to_encoding(d: i32) -> Vec<f32> {
    let mut binary = vec![0.; EMBEDDING_LEN];
    assert!(d >= -2 && d <= 9);
    binary[(d+2) as usize] = 1.;
    binary
}

fn pos_to_encoding(pos: usize, max_num_tokens: usize) -> Vec<f32> {
    let mut binary = vec![0.; max_num_tokens];
    assert!(pos < max_num_tokens);
    binary[pos] = 1.;
    binary
}

pub fn number_to_encoding(mut n: i32, mut pos: usize, max_num_tokens: usize) -> Vec<f32> {
    let mut encodding = Vec::new();
    let is_neg = n < 0;

    if is_neg {
        n = -n;
    }

    let mut digits = number_to_vec(n);
    if is_neg {
        digits.insert(0, -1)
    }

    digits.into_iter().for_each(|d|  {
            encodding.extend(digit_to_encoding(d));
            encodding.extend(pos_to_encoding(pos, max_num_tokens));
            pos+=1;
        }
    );

    encodding
}

pub fn get_digits_from_positive(inp: i32) -> usize {
    successors(Some(inp), |&n| (n >= 10).then(|| n / 10)).count()
}
#[test]
fn test_get_digits_from_positive() {
    assert_eq!(get_digits_from_positive(1), 1);
    assert_eq!(get_digits_from_positive(11), 2);
    assert_eq!(get_digits_from_positive(99), 2);
    assert_eq!(get_digits_from_positive(999), 3);

    assert_eq!(get_digit_count(&-999), 4);
    assert_eq!(get_digit_count(&999), 3);
    assert_eq!(get_digit_count(&-99), 3);
    assert_eq!(get_digit_count(&-11), 3);
    assert_eq!(get_digit_count(&-1), 2);
}

pub fn get_digit_count(inp: &i32) -> usize {
    let mut inp = *inp;
    let mut digit_count = 0;
    if inp < 0 {
        inp = -1 * inp;
        digit_count += 1;
    }

    digit_count += get_digits_from_positive(inp);

    digit_count
}

pub fn number_list_to_encoding(inp: &Vec<i32>, max_num_digits_and_spaces: usize) -> Vec<f32> {
    let mut encoding: Vec<f32> = Vec::new();

    let mut digit_count: usize = inp.iter().map(get_digit_count).sum();
    // println!("Digits before spaces: {}", digit_count);
    digit_count += inp.len() - 1; // add for the spaces
    // println!("Digits with spaces: {}", digit_count);
    let mut i = 0;
    assert!(digit_count <= max_num_digits_and_spaces);
    inp.into_iter().for_each(| n| {
        // add space 
        if i != 0 {
            encoding.extend(digit_to_encoding(-2));
            encoding.extend(pos_to_encoding(i, max_num_digits_and_spaces));
            i+=1;
        }
        
        encoding.extend(number_to_encoding(*n, i, max_num_digits_and_spaces));
        i += get_digit_count(n);
    });

    // pad with 0s for unused digits
    let token_length = EMBEDDING_LEN + max_num_digits_and_spaces;
    assert_eq!(encoding.len()/token_length, digit_count);
    
    let max_length = max_num_digits_and_spaces * token_length;
    let missing = max_length - (digit_count * token_length);
    for _ in 0..missing {
        encoding.push(0.);
    }

    encoding
}

pub fn number_list_to_encoding_f(inp: &Vec<f32>, max_num_tokens: usize) -> Vec<f32> {
    let int_v: Vec<i32> = inp.into_iter().map(|n| *n as i32).collect();
    return number_list_to_encoding(&int_v, max_num_tokens);
}

pub fn decode_number(inp: Vec<f32>) -> i32 {
    let sum: f32 = inp.iter().sum();
    if sum == 0. {
        return -2;
    }
    let index_of_max: Option<usize> = inp
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index);
    let n: i32 = index_of_max.unwrap() as i32 - 2;
    return n;
}

pub fn decode_pos(inp: &Vec<f32>) -> usize {
    let sum: f32 = inp.iter().sum();
    if sum == 0. {
        return usize::MAX;
    }
    let index_of_max: Option<usize> = inp
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index);
    match index_of_max {
        Some(i) => i,
        None => 0,
    }
}

pub fn decode(inp: Vec<f32>, includes_positionals: bool) -> (i32, Option<usize>) {
    let to_decode_n = if includes_positionals {
        inp[0..EMBEDDING_LEN].to_vec()
    } else {
        inp.clone()
    };

    let n = decode_number(to_decode_n);
    
    let mut pos = None;
    if includes_positionals {
        let pos_inp = inp[EMBEDDING_LEN..].to_vec();
        pos = Some(decode_pos(&pos_inp));
        //println!("pos: {:#?}, inp{:#?}", pos, pos_inp);
    }

    (n, pos)
}

#[test]
pub fn test_split() {
    let v = vec![0, 1, 2, 3, 4, 5, 2, 2, 2, 2, 2, 2];
    v.split(|n| *n == -2).map(|d_grouped| {
        
    });
}

pub fn encoding_to_number_list(inp: &Vec<f32>, max_num_tokens: usize, has_positionals: bool) -> Vec<i32> {
    assert!(inp.len() % max_num_tokens == 0);
    let token_length = inp.len() / max_num_tokens;
    assert_eq!(token_length, if has_positionals {EMBEDDING_LEN + max_num_tokens} else {EMBEDDING_LEN});

    let tokens: Vec<Vec<f32>> = inp.chunks(token_length).map(|s| s.into()).collect();

    let mut digits = Vec::new();

    tokens.into_iter().enumerate().for_each(|(pos, encoded)| {
        let (decoded, pos_decode) = decode(encoded, has_positionals);
        if has_positionals {
            //println!("pos2: {:#?} == {}", pos_decode, pos);
            assert!(pos_decode == Some(pos) || pos_decode == Some(usize::MAX));
        } else {
            assert!(pos_decode == None);
        }
        digits.push(decoded);
    });

    // below is the hard part, summing digits into a real number
    let mut ret: Vec<i32> = vec![];
    digits.split(|n| *n == -2).for_each(|d_grouped| {
        if d_grouped.len() > 0 {
            let mut total = 0;
            let mut d_grouped = d_grouped.to_vec();
            d_grouped.reverse();
            let mut multiplier = 1;
            d_grouped.into_iter().for_each(|digit| {
                if digit == -1 {
                    total *= -1;
                } else {
                    assert!(digit >= 0);
                    total += digit * multiplier;
                }

                multiplier *= 10;
            });
            ret.push(total);
        }
    });

    ret
}

#[test]
pub fn test_encode_decode() {
    let original = vec![11,22,33,44,-55,-66,789]; // 9 + 8 = 17 digits
    let digits = 17;
    let spaces = original.len() - 1;
    let max_digits_and_spaces = digits + spaces + 10;
    let encode = number_list_to_encoding(&original, max_digits_and_spaces);
    let single_token_len = EMBEDDING_LEN + (max_digits_and_spaces);
    let token_count = max_digits_and_spaces;
    assert_eq!(single_token_len * token_count, encode.len());

    let decoded = encoding_to_number_list(&encode, token_count, true);
    println!("original {:#?} decoded {:#?}", original, decoded);
    assert_eq!(original, decoded);
}

#[derive(Debug)]
struct DumbformerLayer {
    heads: Vec<LongNet>,
    head_combiner: nn::Linear,
    token_size_no_pos: usize,
    max_num_tokens: usize,
    extra_size: usize,
    total_inp_length_to_head: i64,
    sentence_length_with_extra: i64,
    head_combiner_size: i64,
}
impl DumbformerLayer {
    fn new(vs: &nn::Path, token_size_no_pos: usize, max_num_tokens: usize, extra_size: usize, head_count: usize, head_hidden_layers: usize, head_hidden_node_size:i64) -> DumbformerLayer {
        let pos_size = max_num_tokens;
        
        let total_inp_length_to_head: i64 = (extra_size + ((token_size_no_pos+pos_size) * (max_num_tokens + 1))) as i64;
        let sentence_length_with_extra = (extra_size + ((token_size_no_pos+pos_size) * (max_num_tokens))) as i64;

        let head_combiner_size = (extra_size + (head_count * token_size_no_pos)) as i64;

        let mut heads = vec![];
        for _ in 0..head_count{
            heads.push(LongNet::new(vs, head_hidden_layers, head_hidden_node_size, total_inp_length_to_head, token_size_no_pos as i64));
        }

        let head_combiner = nn::linear(vs, head_combiner_size, (token_size_no_pos) as i64 , Default::default());
        
        DumbformerLayer {
            heads,
            head_combiner,
            token_size_no_pos,
            max_num_tokens,
            extra_size,
            total_inp_length_to_head,
            sentence_length_with_extra,
            head_combiner_size,
        }
    }
}
impl ModuleT for DumbformerLayer {
    /// Input should be: extra, sentence(token1, pos1, token2, pos2, token3, pos3 ...)
    /// position should be after token in the sentence
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        let mut a = xs.view([-1, self.sentence_length_with_extra]); // should it be 1?
        if train {
            assert_eq!(a.requires_grad(), true);
        }
        

        let row_count_real = a.size()[0];

        let extra_size = self.extra_size as i64;
        let extra = a.i((.., 0..extra_size));
        let sentence = a.i((.., extra_size..));
        let token_and_pos_size = (self.token_size_no_pos + self.max_num_tokens) as i64;

        let chunks = sentence.chunk(self.max_num_tokens as i64, 1);
        let mut final_tokens = Vec::new();
        for token in chunks {
            let mut outs = vec![];
            outs.push(extra.copy());
            let inp = Tensor::cat(&[extra.copy(), token, sentence.copy()],1);
            if train {
                assert_eq!(inp.requires_grad(), true);
            }

            for head in &self.heads {
                outs.push(head.forward_t(&inp, train).leaky_relu());
            }
            let total_out = Tensor::cat(&outs, 1);
            if train {
                assert_eq!(total_out.requires_grad(), true);
            }
            final_tokens.push(self.head_combiner.forward_t(&total_out, train).leaky_relu());
        }
        let mut tokens_plus_pos = Vec::new();
        tokens_plus_pos.push(extra);
        for (i, token) in final_tokens.iter().enumerate() {
            let pos = Tensor::of_slice(&(pos_to_encoding(i, self.max_num_tokens))).to_device(Device::cuda_if_available());
            let pos = pos.repeat(&[row_count_real, 1]).to_device(Device::cuda_if_available());
            tokens_plus_pos.push(Tensor::cat(&[token, &pos],1));
        }

        let output = Tensor::cat(&tokens_plus_pos, 1);
        if train {
            assert_eq!(output.requires_grad(), true);
        }
        return output;
        
        // take out extra
        // take out sentence.
        // for each token: inp= extra + token + sentence 
            // foreach head(inp) -> out
                // concatenate outs
            // re-add extra obtain head_combiner out per token
        // sentence = foreach token: token + position
        // final out: extra + sentence.

    }

}
impl DumbformerLayer {
    fn backward(&self) -> Tensor {
        // final out
        // -> final out.grad()
        // split by token
        // for each token, times it by the weights. so t1 * weights + t2 *weights etc
        // this will be the gradient for the out layer of the head combiner.
        // take that gradient and run it through headCombiner.backward_with_grad_data
        // now u have the gradient for the head combiner. 
        // take that gradient and split it by head output. so head1_gradient, head2_gradient...
        // then for each head do backward_with_grad_data with its head1_gradient etc.
        // this will give gradients for each extra + token + sentence.
            // take out the gradient extra and sum it as extra gradient.
            // take out sentence gradient for each one and sum it.
            // take out each token's gradient and combine it into a sentence gradient and add that to the existing sentence gradient
        // return extra -cat- sentence gradient
        // apply to layer above in the calling code via backward_with_grad_data
        Tensor::of_slice(&[0,2,3])
    }
}


#[test]
fn test_transformations() {
    let extra_size = 3;
    let token_size_no_pos = 2;
    let max_num_tokens = 2;
    let pos_size = 1;

    let head_count = 3;

    let total_inp_length_to_head: i64 = (extra_size + ((token_size_no_pos+pos_size) * (max_num_tokens + 1))) as i64;
    assert_eq!(total_inp_length_to_head, 12);

    let sentence_length_with_extra = (extra_size + ((token_size_no_pos+pos_size) * (max_num_tokens))) as i64;
    let head_combiner_size = (extra_size + (head_count * token_size_no_pos)) as i64;
    let token_and_pos_size = (token_size_no_pos + pos_size) as i64;

    let mut xs = Tensor::of_slice(&[
        22.,22.,22.,    0.,1.,2.,3.,4.,44.,
        66.,66.,66.,    5.,6.,7.,8.,9.,99.,
        -22.,-22.,-22., -1., -2., -3., -4., -5., -55.,]);
    xs = xs.requires_grad_(true);
    let row_count = 3;
    let a = xs.view([-1, sentence_length_with_extra]); // should it be 1?
    let row_count_real = a.size()[0];
    assert_eq!(row_count, row_count_real);

    let extra_size = extra_size as i64;
    let extra = a.i((.., 0..extra_size));
    println!("EXTRA:");
    extra.print();
    println!("EXTRA END");
    assert_eq!(extra.internal_shape_as_tensor(), Tensor::of_slice(&[row_count, extra_size]));

    let sentence = a.i((.., extra_size..));
    println!("Sentence:");
    sentence.print();
    println!("Sentence END");
    assert_eq!(sentence.internal_shape_as_tensor(), Tensor::of_slice(&[row_count, sentence_length_with_extra - extra_size]));

    // NOTE first number in chunks is how many chunks there are so size of each chunk is: total_size/chunks
    let chunks = sentence.chunk(max_num_tokens, 1);
    assert_eq!(chunks.len(), max_num_tokens as usize);

    let mut final_tokens = vec![];

    println!("Each token:");
    for token in chunks {
        let mut outs = vec![];
        outs.push(extra.copy());
        println!("token:");
        token.print();
        assert_eq!(token.internal_shape_as_tensor(), Tensor::of_slice(&[row_count, token_and_pos_size]));

        println!("total to head inp:");
        let inp = Tensor::cat(&[extra.copy(), token.copy(), sentence.copy()],1);
        inp.print();
        assert_eq!(inp.requires_grad(), true);
        assert_eq!(inp.internal_shape_as_tensor(), Tensor::of_slice(&[row_count, total_inp_length_to_head]));
        println!("total to head inp END");

        for i in 0..head_count {
            outs.push(token.copy().i((.., 0..token_size_no_pos)));
        }

        let total_out = Tensor::cat(&outs, 1);
        println!("total to head inp:");
        total_out.print();
        assert_eq!(token_size_no_pos * head_count + extra_size, head_combiner_size);
        assert_eq!(total_out.size(), [row_count, head_combiner_size]);
        println!("total to head inp END");

        final_tokens.push(token.copy().i((.., 0..token_size_no_pos)));
    }
    println!("Each token END");

    let mut tokens_plus_pos = Vec::new();
    tokens_plus_pos.push(extra);
    for (i, token) in final_tokens.iter().enumerate() {
        let pos = Tensor::of_slice(&[i as i64]);
        let pos = pos.repeat(&[row_count_real, 1]);
        tokens_plus_pos.push(Tensor::cat(&[token, &pos],1));
    }
    let final_out = Tensor::cat(&tokens_plus_pos, 1);
    assert_eq!(final_out.requires_grad(), true);
    println!("Final out:");
    final_out.print();
    assert_eq!(final_out.size(), a.size());

    // let mut final_tokens = Vec::new();
    // for token in chunks {
    //     let mut outs = vec![];
    //     let inp = Tensor::cat(&[extra.copy(), token, sentence.copy()],1);
    //     for head in &self.heads {
    //         outs.push(head.forward_t(&inp, train).leaky_relu());
    //     }
    //     let total_out = Tensor::cat(&outs, 1);
    //     final_tokens.push(self.head_combiner.forward_t(&total_out, train).leaky_relu());
    // }
    // let mut tokens_plus_pos = Vec::new();
    // tokens_plus_pos.push(extra);
    // for (i, token) in final_tokens.iter().enumerate() {
    //     let pos = Tensor::of_slice(&(pos_to_encoding(i, self.max_num_tokens)));
    //     tokens_plus_pos.push(Tensor::cat(&[token, &pos],1));
    // }
    // return Tensor::cat(&tokens_plus_pos, 1);


    assert!(true);
}


pub fn dumbformer_test_on_identity() {
    /* 
    let root = BitMapBackend::new("plotters-doc-data/0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("cos(sin(10*(x^2))^3)", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-1f32..1f32, -1f32..1f32).unwrap();

    chart.configure_mesh().draw().unwrap();
    */
    let dims = [10000, 4];
    let random_numbers = Tensor::randint_low(1, 999, &dims, kind::INT64_CUDA);
    //random_numbers.print();
    let vec_numbers: Vec<Vec<f32>> = Vec::from(random_numbers);
    // println!("num {:#?}", vec_numbers);

    let max_token_num = 20;
    let encoded_numbers: Vec<Vec<f32>> = vec_numbers.iter().map(|x| number_list_to_encoding_f(&x,max_token_num)).collect();

    let recoded = encoded_numbers.iter().map(|v| encoding_to_number_list(&v, max_token_num, true)).collect::<Vec<Vec<i32>>>();
    assert_eq!(recoded, vec_numbers.iter().map(|n| n.iter().map(|n| *n as i32).collect()).collect::<Vec<Vec<i32>>>());

    let flat: Vec<f32> = encoded_numbers.into_iter().flatten().collect();
    let x = Tensor::of_slice(&flat);
    let x = x.view([10000, -1]).to_device(Device::cuda_if_available());
    let y = x.copy().to_device(Device::cuda_if_available());

    /* 
    chart
        .draw_series(LineSeries::new(
            //true_iter.clone(),
            true_iter2.clone(),
            &RED,
        )).unwrap()
        .label("real")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    */

    let vs = nn::VarStore::new(Device::cuda_if_available());
    
    for _ in 0..1 {
        for layer_count in 0..1 {
            let hidden_layers_count = 2;
            let hidden_layer_neurons = 100;
            
            let net = DumbformerLayer::new(&vs.root(), EMBEDDING_LEN, 20,0,2, hidden_layers_count, hidden_layer_neurons);
            let mut opt = nn::Adam::default().build(&vs, 1e-4).unwrap();
            let batch_size = 20001;
            let epoch_default = 1000;
            let epochs = batch_size / 256 / 1 * epoch_default;
            let epochs = 1000;
            let mut epochs_occured = 0;
            let mut test_accuracy = 0.;
            for epoch in 1..epochs+1 {
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
                println!("epoch: {:4} test acc: {:5.2}%", epoch, 100. * test_accuracy,);
                if test_accuracy > 0.98 && test_accuracy < 1.02 {
                    break;
                }
            }
    
            println!("batch_size: {} epochs {} LayerCount: {}, Nuerons per layer: {} accuracy final: {}", batch_size, epochs_occured, hidden_layers_count + 2, hidden_layer_neurons, test_accuracy);
    
            let mut data = Iter2::new(&x,&y,256);
            let data = data.to_device(Device::cuda_if_available());
            let mut total_vec = Vec::<Vec<(f32,f32)>>::new();
            for (batch_xs, batch_ys) in data {
                //println!("xs: {:#?}", batch_xs.size());
                let out = net.forward_t(&batch_xs, false);
                //out.print();
                let part_iter= Vec::<f32>::from(&out).into_iter().zip(Vec::<f32>::from(&batch_ys));
                total_vec.push(part_iter.collect());
            }
            //println!("Outs: {:?}", total_vec[0]);
            /*
            chart
                .draw_series(LineSeries::new(
                    //true_iter.clone(),
                    total_vec.clone(),
                    &BLUE,
                )).unwrap()
                .label(String::from(format!("net L: {} N: {}", hidden_layers_count, hidden_layer_neurons)))
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
            */
        }
    }
    
    
    /*
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw().unwrap();

    root.present().unwrap();
    */
}
