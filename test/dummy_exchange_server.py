import json
from http.server import BaseHTTPRequestHandler, HTTPServer

# Load rate.json data at the start
with open('rate.json', 'r') as file:
    RATE_DATA = json.load(file)

class SimpleJSONServer(BaseHTTPRequestHandler):
    def do_GET(self):
        # Log request details
        print("Received GET request")
        print("Path:", self.path)
        print("Headers:", self.headers)

        # Respond with the contents of rate.json
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        response = json.dumps(RATE_DATA)
        self.wfile.write(response.encode('utf-8'))

    def log_message(self, format, *args):
        # Override to prevent default logging to stderr
        pass

def run(server_class=HTTPServer, handler_class=SimpleJSONServer, port=8080):
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    print(f"Starting server on port {port}...")
    httpd.serve_forever()

if __name__ == '__main__':
    run()
