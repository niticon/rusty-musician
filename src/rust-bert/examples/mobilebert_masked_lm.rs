// Copyright 2019-present, the HuggingFace Inc. team, The Google AI Language Team and Facebook, Inc.
// Copyright 2019 Guillaume Becquin
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//     http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate anyhow;

use rust_bert::mobilebert::{
    MobileBertConfig, MobileBertConfigResources, MobileBertForMaskedLM, MobileBertModelResources,
    MobileBertVocabResources,
};
use rust_bert::resources::{RemoteResource, Resource};
use rust_bert::Config;
use rust_tokenizers::tokenizer::{BertTokenizer, MultiThreadedTokenizer, TruncationStrategy};
use rust_tokenizers::vocab::Vocab;
use tch::{nn, no_grad, Device, Tensor};

fn main() -> anyhow::Result<()> {
    //    Resources paths
    let config_resource = Resource::Remote(RemoteResource::from_pretrained(
        MobileBertConfigResources::MOBILEBERT_UNCASED,
    ));
    let vocab_resource = Resource::Remote(RemoteResource::from_pretrained(
        MobileBertVocabResources::MOBILEBERT_UNCASED,
    ));
    let weights_resource = Resource::Remote(RemoteResource::from_pretrained(
        MobileBertModelResources::MOBILEBERT_UNCASED,
    ));
    let config_path = config_resource.get_local_path()?;
    let vocab_path = vocab_resource.get_local_path()?;
    let weights_path = weights_resource.get_local_path()?;

    //    Set-up masked LM model
    let device = Device::Cpu;
    let mut vs = nn::VarStore::new(device);
    let tokenizer: BertTokenizer =
        BertTokenizer::from_file(vocab_path.to_str().unwrap(), true, true)?;
    let config = MobileBertConfig::from_file(config_path);
    let mobilebert_model = MobileBertForMaskedLM::new(&vs.root(), &config);
    vs.load(weights_path)?;

    //    Define input
    let input = [
        "Looks like one [MASK] is missing",
        "It was a very nice and [MASK] day",
    ];
    let tokenized_input = tokenizer.encode_list(&input, 128, &TruncationStrategy::LongestFirst, 0);
    let max_len = tokenized_input
        .iter()
        .map(|input| input.token_ids.len())
        .max()
        .unwrap();
    let tokenized_input = tokenized_input
        .iter()
        .map(|input| input.token_ids.clone())
        .map(|mut input| {
            input.extend(vec![0; max_len - input.len()]);
            input
        })
        .map(|input| Tensor::of_slice(&(input)))
        .collect::<Vec<_>>();
    let input_tensor = Tensor::stack(tokenized_input.as_slice(), 0).to(device);

    //    Forward pass
    let model_output =
        no_grad(|| mobilebert_model.forward_t(Some(&input_tensor), None, None, None, None, false))?;

    //    Print masked tokens
    let index_1 = model_output.logits.get(0).get(4).argmax(0, false);
    let index_2 = model_output.logits.get(1).get(7).argmax(0, false);
    let word_1 = tokenizer.vocab().id_to_token(&index_1.int64_value(&[]));
    let word_2 = tokenizer.vocab().id_to_token(&index_2.int64_value(&[]));

    println!("{}", word_1); // Outputs "thing" : "Looks like one [thing] is missing"
    println!(
        "score: {}",
        model_output
            .logits
            .get(0)
            .get(4)
            .double_value(&[i64::from(&index_1)])
    ); // 10.0558

    println!("{}", word_2); // Outputs "sunny" : "It was a very nice and [sunny] day"
    println!(
        "score: {}",
        model_output
            .logits
            .get(1)
            .get(7)
            .double_value(&[i64::from(&index_2)])
    ); // 14.2708
    Ok(())
}
