# RiceGrep AI/ML Dependencies Setup Guide

RiceGrep uses advanced AI/ML libraries for enhanced code search capabilities. This guide explains how to set up the required dependencies for optimal performance, including GPU acceleration support.

## Current Status

**⚠️ WORK IN PROGRESS**: AI features are currently optional. PyTorch installation is required for full AI functionality.

## Required Dependencies

### PyTorch/libtorch (Required for AI Features)

RiceGrep uses `rust-bert` which depends on PyTorch's C++ library (libtorch) for neural network computations.

**Version Required**: PyTorch v2.0.0 (compatible with rust-bert's tch dependency)

#### Installation Options

##### Option 1: pip Installation (Recommended)

```bash
# CPU-only version (works immediately)
pip install torch==2.0.0 torchvision==0.15.0 torchaudio==2.0.0 --index-url https://download.pytorch.org/whl/cpu

# CUDA version (for GPU acceleration) - requires CUDA-compatible GPU
pip install torch==2.0.0 torchvision==0.15.0 torchaudio==2.0.0 --index-url https://download.pytorch.org/whl/cu118
```

**Note**: CUDA version requires ~2.6GB download and NVIDIA GPU with CUDA support.

##### Option 2: Manual libtorch Installation

1. **Download libtorch from PyTorch:**
   - CPU: https://download.pytorch.org/libtorch/cpu/libtorch-win-shared-with-deps-2.0.0%2Bcpu.zip
   - CUDA: https://download.pytorch.org/libtorch/cu118/libtorch-win-shared-with-deps-2.0.0%2Bcu118.zip

2. **Extract and set environment variables:**
   ```bash
   # Windows (PowerShell)
   Expand-Archive libtorch-win-shared-with-deps-2.0.0+cpu.zip -DestinationPath C:\libtorch
   $env:LIBTORCH = "C:\libtorch"
   $env:Path += ";C:\libtorch\lib"
   ```

#### GPU Acceleration Setup

For GPU acceleration on systems with CUDA-compatible GPUs:

1. **Install CUDA version of PyTorch:**
   ```bash
   pip install torch==2.0.0 torchvision==0.15.0 torchaudio==2.0.0 --index-url https://download.pytorch.org/whl/cu118
   ```

2. **Verify CUDA availability:**
   ```python
   import torch
   print(f"CUDA available: {torch.cuda.is_available()}")
   print(f"CUDA version: {torch.version.cuda}")
   print(f"GPU count: {torch.cuda.device_count()}")
   ```

**Requirements for CUDA:**
- NVIDIA GPU with CUDA compute capability 6.0+
- CUDA Toolkit 11.8+ installed
- cuDNN (included in PyTorch distribution)

#### Verification

Test your PyTorch installation:
```python
import torch
print(f"PyTorch version: {torch.__version__}")
print(f"CUDA available: {torch.cuda.is_available()}")
```

Test RiceGrep AI features:
```bash
cd projects/ricecoder
cargo build --features rust-bert
cargo run --bin ricegrep --features rust-bert -- --help
```

## Performance Notes

### CPU vs GPU Performance

- **CPU**: Good for development, slower inference
- **GPU**: 5-10x faster inference for AI features
- **Memory**: GPU requires ~2-4GB VRAM for models

### Model Loading

- First run downloads models (~250MB for DistilBERT)
- Models cached in `~/.cache/.rustbert`
- Override cache location: `export RUSTBERT_CACHE=/custom/path`

### Troubleshooting

**Common Issues:**

1. **"libtorch not found"**
   - Ensure `LIBTORCH` environment variable is set
   - Check `LD_LIBRARY_PATH` (Linux/macOS) or `Path` (Windows)

2. **CUDA errors**
   - Verify CUDA installation: `nvidia-smi`
   - Check CUDA version compatibility
   - Ensure GPU drivers are up to date

3. **Build failures**
   - Clear cargo cache: `cargo clean`
   - Delete target directory
   - Rebuild with fresh libtorch download

**Performance Optimization:**
- Use GPU for production deployments
- Pre-load models in application startup
- Monitor memory usage with large codebases

## Building with AI Features

### Enable AI Features in Cargo

To build RiceGrep with AI capabilities:

```bash
# Build with AI features
cargo build --features rust-bert

# Run with AI features
cargo run --bin ricegrep --features rust-bert -- [args]
```

### Environment Variables

When using manual libtorch installation:

```bash
# Windows
set LIBTORCH=C:\path\to\libtorch
set LIBTORCH_USE_PYTORCH=1  # Use Python PyTorch instead of manual libtorch

# Linux/macOS
export LIBTORCH=/path/to/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
export LIBTORCH_USE_PYTORCH=1
```

### Optional Environment Variables

```bash
# Custom model cache location
export RUSTBERT_CACHE=/custom/cache/path

# Force CPU-only operation (even with CUDA PyTorch installed)
export CUDA_VISIBLE_DEVICES=""
```

## Performance Optimization

### CPU vs GPU Performance

- **CPU**: 1-2x baseline performance, works on all systems
- **GPU**: 5-10x faster inference, requires CUDA-compatible GPU
- **Memory**: GPU models require 2-4GB VRAM

### Model Loading

- First run downloads models (~250MB for DistilBERT)
- Models cached in `~/.cache/.rustbert` (or custom location)
- Pre-load models for better startup performance

## Troubleshooting

### Common Issues

1. **"torch-sys build failed"**
   - Ensure PyTorch 2.0.0 is installed
   - Set `LIBTORCH_USE_PYTORCH=1` to use Python installation
   - Check NumPy compatibility (< 2.0)

2. **CUDA not available**
   - Verify GPU has CUDA support
   - Check CUDA toolkit installation
   - Ensure correct PyTorch CUDA version

3. **Import errors**
   - Reinstall PyTorch: `pip install --force-reinstall torch`
   - Check NumPy version compatibility
   - Clear Python cache: `pip cache purge`

4. **Build failures**
   - Clean build: `cargo clean`
   - Update dependencies: `cargo update`
   - Check Rust version (1.70+ required)

### Verification Commands

```bash
# Check PyTorch
python -c "import torch; print(torch.__version__, torch.cuda.is_available())"

# Check CUDA
nvidia-smi

# Test build
cargo check --features rust-bert
```

## Next Steps

1. Install PyTorch 2.0.0 (CPU or CUDA)
2. Set environment variables if using manual libtorch
3. Build with `--features rust-bert`
4. Test AI-enhanced search functionality

For questions or issues, check the rust-bert documentation: https://github.com/guillaume-be/rust-bert