#!/bin/bash

# Build script for wgpu WebAssembly test

echo "Building wgpu WebAssembly module..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WebAssembly module
echo "Running wasm-pack build..."
wasm-pack build --target web --out-dir pkg

# Create a simple HTTP server script
cat > serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

    def guess_type(self, path):
        mimetype = super().guess_type(path)
        if path.endswith('.wasm'):
            return 'application/wasm'
        return mimetype

PORT = 8000
os.chdir(os.path.dirname(os.path.abspath(__file__)))

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Server running at http://localhost:{PORT}/")
    print("Press Ctrl+C to stop")
    httpd.serve_forever()
EOF

chmod +x serve.py

echo "Build complete!"
echo ""
echo "To test the application:"
echo "1. Run: python3 serve.py"
echo "2. Open http://localhost:8000/index.html in a WebGPU-enabled browser"
echo ""
echo "Note: You need a browser with WebGPU support (Chrome/Edge with WebGPU enabled)"