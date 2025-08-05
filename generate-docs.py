#!/usr/bin/env python3
"""
DuxNet API Documentation Generator
Generates comprehensive API documentation for production deployment
"""

import json
import argparse
from typing import Dict, List, Any

def generate_api_docs() -> Dict[str, Any]:
    """Generate comprehensive API documentation"""
    return {
        "openapi": "3.0.0",
        "info": {
            "title": "DuxNet API - Decentralized API Marketplace",
            "description": "Complete API for the DuxNet decentralized API marketplace platform",
            "version": "1.0.0",
            "contact": {
                "name": "DuxNet Team",
                "url": "https://duxnet.io",
                "email": "api@duxnet.io"
            },
            "license": {
                "name": "MIT",
                "url": "https://opensource.org/licenses/MIT"
            }
        },
        "servers": [
            {
                "url": "http://localhost:8081",
                "description": "Development server"
            },
            {
                "url": "https://api.duxnet.io",
                "description": "Production server"
            }
        ],
        "security": [
            {
                "ApiKeyAuth": []
            }
        ],
        "components": {
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key"
                }
            },
            "schemas": {
                "Error": {
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean", "example": False},
                        "message": {"type": "string", "example": "Error description"}
                    }
                },
                "Success": {
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean", "example": True},
                        "message": {"type": "string", "example": "Operation successful"}
                    }
                },
                "DuxBalance": {
                    "type": "object",
                    "properties": {
                        "success": {"type": "boolean"},
                        "balance": {"type": "number", "format": "double"},
                        "confirmed": {"type": "number", "format": "double"},
                        "unconfirmed": {"type": "number", "format": "double"},
                        "address": {"type": "string"},
                        "currency": {"type": "string", "example": "DUX"}
                    }
                },
                "DuxTransaction": {
                    "type": "object",
                    "properties": {
                        "txid": {"type": "string"},
                        "amount": {"type": "number", "format": "double"},
                        "confirmations": {"type": "integer"},
                        "address": {"type": "string"},
                        "time": {"type": "integer", "format": "int64"},
                        "category": {"type": "string"}
                    }
                }
            }
        },
        "paths": {
            "/api/status": {
                "get": {
                    "summary": "Get node status",
                    "description": "Returns the current status of the DuxNet node",
                    "responses": {
                        "200": {
                            "description": "Node status information",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "node_id": {"type": "string"},
                                            "did": {"type": "string"},
                                            "is_online": {"type": "boolean"},
                                            "uptime_seconds": {"type": "integer"},
                                            "services_count": {"type": "integer"},
                                            "reputation_score": {"type": "number"},
                                            "peers_count": {"type": "integer"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/dux/balance": {
                "get": {
                    "summary": "Get DUX balance",
                    "description": "Get the current DUX balance for the node's wallet",
                    "security": [{"ApiKeyAuth": []}],
                    "responses": {
                        "200": {
                            "description": "DUX balance information",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/DuxBalance"}
                                }
                            }
                        },
                        "400": {
                            "description": "Error getting balance",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Error"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/dux/send": {
                "post": {
                    "summary": "Send DUX coins",
                    "description": "Send DUX coins to another address",
                    "security": [{"ApiKeyAuth": []}],
                    "requestBody": {
                        "required": True,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["to_address", "amount"],
                                    "properties": {
                                        "to_address": {
                                            "type": "string",
                                            "description": "Destination DUX address"
                                        },
                                        "amount": {
                                            "type": "number",
                                            "format": "double",
                                            "description": "Amount to send in DUX"
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Transaction sent successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": {"type": "boolean"},
                                            "txid": {"type": "string"},
                                            "from_address": {"type": "string"},
                                            "to_address": {"type": "string"},
                                            "amount": {"type": "number"},
                                            "message": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Invalid request or insufficient funds",
                            "content": {
                                "application/json": {
                                    "schema": {"$ref": "#/components/schemas/Error"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/dux/transactions": {
                "get": {
                    "summary": "Get DUX transactions",
                    "description": "Get transaction history for the node's DUX wallet",
                    "security": [{"ApiKeyAuth": []}],
                    "responses": {
                        "200": {
                            "description": "Transaction history",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": {"type": "boolean"},
                                            "transactions": {
                                                "type": "array",
                                                "items": {"$ref": "#/components/schemas/DuxTransaction"}
                                            },
                                            "address": {"type": "string"},
                                            "count": {"type": "integer"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/dux/network": {
                "get": {
                    "summary": "Get DUX network info",
                    "description": "Get information about the DUX network",
                    "responses": {
                        "200": {
                            "description": "Network information",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": {"type": "boolean"},
                                            "network": {
                                                "type": "object",
                                                "properties": {
                                                    "difficulty": {"type": "number"},
                                                    "block_height": {"type": "integer"},
                                                    "connections": {"type": "integer"},
                                                    "hash_rate": {"type": "number"}
                                                }
                                            },
                                            "status": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/services/register": {
                "post": {
                    "summary": "Register a new service",
                    "description": "Register a new API service in the marketplace",
                    "security": [{"ApiKeyAuth": []}],
                    "requestBody": {
                        "required": True,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["name", "description", "endpoint", "price"],
                                    "properties": {
                                        "name": {"type": "string"},
                                        "description": {"type": "string"},
                                        "endpoint": {"type": "string"},
                                        "price": {"type": "number"},
                                        "category": {"type": "string"},
                                        "tags": {"type": "array", "items": {"type": "string"}}
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Service registered successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "service_id": {"type": "string"},
                                            "success": {"type": "boolean"},
                                            "message": {"type": "string"},
                                            "api_key": {"type": "string"},
                                            "documentation_url": {"type": "string"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/services/search": {
                "post": {
                    "summary": "Search for services",
                    "description": "Search for services in the marketplace",
                    "requestBody": {
                        "required": True,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "query": {"type": "string"},
                                        "category": {"type": "string"},
                                        "limit": {"type": "integer"},
                                        "offset": {"type": "integer"}
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Search results",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "services": {"type": "array"},
                                            "total_count": {"type": "integer"},
                                            "success": {"type": "boolean"},
                                            "message": {"type": "string"},
                                            "pagination": {"type": "object"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/wallet/backup": {
                "post": {
                    "summary": "Backup wallet",
                    "description": "Create a backup of the wallet",
                    "security": [{"ApiKeyAuth": []}],
                    "responses": {
                        "200": {
                            "description": "Wallet backed up successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": {"type": "boolean"},
                                            "backup_data": {"type": "string"},
                                            "message": {"type": "string"},
                                            "timestamp": {"type": "integer"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/analytics/usage": {
                "get": {
                    "summary": "Get usage analytics",
                    "description": "Get API usage analytics",
                    "security": [{"ApiKeyAuth": []}],
                    "parameters": [
                        {
                            "name": "hours",
                            "in": "query",
                            "description": "Number of hours to look back",
                            "schema": {"type": "integer", "default": 24}
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Usage analytics data",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "success": {"type": "boolean"},
                                            "period_hours": {"type": "integer"},
                                            "total_requests": {"type": "integer"},
                                            "endpoint_analytics": {"type": "array"}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

def generate_postman_collection() -> Dict[str, Any]:
    """Generate Postman collection for API testing"""
    return {
        "info": {
            "name": "DuxNet API Collection",
            "description": "Complete API collection for DuxNet testing",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "variable": [
            {
                "key": "baseUrl",
                "value": "http://localhost:8081",
                "type": "string"
            },
            {
                "key": "apiKey",
                "value": "demo-api-key-123",
                "type": "string"
            }
        ],
        "item": [
            {
                "name": "Node Status",
                "request": {
                    "method": "GET",
                    "header": [],
                    "url": {
                        "raw": "{{baseUrl}}/api/status",
                        "host": ["{{baseUrl}}"],
                        "path": ["api", "status"]
                    }
                }
            },
            {
                "name": "Get DUX Balance",
                "request": {
                    "method": "GET",
                    "header": [
                        {
                            "key": "X-API-Key",
                            "value": "{{apiKey}}",
                            "type": "text"
                        }
                    ],
                    "url": {
                        "raw": "{{baseUrl}}/api/dux/balance",
                        "host": ["{{baseUrl}}"],
                        "path": ["api", "dux", "balance"]
                    }
                }
            },
            {
                "name": "Send DUX",
                "request": {
                    "method": "POST",
                    "header": [
                        {
                            "key": "X-API-Key",
                            "value": "{{apiKey}}",
                            "type": "text"
                        },
                        {
                            "key": "Content-Type",
                            "value": "application/json",
                            "type": "text"
                        }
                    ],
                    "body": {
                        "mode": "raw",
                        "raw": json.dumps({
                            "to_address": "DUX_ADDRESS_HERE",
                            "amount": 1.0
                        }, indent=2)
                    },
                    "url": {
                        "raw": "{{baseUrl}}/api/dux/send",
                        "host": ["{{baseUrl}}"],
                        "path": ["api", "dux", "send"]
                    }
                }
            },
            {
                "name": "Register Service",
                "request": {
                    "method": "POST",
                    "header": [
                        {
                            "key": "X-API-Key",
                            "value": "{{apiKey}}",
                            "type": "text"
                        },
                        {
                            "key": "Content-Type",
                            "value": "application/json",
                            "type": "text"
                        }
                    ],
                    "body": {
                        "mode": "raw",
                        "raw": json.dumps({
                            "name": "Test Service",
                            "description": "A test API service",
                            "endpoint": "https://api.example.com",
                            "price": 0.01,
                            "category": "ai",
                            "tags": ["test", "api"]
                        }, indent=2)
                    },
                    "url": {
                        "raw": "{{baseUrl}}/api/services/register",
                        "host": ["{{baseUrl}}"],
                        "path": ["api", "services", "register"]
                    }
                }
            },
            {
                "name": "Search Services",
                "request": {
                    "method": "POST",
                    "header": [
                        {
                            "key": "Content-Type",
                            "value": "application/json",
                            "type": "text"
                        }
                    ],
                    "body": {
                        "mode": "raw",
                        "raw": json.dumps({
                            "query": "ai",
                            "limit": 10,
                            "offset": 0
                        }, indent=2)
                    },
                    "url": {
                        "raw": "{{baseUrl}}/api/services/search",
                        "host": ["{{baseUrl}}"],
                        "path": ["api", "services", "search"]
                    }
                }
            }
        ]
    }

def main():
    parser = argparse.ArgumentParser(description="Generate DuxNet API documentation")
    parser.add_argument("--format", choices=["openapi", "postman", "both"], default="both",
                      help="Documentation format to generate")
    parser.add_argument("--output-dir", default="./docs", help="Output directory for documentation")
    
    args = parser.parse_args()
    
    import os
    os.makedirs(args.output_dir, exist_ok=True)
    
    if args.format in ["openapi", "both"]:
        openapi_spec = generate_api_docs()
        with open(f"{args.output_dir}/openapi.json", "w") as f:
            json.dump(openapi_spec, f, indent=2)
        print(f"✅ OpenAPI specification generated: {args.output_dir}/openapi.json")
    
    if args.format in ["postman", "both"]:
        postman_collection = generate_postman_collection()
        with open(f"{args.output_dir}/postman_collection.json", "w") as f:
            json.dump(postman_collection, f, indent=2)
        print(f"✅ Postman collection generated: {args.output_dir}/postman_collection.json")
    
    print(f"\n📚 Documentation generated in: {args.output_dir}")
    print("🌐 You can now use these files with:")
    print("   - Swagger UI for OpenAPI documentation")
    print("   - Postman for API testing")
    print("   - Any OpenAPI-compatible tool")

if __name__ == "__main__":
    main()
