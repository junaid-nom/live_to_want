extern crate tch;
extern crate rust_bert;

use std::iter::successors;

use self::rust_bert::bert::BertConfig;
use normal_nets::*;


use self::tch::kind;

use self::tch::data::Iter2;

use self::tch::{nn, nn::ModuleT, nn::LinearConfig, nn::OptimizerConfig, Device, Tensor, no_grad_guard};
extern crate plotters;
use self::plotters::prelude::*;
use self::rust_bert::bert::BertEncoder;

/// 10 digits, -1 for negative, -2 for space
static EMBEDDING_LEN: usize = 10+2;

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

pub fn number_list_to_encoding(inp: &Vec<i32>) -> Vec<f32> {
    let mut encoding: Vec<f32> = Vec::new();

    let mut digit_count: usize = inp.iter().map(get_digit_count).sum();
    println!("Digits before spaces: {}", digit_count);
    digit_count += inp.len() - 1; // add for the spaces
    println!("Digits with spaces: {}", digit_count);
    let mut i = 0;
    inp.into_iter().for_each(| n| {
        // add space 
        if i != 0 {
            encoding.extend(digit_to_encoding(-2));
            encoding.extend(pos_to_encoding(i, digit_count));
            i+=1;
        }
        
        encoding.extend(number_to_encoding(*n, i, digit_count));
        i += get_digit_count(n);
    });

    encoding
}

pub fn decode_number(inp: Vec<f32>) -> i32 {
    let index_of_max: Option<usize> = inp
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index);
    let n: i32 = index_of_max.unwrap() as i32 - 2;
    return n;
}

pub fn decode_pos(inp: Vec<f32>) -> usize {
    let index_of_max: Option<usize> = inp
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index);
    return index_of_max.unwrap();
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
        pos = Some(decode_pos(pos_inp));
    }

    (n, pos)
}

// TODONEXT: Make decoding so encoding -> list of numbers and test both

pub fn encoding_to_number_list(inp: &Vec<f32>, max_num_tokens: usize, has_positionals: bool) -> Vec<i32> {
    assert!(inp.len() % max_num_tokens == 0);
    let token_length = inp.len() / max_num_tokens;
    assert_eq!(token_length, if has_positionals {EMBEDDING_LEN + max_num_tokens} else {EMBEDDING_LEN});

    let tokens: Vec<Vec<f32>> = inp.chunks(token_length).map(|s| s.into()).collect();

    let mut digits = Vec::new();

    tokens.into_iter().enumerate().for_each(|(pos, encoded)| {
        let (decoded, pos_decode) = decode(encoded, has_positionals);
        if has_positionals {
            assert!(pos_decode == Some(pos));
        } else {
            assert!(pos_decode == None);
        }
        digits.push(decoded);
    });

    // below is the hard part, summing digits into a real number
    let ret: Vec<i32> = digits.split(|n| *n == -2).map(|d_grouped| {
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
        total
    }).collect();

    ret
}

#[test]
pub fn test_encode_decode() {
    let original = vec![11,22,33,44,-55,-66,789]; // 9 + 8 = 17 digits
    let digits = 17;
    let spaces = original.len() - 1;
    let encode = number_list_to_encoding(&original);
    let single_token_len = EMBEDDING_LEN + (spaces + digits);
    let token_count = spaces + digits;
    assert_eq!(single_token_len * token_count, encode.len());

    let decoded = encoding_to_number_list(&encode, token_count, true);
    println!("original {:#?} decoded {:#?}", original, decoded);
    assert_eq!(original, decoded);
}

#[derive(Debug)]
struct DumbformerLayer {
    heads: Vec<LongNet>,
    head_combiner: nn::Linear,
    token_size: usize,
    max_num_tokens: usize,
    extra_size: usize,
    total_inp_length: i64,
}
impl DumbformerLayer {
    fn new(vs: &nn::Path, token_size: usize, max_num_tokens: usize, extra_size: usize, head_count: usize, head_hidden_layers: usize, head_hidden_node_size:i64, head_combiner_node_size: i64) -> DumbformerLayer {
        let total_inp_length: i64 = (extra_size + ((token_size+max_num_tokens) * (max_num_tokens + 1))) as i64;
        
        let mut heads = vec![];
        for _ in 0..head_count{
            heads.push(LongNet::new(vs, head_hidden_layers, head_hidden_node_size, total_inp_length, token_size as i64));
        }

        let head_combiner = nn::linear(vs, total_inp_length, head_combiner_node_size, Default::default());
        
        DumbformerLayer {
            heads,
            head_combiner,
            token_size,
            max_num_tokens,
            extra_size,
            total_inp_length,
        }
    }
}
impl ModuleT for DumbformerLayer {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        let a = xs.view([-1, self.total_inp_length]);
        // let mut a = a.apply(&self.in_layer).leaky_relu();
        // for layer in &self.hidden_linears {
        //     a = a.apply(layer).leaky_relu();
        // }
        // a = a.apply(&self.out_layer);
        a
    }
}



pub fn bert_test_on_math() {
    let bert_config = BertConfig {
        hidden_act: rust_bert::Activation::relu,
        attention_probs_dropout_prob: 0.,
        hidden_dropout_prob: 0.,
        hidden_size: todo!(),
        initializer_range: todo!(),
        intermediate_size: todo!(),
        max_position_embeddings: todo!(),
        num_attention_heads: todo!(),
        num_hidden_layers: todo!(),
        type_vocab_size: todo!(),
        vocab_size: todo!(),
        output_attentions: todo!(),
        output_hidden_states: todo!(),
        is_decoder: todo!(),
        id2label: todo!(),
        label2id: todo!(),
    };

    
}
