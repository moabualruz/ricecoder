# Provider Configuration Examples

This directory contains comprehensive examples of configuring and using various AI providers with RiceCoder.

## Supported Providers

RiceCoder supports 75+ AI providers including OpenAI, Anthropic, Google, Ollama, and many others.

## OpenAI Configuration

### Basic Setup

```bash
# Configure OpenAI provider
rice provider config openai --api-key sk-your-openai-api-key-here

# Test the connection
rice provider test openai

# Set as default provider
rice provider default openai
```

### Advanced OpenAI Configuration

```yaml
# config/providers.yaml
providers:
  openai:
    api_key: "${OPENAI_API_KEY}"
    models:
      - gpt-4
      - gpt-4-turbo-preview
      - gpt-3.5-turbo
    settings:
      temperature: 0.7
      max_tokens: 4000
      top_p: 1.0
      frequency_penalty: 0.0
      presence_penalty: 0.0
    timeouts:
      connect: 10000
      read: 30000
    retry:
      max_attempts: 3
      backoff_multiplier: 2.0
```

```bash
# Configure with custom settings
rice provider config openai \
  --api-key sk-... \
  --model gpt-4 \
  --temperature 0.1 \
  --max-tokens 2000

# Use specific model for chat
rice chat --provider openai --model gpt-4-turbo-preview

# Use for code generation
rice gen --spec my-spec.md --provider openai --model gpt-4
```

### OpenAI Enterprise Setup

```yaml
# config/providers.yaml
providers:
  openai-enterprise:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://your-org.openai.azure.com/"
    api_version: "2023-12-01-preview"
    deployment_name: "gpt-4"
    models:
      - gpt-4
      - gpt-35-turbo
    enterprise:
      organization: "your-org-id"
      project: "your-project-id"
```

## Anthropic Configuration

### Claude Setup

```bash
# Configure Anthropic provider
rice provider config anthropic --api-key sk-ant-your-anthropic-key-here

# Test connection
rice provider test anthropic

# Set as default
rice provider default anthropic
```

### Advanced Claude Configuration

```yaml
# config/providers.yaml
providers:
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    models:
      - claude-3-opus-20240229
      - claude-3-sonnet-20240229
      - claude-3-haiku-20240307
    settings:
      temperature: 0.7
      max_tokens: 4000
      top_p: 1.0
      top_k: 250
    timeouts:
      connect: 10000
      read: 60000  # Claude can be slower
```

```bash
# Use Claude for creative tasks
rice chat --provider anthropic --model claude-3-opus-20240229

# Use Claude for code review
rice review src/ --provider anthropic --model claude-3-sonnet-20240229

# Use Claude for documentation
rice gen --spec docs.spec.md --provider anthropic --model claude-3-haiku-20240307
```

## Ollama Local Models

### Basic Ollama Setup

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama server
ollama serve

# Pull a model
ollama pull llama2:13b

# Configure RiceCoder to use Ollama
rice provider config ollama --base-url http://localhost:11434

# Test connection
rice provider test ollama
```

### Advanced Ollama Configuration

```yaml
# config/providers.yaml
providers:
  ollama:
    base_url: "http://localhost:11434"
    models:
      - llama2:13b
      - codellama:34b
      - mistral:7b
      - phi:2.7b
    settings:
      temperature: 0.1
      top_p: 0.9
      top_k: 40
      num_ctx: 4096
      repeat_penalty: 1.1
    performance:
      num_thread: 8
      num_gpu: 1
    timeouts:
      connect: 5000
      read: 120000  # Local models can be slow
```

```bash
# Use local model for chat
rice chat --provider ollama --model llama2:13b

# Use CodeLlama for code generation
rice gen --spec implementation.spec.md --provider ollama --model codellama:34b

# Use Mistral for analysis
rice analyze --provider ollama --model mistral:7b
```

### Ollama Model Management

```bash
# List available models
rice provider models ollama

# Pull new models
ollama pull deepseek-coder:6.7b

# Update provider configuration
rice provider config ollama --models "llama2:13b,codellama:34b,deepseek-coder:6.7b"

# Monitor model performance
rice provider monitor ollama
```

## Google AI Configuration

### Gemini Setup

```yaml
# config/providers.yaml
providers:
  google:
    api_key: "${GOOGLE_AI_API_KEY}"
    models:
      - gemini-pro
      - gemini-pro-vision
    settings:
      temperature: 0.7
      max_tokens: 2048
      top_p: 1.0
      top_k: 32
```

```bash
# Configure Google AI
rice provider config google --api-key your-google-ai-key

# Use Gemini for multimodal tasks
rice chat --provider google --model gemini-pro-vision

# Use for code generation
rice gen --spec ui-spec.md --provider google --model gemini-pro
```

## Multi-Provider Setup

### Provider Routing

```yaml
# config/providers.yaml
providers:
  routing:
    default: openai
    fallback:
      - anthropic
      - ollama
    task_routing:
      code_generation: openai
      code_review: anthropic
      chat: ollama
      analysis: google
    performance_thresholds:
      latency_ms: 5000
      success_rate: 0.95
```

```bash
# Configure provider routing
rice provider route code_generation openai
rice provider route code_review anthropic
rice provider route chat ollama

# Enable automatic failover
rice provider failover enable

# Test routing configuration
rice provider test-routing
```

### Load Balancing

```yaml
# config/providers.yaml
providers:
  load_balancing:
    strategy: "round_robin"
    providers:
      - openai
      - anthropic
      - google
    weights:
      openai: 0.5
      anthropic: 0.3
      google: 0.2
    health_checks:
      interval_seconds: 60
      timeout_seconds: 10
      failure_threshold: 3
```

```bash
# Set up load balancing
rice provider load-balance round_robin openai anthropic google

# Monitor load balancing
rice provider monitor load-balancing

# Adjust weights
rice provider weight openai 0.6
rice provider weight anthropic 0.4
```

## Enterprise Provider Configuration

### Azure OpenAI

```yaml
# config/providers.yaml
providers:
  azure-openai:
    api_key: "${AZURE_OPENAI_API_KEY}"
    base_url: "https://your-resource.openai.azure.com/"
    api_version: "2023-12-01-preview"
    deployment_name: "gpt-4"
    models:
      - gpt-4
      - gpt-35-turbo
    enterprise:
      subscription_id: "${AZURE_SUBSCRIPTION_ID}"
      resource_group: "your-resource-group"
      workspace: "your-workspace"
```

### AWS Bedrock

```yaml
# config/providers.yaml
providers:
  bedrock:
    region: "us-east-1"
    access_key_id: "${AWS_ACCESS_KEY_ID}"
    secret_access_key: "${AWS_SECRET_ACCESS_KEY}"
    models:
      - amazon.titan-text-express-v1
      - anthropic.claude-v2
      - ai21.j2-ultra-v1
    settings:
      temperature: 0.7
      max_tokens: 4000
```

### GCP Vertex AI

```yaml
# config/providers.yaml
providers:
  vertex-ai:
    project_id: "${GCP_PROJECT_ID}"
    region: "us-central1"
    credentials_file: "/path/to/service-account.json"
    models:
      - text-bison
      - chat-bison
      - codey
```

## Provider Management

### Provider Commands

```bash
# List all configured providers
rice provider list

# Show provider details
rice provider info openai

# Update provider settings
rice provider update openai --temperature 0.5

# Remove a provider
rice provider remove ollama

# Backup provider configurations
rice provider backup --output providers-backup.yaml

# Restore provider configurations
rice provider restore providers-backup.yaml
```

### Performance Monitoring

```bash
# Monitor all providers
rice provider monitor

# Monitor specific provider
rice provider monitor openai

# View performance metrics
rice provider metrics

# Generate performance report
rice provider report --output provider-performance.md

# Set up alerts
rice provider alert create --metric latency --threshold 5000ms --provider openai
```

### Cost Management

```bash
# Track provider costs
rice provider costs

# Set cost limits
rice provider limit openai --monthly-budget 100

# View cost breakdown
rice provider costs breakdown

# Optimize for cost
rice provider optimize cost
```

## Troubleshooting Providers

### Common Issues

```bash
# Test all providers
rice provider test all

# Debug provider connection
rice provider debug openai

# Check provider logs
rice provider logs openai

# Reset provider configuration
rice provider reset openai

# Validate provider settings
rice provider validate
```

### Network Issues

```bash
# Configure proxy
rice provider proxy set http://proxy.company.com:8080

# Test connectivity
rice network test api.openai.com

# Configure timeouts
rice provider timeout openai 30000

# Use custom DNS
rice provider dns 8.8.8.8
```

### Authentication Issues

```bash
# Rotate API keys
rice provider rotate-key openai sk-new-key-here

# Test authentication
rice provider auth-test openai

# View authentication logs
rice provider auth-logs openai

# Configure OAuth
rice provider oauth configure openai
```

## Best Practices

### Security

```bash
# Use environment variables for API keys
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Enable API key encryption
rice provider encrypt-keys enable

# Set up key rotation
rice provider key-rotation enable --interval 30d

# Audit provider access
rice provider audit enable
```

### Performance

```bash
# Configure connection pooling
rice provider pool configure --max-connections 10

# Enable response caching
rice provider cache enable --ttl 300

# Set up request batching
rice provider batch enable --size 5

# Monitor and optimize
rice provider optimize performance
```

### Reliability

```bash
# Configure retry policies
rice provider retry configure --max-attempts 3 --backoff 2.0

# Set up circuit breakers
rice provider circuit-breaker enable --failure-threshold 5

# Configure health checks
rice provider health-check enable --interval 60

# Set up failover
rice provider failover configure --fallback-providers anthropic,ollama
```

This comprehensive guide covers provider configuration from basic setup to advanced enterprise scenarios, ensuring optimal performance and reliability across all supported AI providers.