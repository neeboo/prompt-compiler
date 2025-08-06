# PC Node Testing Scripts

This directory contains testing scripts and utilities for validating the Prompt Compiler Node functionality.

## 🧪 Test Scripts

### `test_pc_node.py`

A comprehensive testing script that validates all core PC Node functionalities.

#### Features Tested

1. **Health Check** - Basic service availability
2. **OpenAI API Compatibility** - Standard chat completion interface
3. **Context Sharing** - Single agent context preservation across conversations
4. **Multi-Agent Context Sharing** - Knowledge sharing between different agents
5. **Semantic Compression** - Context compression and optimization
6. **Performance Metrics** - Response time and token usage analysis
7. **Error Handling** - Proper error responses for invalid inputs

#### Usage

```bash
# Install dependencies
pip install requests

# Run all tests
python test_pc_node.py

# Run specific test
python test_pc_node.py --test health
python test_pc_node.py --test openai
python test_pc_node.py --test context
python test_pc_node.py --test multi-agent
python test_pc_node.py --test compression
python test_pc_node.py --test performance
python test_pc_node.py --test errors

# Test against different endpoint
python test_pc_node.py --url http://production-server:3000
```

#### Expected Output

```
🚀 Starting PC Node Comprehensive Tests
==================================================

🔍 Testing health endpoint...
✅ Health check passed: {'status': 'healthy', 'version': '1.0.0'}
✅ Health Check: PASSED

🔍 Testing OpenAI API compatibility...
✅ OpenAI API compatible response received
   Model: gpt-3.5-turbo
   Content: Hello! Yes, I'm working and ready to help you...
   Tokens: 45
✅ OpenAI Compatibility: PASSED

🔍 Testing Context Sharing...
   📝 First conversation (establishing context)...
   ✅ Context established: Hello Alice! I'd be happy to help you with your Python machine learning project...
   🔄 Second conversation (testing context reuse)...
   ✅ Context sharing working: Your name is Alice and you're working on a Python project about machine learning...
   📊 Token usage comparison:
      First: 89 tokens
      Second: 67 tokens
✅ Context Sharing: PASSED

📊 Test Results Summary
==================================================
   ✅ PASS: Health Check
   ✅ PASS: OpenAI Compatibility
   ✅ PASS: Context Sharing
   ✅ PASS: Multi-Agent Sharing
   ✅ PASS: Semantic Compression
   ✅ PASS: Performance Metrics
   ✅ PASS: Error Handling

🎯 Overall Success Rate: 7/7 (100.0%)
🎉 PC Node is working well!
```

## 🔧 Test Configuration

### Environment Variables

```bash
# Optional: Set OpenAI API key for backend testing
export OPENAI_API_KEY="your-api-key-here"

# Optional: Set custom PC Node endpoint
export PC_NODE_URL="http://localhost:3000"
```

### Custom Headers

The test script supports PC-specific headers:

- `X-PC-Context-Share`: Enable context sharing
- `X-PC-Agent-ID`: Specify agent identifier
- `X-PC-Agent-Role`: Define agent role for multi-agent scenarios
- `X-PC-Context-Compress`: Enable semantic compression

## 📊 Performance Benchmarks

The performance test provides metrics on:

- **Response Time**: Average time to process requests
- **Token Usage**: Input/output token consumption
- **Success Rate**: Percentage of successful requests
- **Throughput**: Requests processed per second

## 🐛 Troubleshooting

### Common Issues

1. **Connection Refused**
   ```
   ❌ Health check error: Connection refused
   ```
   - Ensure PC Node is running on the specified port
   - Check firewall settings

2. **Authentication Errors**
   ```
   ❌ OpenAI API test failed: 401
   ```
   - Verify OPENAI_API_KEY is set correctly
   - Check API key permissions

3. **Context Not Preserved**
   ```
   ❌ Context not preserved in response
   ```
   - Verify context sharing is enabled in PC Node
   - Check storage backend connectivity

### Debug Mode

Add debug output by modifying the script:

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

## 🚀 Continuous Integration

Integrate with CI/CD pipelines:

```yaml
# .github/workflows/test.yml
- name: Test PC Node
  run: |
    python scripts/test_pc_node.py --url http://localhost:3000
```

## 📈 Metrics Collection

The test script can output metrics in JSON format for monitoring:

```bash
python test_pc_node.py --format json > test_results.json
```

This enables integration with monitoring systems like Prometheus, Grafana, or custom dashboards.
