# Model Manager Skill

## Description
Manage local LLMs via Ollama — download, serve, switch, and monitor models. Supports GPU acceleration, cloud fallback, and Hermes agent model lifecycle.

## Triggers
- User wants to use a different AI model
- Agent needs a model for a task
- Resource optimization events

## Actions

### list_models()
List available local models:
- Name, size, quantization, parameter count
- Currently loaded/running status
- GPU memory usage per model

### pull_model(name, quantization="q4_k_m")
Download a model from Ollama registry:
- `name`: model name (e.g., "llama3.2", "hermes-3-llama-3.1")
- `quantization`: q4_k_m, q5_k_m, q8_0, f16
- Streams download progress

### serve_model(name, ctx_len=8192, gpu_layers=-1)
Load and serve a model:
- `ctx_len`: context window size
- `gpu_layers`: GPU offload layers (-1 = auto)
- Returns endpoint URL and status

### switch_model(name)
Hot-swap the active Hermes agent model:
- Gracefully unloads current model
- Loads new model from cache or pulls if missing
- Updates agent configuration

### get_gpu_info()
Query GPU state for model serving:
- GPU model, VRAM total/used, temperature
- CUDA/NVIDIA driver version (if applicable)
- ROCm/HIP info (AMD)

### unload_model(name)
Free resources by unloading a model:
- Stops Ollama process for the model
- Frees GPU and system memory

### set_fallback(chain)
Configure cloud fallback chain:
- `chain`: list of providers in priority order (e.g., ["ollama", "openai", "anthropic"])
- Each provider has model, API key ref, and timeout

## Example
User: "Switch to a coding model"
Agent: *calls list_models, finds deepseek-coder, calls switch_model*

## Dependencies
- Ollama service (MCP endpoint)
- NVIDIA/CUDA or AMD ROCm drivers (optional)
- Sufficient disk space for model storage
