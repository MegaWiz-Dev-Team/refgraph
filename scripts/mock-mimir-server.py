#!/usr/bin/env python3
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import sys

class MockMimirHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if '/health' in self.path:
            response = {"status": "healthy", "uptime": 0}
        elif '/search' in self.path:
            response = {
                "results": [
                    {"id": "1", "text": "Critical Illness insurance", "score": 0.95},
                    {"id": "2", "text": "Health coverage", "score": 0.92},
                    {"id": "3", "text": "Premium rates", "score": 0.88}
                ],
                "score": 0.95,
                "count": 3
            }
        else:
            response = {"status": "ok"}
        
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())
    
    def do_POST(self):
        if '/ingest' in self.path:
            response = {
                "success": True,
                "entities_ingested": 10,
                "relationships_ingested": 5
            }
            self.send_response(201)
        else:
            response = {"status": "ok"}
            self.send_response(200)
        
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())
    
    def log_message(self, format, *args):
        pass  # Silent

if __name__ == '__main__':
    server = HTTPServer(('127.0.0.1', 9999), MockMimirHandler)
    print("🟢 Mock Mimir on http://127.0.0.1:9999")
    print("   Ready for integration testing\n")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Mock server stopped")
        sys.exit(0)
