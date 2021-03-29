// Copyright (c) 2020  The Google AI Language Team Authors, The HuggingFace Inc. team and github/lonePatient
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

use crate::common::dropout::Dropout;
use crate::mobilebert::mobilebert_model::{NormalizationLayer, NormalizationType};
use crate::mobilebert::MobileBertConfig;
use crate::RustBertError;
use std::borrow::Borrow;
use tch::nn::EmbeddingConfig;
use tch::{nn, Kind, Tensor};

pub struct MobileBertEmbeddings {
    trigram_input: bool,
    embedding_size: i64,
    hidden_size: i64,
    pub(crate) word_embeddings: nn::Embedding,
    position_embeddings: nn::Embedding,
    token_type_embeddings: nn::Embedding,
    embedding_transformation: nn::Linear,
    layer_norm: NormalizationLayer,
    dropout: Dropout,
}

impl MobileBertEmbeddings {
    pub fn new<'p, P>(p: P, config: &MobileBertConfig) -> MobileBertEmbeddings
    where
        P: Borrow<nn::Path<'p>>,
    {
        let p = p.borrow();
        let trigram_input = config.trigram_input.unwrap_or(true);
        let embedding_size = config.embedding_size;
        let hidden_size = config.hidden_size;

        let word_embeddings_config = EmbeddingConfig {
            padding_idx: config.pad_token_idx.unwrap_or(0),
            ..Default::default()
        };
        let word_embeddings = nn::embedding(
            p / "word_embeddings",
            config.vocab_size,
            embedding_size,
            word_embeddings_config,
        );
        let position_embeddings = nn::embedding(
            p / "position_embeddings",
            config.max_position_embeddings,
            hidden_size,
            Default::default(),
        );
        let token_type_embeddings = nn::embedding(
            p / "token_type_embeddings",
            config.type_vocab_size,
            hidden_size,
            Default::default(),
        );

        let embed_dim_multiplier = if trigram_input { 3 } else { 1 };
        let embedded_input_size = embedding_size * embed_dim_multiplier;
        let embedding_transformation = nn::linear(
            p / "embedding_transformation",
            embedded_input_size,
            hidden_size,
            Default::default(),
        );

        let layer_norm = NormalizationLayer::new(
            p / "LayerNorm",
            config
                .normalization_type
                .unwrap_or(NormalizationType::no_norm),
            hidden_size,
            None,
        );

        let dropout = Dropout::new(config.hidden_dropout_prob);
        MobileBertEmbeddings {
            trigram_input,
            embedding_size,
            hidden_size,
            word_embeddings,
            position_embeddings,
            token_type_embeddings,
            embedding_transformation,
            layer_norm,
            dropout,
        }
    }

    pub fn forward_t(
        &self,
        input_ids: Option<&Tensor>,
        token_type_ids: &Tensor,
        position_ids: &Tensor,
        input_embeds: Option<Tensor>,
        train: bool,
    ) -> Result<Tensor, RustBertError> {
        let (mut input_embeddings, input_shape) = match input_ids {
            Some(input_value) => match input_embeds {
                Some(_) => {
                    return Err(RustBertError::ValueError(
                        "Only one of input ids or input embeddings may be set".into(),
                    ));
                }
                None => (
                    input_value.apply_t(&self.word_embeddings, train),
                    input_value.size(),
                ),
            },
            None => match input_embeds {
                Some(embeds) => {
                    let size = vec![embeds.size()[0], embeds.size()[1]];
                    (embeds, size)
                }
                None => {
                    return Err(RustBertError::ValueError(
                        "At least one of input ids or input embeddings must be set".into(),
                    ));
                }
            },
        };

        let seq_length = input_shape[1];

        if self.trigram_input {
            let padding_tensor = Tensor::zeros(
                &[input_shape[0], 1, self.embedding_size],
                (Kind::Float, input_embeddings.device()),
            );
            input_embeddings = Tensor::cat(
                &[
                    &Tensor::cat(
                        &[
                            &input_embeddings.slice(1, 1, seq_length, 1),
                            &padding_tensor,
                        ],
                        1,
                    ),
                    &input_embeddings,
                    &Tensor::cat(
                        &[
                            &padding_tensor,
                            &input_embeddings.slice(1, 0, seq_length - 1, 1),
                        ],
                        1,
                    ),
                ],
                2,
            );
        };

        if self.trigram_input | (self.embedding_size != self.hidden_size) {
            input_embeddings = input_embeddings.apply(&self.embedding_transformation);
        }

        let position_embeddings = position_ids.apply(&self.position_embeddings);
        let token_type_embeddings = token_type_ids.apply(&self.token_type_embeddings);
        let embeddings = input_embeddings + position_embeddings + token_type_embeddings;

        Ok(self
            .layer_norm
            .forward(&embeddings)
            .apply_t(&self.dropout, train))
    }
}
