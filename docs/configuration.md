# Configuration Guide 🔧

This guide covers the configuration options for the Prompt Compiler system.

## 📄 Configuration Files

### Main Configuration (`config.toml`)

The main configuration file supports the following sections:

```toml
[compiler]
# Compilation engine settings
optimization_level = "high"  # low, medium, high, aggressive
enable_weight_analysis = true
max_context_length = 4096
compression_threshold = 0.7

[storage]
# RocksDB storage settings
db_path = "./prompt_compiler.db"
max_open_files = 1000
write_buffer_size = "64MB"
compression = "lz4"
cache_size = "256MB"

[crypto]
# Cryptographic verification settings
enable_signing = true
signature_algorithm = "ed25519"
key_path = "./keys/"

[web]
# Web server settings
host = "127.0.0.1"
port = 3000
cors_enabled = true
rate_limit = 1000  # requests per minute

[embedding]
# Embedding generation settings
model = "text-embedding-3-large"
dimension = 3072
batch_size = 100
cache_enabled = true
```

## 🌍 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENAI_API_KEY` | - | OpenAI API key for embedding generation |
| `OPENAI_MODEL` | `text-embedding-3-large` | Embedding model to use |
| `PC_CONFIG_PATH` | `./config.toml` | Path to configuration file |
| `PC_LOG_LEVEL` | `info` | Logging level (trace, debug, info, warn, error) |
| `PC_STORAGE_PATH` | `./prompt_compiler.db` | RocksDB storage path |

## 🚀 Performance Tuning

### Weight Dynamics Optimization

```toml
[weight_dynamics]
# ICL weight update settings
max_iterations = 100
convergence_threshold = 1e-4
learning_rate = 0.01
rank_limit = 64
batch_processing = true
```

### Storage Optimization

```toml
[storage.rocksdb]
# High-performance settings for production
write_buffer_size = "128MB"
max_write_buffer_number = 4
target_file_size_base = "64MB"
max_bytes_for_level_base = "512MB"
compression = "zstd"
bloom_filter_bits = 10
```

### Memory Management

```toml
[memory]
# Memory usage limits
max_embedding_cache_size = "1GB"
max_context_cache_size = "512MB"
gc_threshold = 0.8
preload_embeddings = false
```

## 🔧 Development Settings

### Debug Configuration

```toml
[debug]
enable_profiling = true
detailed_logging = true
save_intermediate_results = true
benchmark_mode = false
```

### Testing Configuration

```toml
[testing]
use_mock_embeddings = true
deterministic_mode = true
test_data_path = "./test_data/"
```

## 📊 Monitoring & Metrics

### Metrics Collection

```toml
[metrics]
enable_metrics = true
metrics_endpoint = "/metrics"
collection_interval = "30s"
export_format = "prometheus"
```

### Performance Monitoring

```toml
[monitoring]
track_compilation_time = true
track_weight_convergence = true
track_storage_performance = true
alert_thresholds = {
    compilation_time_ms = 1000,
    convergence_iterations = 150,
    storage_latency_ms = 10
}
```

## 🔐 Security Configuration

### API Security

```toml
[security]
enable_auth = true
jwt_secret = "your-secret-key"
token_expiry = "24h"
rate_limiting = true
```

### Data Protection

```toml
[encryption]
encrypt_storage = true
encryption_algorithm = "aes-256-gcm"
key_rotation_interval = "30d"
```

## 🌐 Deployment Configurations

### Production Setup

```toml
# production.toml
[compiler]
optimization_level = "aggressive"
enable_weight_analysis = true

[storage]
db_path = "/var/lib/prompt-compiler/db"
compression = "zstd"
cache_size = "2GB"

[web]
host = "0.0.0.0"
port = 8080
cors_enabled = false
```

### Development Setup

```toml
# development.toml
[compiler]
optimization_level = "medium"
enable_weight_analysis = true

[debug]
enable_profiling = true
detailed_logging = true

[web]
host = "127.0.0.1"
port = 3000
cors_enabled = true
```

## 📝 Configuration Validation

The system automatically validates configuration on startup. Common issues:

- Invalid optimization levels
- Insufficient memory allocations
- Missing API keys
- Invalid file paths
- Network configuration conflicts

## 🔄 Hot Reloading

Some configuration changes can be applied without restart:

- Log levels
- Rate limits
- Cache sizes
- Monitoring settings

Use the admin API to reload configuration:

```bash
curl -X POST http://localhost:3000/admin/reload-config
```
