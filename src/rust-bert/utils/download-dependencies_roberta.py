from transformers import ROBERTA_PRETRAINED_CONFIG_ARCHIVE_MAP
from transformers.tokenization_roberta import PRETRAINED_VOCAB_FILES_MAP
from transformers.file_utils import get_from_cache, hf_bucket_url
from pathlib import Path
import shutil
import os
import numpy as np
import torch
import subprocess

config_path = ROBERTA_PRETRAINED_CONFIG_ARCHIVE_MAP["roberta-base"]
vocab_path = PRETRAINED_VOCAB_FILES_MAP["vocab_file"]["roberta-base"]
merges_path = PRETRAINED_VOCAB_FILES_MAP["merges_file"]["roberta-base"]
weights_path = "roberta-base"

target_path = Path.home() / 'rustbert' / 'roberta'

temp_config = get_from_cache(config_path)
temp_vocab = get_from_cache(vocab_path)
temp_merges = get_from_cache(merges_path)
temp_weights = get_from_cache(hf_bucket_url(weights_path, filename="pytorch_model.bin", use_cdn=True))

os.makedirs(str(target_path), exist_ok=True)

config_path = str(target_path / 'config.json')
vocab_path = str(target_path / 'vocab.txt')
merges_path = str(target_path / 'merges.txt')
model_path = str(target_path / 'model.bin')

shutil.copy(temp_config, config_path)
shutil.copy(temp_vocab, vocab_path)
shutil.copy(temp_merges, merges_path)
shutil.copy(temp_weights, model_path)

weights = torch.load(temp_weights, map_location='cpu')
nps = {}
for k, v in weights.items():
    k = k.replace("gamma", "weight").replace("beta", "bias")
    nps[k] = np.ascontiguousarray(v.cpu().numpy())

np.savez(target_path / 'model.npz', **nps)

source = str(target_path / 'model.npz')
target = str(target_path / 'model.ot')

toml_location = (Path(__file__).resolve() / '..' / '..' / 'Cargo.toml').resolve()

subprocess.call(
    ['cargo', 'run', '--bin=convert-tensor', '--manifest-path=%s' % toml_location, '--', source, target])

os.remove(str(target_path / 'model.bin'))
os.remove(str(target_path / 'model.npz'))