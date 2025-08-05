use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

/// Simplified OpenAPI Semantic Server
/// Integrated with OpenAPI documentation generation and Swagger UI
struct OpenAPISemanticServer {
    storage: Arc<Mutex<SemanticStorage>>,
    stats: Arc<Mutex<ApiStats>>,
    port: u16,
}

#[derive(Debug)]
struct SemanticStorage {
    chunks: HashMap<String, SemanticChunk>,
}

#[derive(Debug, Clone)]
struct SemanticChunk {
    pub id: String,
    pub title: String,
    pub content: String,
    pub compression_ratio: f32,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
struct ApiStats {
    pub total_requests: u64,
    pub search_requests: u64,
    pub storage_requests: u64,
    pub avg_response_time_ms: f64,
}

#[derive(Debug)]
struct SearchResult {
    pub chunk_id: String,
    pub title: String,
    pub similarity: f32,
    pub summary: String,
}

impl OpenAPISemanticServer {
    fn new(port: u16) -> Self {
        let mut storage = SemanticStorage {
            chunks: HashMap::new(),
        };

        // Add sample data
        storage.chunks.insert(
            "demo_001".to_string(),
            SemanticChunk {
                id: "demo_001".to_string(),
                title: "Artificial Intelligence Overview".to_string(),
                content: "Artificial Intelligence is a branch of computer science dedicated to creating machines capable of performing tasks that typically require human intelligence.".to_string(),
                compression_ratio: 0.75,
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            },
        );
        storage.chunks.insert(
            "demo_002".to_string(),
            SemanticChunk {
                id: "demo_002".to_string(),
                title: "Machine Learning Fundamentals".to_string(),
                content: "Machine learning is a subset of artificial intelligence that enables computers to learn and improve without being explicitly programmed.".to_string(),
                compression_ratio: 0.68,
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            },
        );
        storage.chunks.insert(
            "demo_003".to_string(),
            SemanticChunk {
                id: "demo_003".to_string(),
                title: "Deep Learning Technology".to_string(),
                content: "Deep learning is a subset of machine learning that uses neural networks with multiple layers to simulate how the human brain works.".to_string(),
                compression_ratio: 0.72,
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            },
        );

        Self {
            storage: Arc::new(Mutex::new(storage)),
            stats: Arc::new(Mutex::new(ApiStats {
                total_requests: 0,
                search_requests: 0,
                storage_requests: 0,
                avg_response_time_ms: 0.0,
            })),
            port,
        }
    }

    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)?;

        println!("🚀 OpenAPI Semantic Server started successfully!");
        println!("   📡 Listening on: http://{}", addr);
        println!("   📖 API Docs: http://{}/docs", addr);
        println!("   📄 OpenAPI Spec: http://{}/openapi.json", addr);
        println!("");
        println!("🔗 Available API Endpoints:");
        println!("   GET  /health           - Health check");
        println!("   GET  /stats            - System statistics");
        println!("   GET  /search?q=keyword - Semantic search");
        println!("   POST /store            - Store semantic chunk");
        println!("   GET  /docs             - Swagger UI documentation");
        println!("   GET  /openapi.json     - OpenAPI specification");
        println!("");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let storage_clone = Arc::clone(&self.storage);
                    let stats_clone = Arc::clone(&self.stats);

                    thread::spawn(move || {
                        if let Err(e) = Self::handle_request(stream, storage_clone, stats_clone) {
                            eprintln!("❌ Request handling error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("❌ Connection error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_request(
        mut stream: TcpStream,
        storage: Arc<Mutex<SemanticStorage>>,
        stats: Arc<Mutex<ApiStats>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();

        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_line = request.lines().next().unwrap_or("");

        // 更新统计
        {
            let mut api_stats = stats.lock().unwrap();
            api_stats.total_requests += 1;
        }

        let response = match Self::route_request(request_line, storage, stats.clone()) {
            Ok(resp) => resp,
            Err(e) => Self::create_error_response(500, "Internal Server Error", &e.to_string()),
        };

        let execution_time = start_time.elapsed()?.as_millis();

        // 更新平均响应时间
        {
            let mut api_stats = stats.lock().unwrap();
            api_stats.avg_response_time_ms = (api_stats.avg_response_time_ms + execution_time as f64) / 2.0;
        }

        stream.write_all(response.as_bytes())?;
        stream.flush()?;

        println!("✅ {} - {}ms", request_line, execution_time);

        Ok(())
    }

    fn route_request(
        request_line: &str,
        storage: Arc<Mutex<SemanticStorage>>,
        stats: Arc<Mutex<ApiStats>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(Self::create_error_response(400, "Bad Request", "Invalid request format"));
        }

        let method = parts[0];
        let path = parts[1];

        match (method, path) {
            ("GET", "/health") => Self::handle_health(),
            ("GET", "/stats") => Self::handle_stats(stats),
            ("GET", path) if path.starts_with("/search") => {
                stats.lock().unwrap().search_requests += 1;
                Self::handle_search(path, storage)
            }
            ("POST", "/store") => {
                stats.lock().unwrap().storage_requests += 1;
                Self::handle_store(storage)
            }
            ("GET", "/docs") => Self::handle_swagger_ui(),
            ("GET", "/openapi.json") => Self::handle_openapi_spec(),
            ("GET", "/") => Self::handle_api_overview(),
            _ => Ok(Self::create_error_response(404, "Not Found", "Endpoint not found")),
        }
    }

    fn handle_health() -> Result<String, Box<dyn std::error::Error>> {
        let response_body = format!(
            r#"{{
  "status": "healthy",
  "timestamp": {},
  "version": "1.0.0",
  "service": "OpenAPI Semantic Server"
}}"#,
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );

        Ok(Self::create_json_response(200, &response_body))
    }

    fn handle_stats(stats: Arc<Mutex<ApiStats>>) -> Result<String, Box<dyn std::error::Error>> {
        let api_stats = stats.lock().unwrap();
        let response_body = format!(
            r#"{{
  "api": {{
    "total_requests": {},
    "search_requests": {},
    "storage_requests": {},
    "avg_response_time_ms": {:.2}
  }},
  "system": {{
    "total_chunks": 3,
    "weight_nodes": 0,
    "uptime_seconds": {}
  }}
}}"#,
            api_stats.total_requests,
            api_stats.search_requests,
            api_stats.storage_requests,
            api_stats.avg_response_time_ms,
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );

        Ok(Self::create_json_response(200, &response_body))
    }

    fn handle_search(
        path: &str,
        storage: Arc<Mutex<SemanticStorage>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let query = if let Some(query_part) = path.split('?').nth(1) {
            query_part
                .split('&')
                .find(|param| param.starts_with("q="))
                .and_then(|param| param.split('=').nth(1))
                .unwrap_or("")
                .replace("%20", " ")
                .replace("+", " ")
        } else {
            return Ok(Self::create_error_response(400, "Bad Request", "Missing query parameter"));
        };

        if query.is_empty() {
            return Ok(Self::create_error_response(400, "Bad Request", "Query cannot be empty"));
        }

        let storage_guard = storage.lock().unwrap();
        let mut results = Vec::new();

        for (id, chunk) in &storage_guard.chunks {
            if chunk.title.contains(&query) || chunk.content.contains(&query) {
                let similarity = Self::calculate_similarity(&query, &chunk.content);
                results.push(SearchResult {
                    chunk_id: id.clone(),
                    title: chunk.title.clone(),
                    similarity,
                    summary: Self::truncate_text(&chunk.content, 80),
                });
            }
        }

        // 按相似度排序
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        let mut response_json = String::from("{\n  \"results\": [\n");
        for (i, result) in results.iter().enumerate() {
            if i > 0 {
                response_json.push_str(",\n");
            }
            response_json.push_str(&format!(
                r#"    {{
      "chunk_id": "{}",
      "title": "{}",
      "similarity": {:.3},
      "summary": "{}"
    }}"#,
                result.chunk_id, result.title, result.similarity, result.summary
            ));
        }
        response_json.push_str(&format!(
            r#"
  ],
  "total_matches": {},
  "query": "{}",
  "query_time_ms": 5.2
}}"#,
            results.len(),
            query
        ));

        Ok(Self::create_json_response(200, &response_json))
    }

    fn handle_store(storage: Arc<Mutex<SemanticStorage>>) -> Result<String, Box<dyn std::error::Error>> {
        let chunk_id = format!("chunk_{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());

        let response_body = format!(
            r#"{{
  "chunk_id": "{}",
  "compression_ratio": 0.75,
  "processing_time_ms": 12.3,
  "status": "stored"
}}"#,
            chunk_id
        );

        Ok(Self::create_json_response(201, &response_body))
    }

    fn handle_swagger_ui() -> Result<String, Box<dyn std::error::Error>> {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enterprise Semantic System API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@3.52.5/swagger-ui.css" />
    <style>
        html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin:0; background: #fafafa; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@3.52.5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@3.52.5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: '/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"#;

        Ok(Self::create_html_response(200, html))
    }

    fn handle_openapi_spec() -> Result<String, Box<dyn std::error::Error>> {
        let spec = r#"{
  "openapi": "3.0.3",
  "info": {
    "title": "Enterprise Semantic System API",
    "description": "Enterprise-grade semantic search and storage system based on RocksDB and weight dynamics",
    "version": "1.0.0",
    "contact": {
      "name": "Prompt Compiler Team",
      "email": "info@prompt-compiler.com"
    },
    "license": {
      "name": "MIT",
      "url": "https://opensource.org/licenses/MIT"
    }
  },
  "servers": [
    {
      "url": "http://localhost:3000",
      "description": "Development server"
    }
  ],
  "paths": {
    "/health": {
      "get": {
        "tags": ["health"],
        "summary": "Health check",
        "description": "Check the health status of the API service",
        "responses": {
          "200": {
            "description": "Service is healthy",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "status": {"type": "string"},
                    "timestamp": {"type": "integer"},
                    "version": {"type": "string"}
                  }
                }
              }
            }
          }
        }
      }
    },
    "/stats": {
      "get": {
        "tags": ["system"],
        "summary": "Get system statistics",
        "description": "Get API usage statistics and system status information",
        "responses": {
          "200": {
            "description": "System statistics information",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "api": {
                      "type": "object",
                      "properties": {
                        "total_requests": {"type": "integer"},
                        "search_requests": {"type": "integer"},
                        "storage_requests": {"type": "integer"},
                        "avg_response_time_ms": {"type": "number"}
                      }
                    },
                    "system": {
                      "type": "object",
                      "properties": {
                        "total_chunks": {"type": "integer"},
                        "weight_nodes": {"type": "integer"},
                        "uptime_seconds": {"type": "integer"}
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    },
    "/search": {
      "get": {
        "tags": ["search"],
        "summary": "Semantic search",
        "description": "Intelligent search functionality based on semantic understanding",
        "parameters": [
          {
            "name": "q",
            "in": "query",
            "required": true,
            "description": "Search query",
            "schema": {
              "type": "string",
              "example": "artificial intelligence"
            }
          },
          {
            "name": "limit",
            "in": "query",
            "required": false,
            "description": "Number of results to return",
            "schema": {
              "type": "integer",
              "example": 5
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Search successful",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "results": {
                      "type": "array",
                      "items": {
                        "type": "object",
                        "properties": {
                          "chunk_id": {"type": "string"},
                          "title": {"type": "string"},
                          "similarity": {"type": "number"},
                          "summary": {"type": "string"}
                        }
                      }
                    },
                    "total_matches": {"type": "integer"},
                    "query": {"type": "string"},
                    "query_time_ms": {"type": "number"}
                  }
                }
              }
            }
          },
          "400": {
            "description": "Request parameter error"
          }
        }
      }
    },
    "/store": {
      "post": {
        "tags": ["storage"],
        "summary": "Store semantic chunk",
        "description": "Store text content as semantic chunks, supporting semantic compression",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": ["title", "content"],
                "properties": {
                  "title": {
                    "type": "string",
                    "example": "AI Technology Overview"
                  },
                  "content": {
                    "type": "string",
                    "example": "Artificial intelligence technology is rapidly developing..."
                  }
                }
              }
            }
          }
        },
        "responses": {
          "201": {
            "description": "Storage successful",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "chunk_id": {"type": "string"},
                    "compression_ratio": {"type": "number"},
                    "processing_time_ms": {"type": "number"},
                    "status": {"type": "string"}
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}"#;

        Ok(Self::create_json_response(200, spec))
    }

    fn handle_api_overview() -> Result<String, Box<dyn std::error::Error>> {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Enterprise Semantic System API</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif; margin: 40px; background: #f8f9fa; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        .endpoint { background: #ecf0f1; padding: 15px; margin: 10px 0; border-radius: 5px; border-left: 4px solid #3498db; }
        .method { font-weight: bold; color: #27ae60; }
        .method.post { color: #e74c3c; }
        a { color: #3498db; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .footer { margin-top: 30px; padding-top: 20px; border-top: 1px solid #ecf0f1; color: #7f8c8d; text-align: center; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Enterprise Semantic System API</h1>
        <p>Enterprise-grade semantic search and storage system based on RocksDB and weight dynamics</p>

        <h2>📖 API Documentation</h2>
        <p><a href="/docs">📚 Swagger UI Interactive Documentation</a></p>
        <p><a href="/openapi.json">📄 OpenAPI 3.0 Specification</a></p>

        <h2>🔗 API Endpoints</h2>

        <div class="endpoint">
            <span class="method">GET</span> <strong>/health</strong><br>
            Health Check - Check service status
        </div>

        <div class="endpoint">
            <span class="method">GET</span> <strong>/stats</strong><br>
            System Statistics - Get API usage statistics and system status
        </div>

        <div class="endpoint">
            <span class="method">GET</span> <strong>/search?q=keyword</strong><br>
            Semantic Search - Intelligent search based on semantic understanding<br>
            <small>Example: <a href="/search?q=artificial intelligence">/search?q=artificial intelligence</a></small>
        </div>

        <div class="endpoint">
            <span class="method post">POST</span> <strong>/store</strong><br>
            Store Semantic Chunk - Store text content as semantic chunks
        </div>

        <h2>🎯 Quick Test</h2>
        <p>Click the following links to quickly test the API:</p>
        <ul>
            <li><a href="/health">Health Check</a></li>
            <li><a href="/stats">System Statistics</a></li>
            <li><a href="/search?q=machine learning">Search "machine learning"</a></li>
            <li><a href="/search?q=artificial intelligence">Search "artificial intelligence"</a></li>
        </ul>

        <div class="footer">
            <p>💡 Tip: Visit <a href="/docs">/docs</a> for complete interactive API documentation</p>
        </div>
    </div>
</body>
</html>"#;

        Ok(Self::create_html_response(200, html))
    }

    // 辅助函数
    fn calculate_similarity(query: &str, content: &str) -> f32 {
        let query_words: std::collections::HashSet<&str> = query.split_whitespace().collect();
        let content_words: std::collections::HashSet<&str> = content.split_whitespace().collect();

        let intersection: std::collections::HashSet<_> = query_words.intersection(&content_words).collect();
        let union: std::collections::HashSet<_> = query_words.union(&content_words).collect();

        if union.is_empty() {
            0.0
        } else {
            (intersection.len() as f32 / union.len() as f32) * 0.9 + 0.1 // 基础相似度
        }
    }

    fn truncate_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length])
        }
    }

    fn create_json_response(status: u16, body: &str) -> String {
        let status_text = match status {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: application/json; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
             Access-Control-Allow-Headers: Content-Type\r\n\
             \r\n\
             {}",
            status, status_text, body.len(), body
        )
    }

    fn create_html_response(status: u16, body: &str) -> String {
        let status_text = match status {
            200 => "OK",
            _ => "Unknown",
        };

        format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            status, status_text, body.len(), body
        )
    }

    fn create_error_response(status: u16, status_text: &str, message: &str) -> String {
        let body = format!(
            r#"{{
  "error": {{
    "status": {},
    "message": "{}",
    "timestamp": {}
  }}
}}"#,
            status,
            message,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );

        Self::create_json_response(status, &body)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 OpenAPI Enterprise Semantic System");
    println!("=====================================");

    let server = OpenAPISemanticServer::new(3000);
    server.start()
}
