extern crate tch;
extern crate rust_bert;

use self::rust_bert::bert::BertConfig;

use self::tch::kind;

use self::tch::data::Iter2;

use self::tch::{nn, nn::ModuleT, nn::LinearConfig, nn::OptimizerConfig, Device, Tensor, no_grad_guard};
extern crate plotters;
use self::plotters::prelude::*;
use self::rust_bert::bert::BertEncoder;

/// 10 digits, -1 for negative, -2 for space
static embedding_len: usize = 10+2; 

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
    let mut binary = vec![0.; 11];
    assert!(d >= -2 && d <= 9);
    binary[(d+2) as usize] = 1.;
    binary
}


pub fn number_to_encoding(n: i32) -> Vec<f32> {
    let mut encodding = Vec::new();
    let isNeg = n < 0;

    if isNeg {
        n = -n;
    }

    let mut digits = number_to_vec(n);
    if isNeg {
        digits.insert(0, -1)
    }

    digits.into_iter().for_each(|d| 
        encodding.extend(digit_to_encoding(d))
    );

    encodding
}

pub fn number_list_to_encoding(inp: Vec<i32>) -> Vec<f32> {
    let mut encoding: Vec<f32> = Vec::new();
    inp.into_iter().enumerate().for_each(|(i, n)| {
        // add space 
        if i != 0 {
            encoding.extend(digit_to_encoding(-2));
        }

        encoding.extend(number_to_encoding(n));
    });

    encoding
}

// TODONEXT: Make encoding -> list of numbers and test both

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
