// Copyright 2018 Google AI and Google Brain team.
// Copyright 2018 Carnegie Mellon University Authors.
// Copyright 2020-present, the HuggingFace Inc. team.
// Copyright 2020 Guillaume Becquin
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

use rust_bert::reformer::{
    ReformerConfig, ReformerConfigResources, ReformerModelResources, ReformerModelWithLMHead,
    ReformerVocabResources,
};
use rust_bert::resources::{RemoteResource, Resource};
use rust_bert::Config;
use rust_tokenizers::tokenizer::{MultiThreadedTokenizer, ReformerTokenizer, TruncationStrategy};
use tch::{nn, Device, Tensor};

fn main() -> anyhow::Result<()> {
    //    Resources paths
    let config_resource = Resource::Remote(RemoteResource::from_pretrained(
        ReformerConfigResources::CRIME_AND_PUNISHMENT,
    ));
    let vocab_resource = Resource::Remote(RemoteResource::from_pretrained(
        ReformerVocabResources::CRIME_AND_PUNISHMENT,
    ));
    let weights_resource = Resource::Remote(RemoteResource::from_pretrained(
        ReformerModelResources::CRIME_AND_PUNISHMENT,
    ));
    let config_path = config_resource.get_local_path()?;
    let vocab_path = vocab_resource.get_local_path()?;
    let weights_path = weights_resource.get_local_path()?;

    //    Set-up masked LM model
    let device = Device::cuda_if_available();
    let mut vs = nn::VarStore::new(device);
    let tokenizer = ReformerTokenizer::from_file(vocab_path.to_str().unwrap(), false)?;
    let config = ReformerConfig::from_file(config_path);
    let reformer_model = ReformerModelWithLMHead::new(&vs.root(), &config)?;
    vs.load(weights_path)?;

    //    Define input
    let input = ["One two three four five six seven eight nine ten eleven One two three four five six seven eight nine ten eleven One two three four five six seven eight nine ten eleven"];
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
    let _model_output =
        reformer_model.forward_t(Some(&input_tensor), None, None, None, None, None, false)?;

    _model_output.logits.print();
    Ok(())
}
