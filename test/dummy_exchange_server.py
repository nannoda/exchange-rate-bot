import json
import random
import time
from datetime import datetime
from http.server import BaseHTTPRequestHandler, HTTPServer


class SimpleJSONServer(BaseHTTPRequestHandler):
    def do_GET(self):
        # Log request details
        print("Received GET request")
        print("Path:", self.path)
        print("Headers:", self.headers)

        # Generate dynamic exchange rate data
        response_data = {
            "success": True,
            "timestamp": int(time.time()),  # Current UNIX timestamp
            "base": "EUR",
            "date": datetime.now().strftime(
                "%Y-%m-%d"
            ),  # Current date in YYYY-MM-DD format
            "rates": {
                "AUD": round(random.uniform(1.4, 1.6), 6),
                "CAD": round(random.uniform(1.4, 1.6), 6),
                "CHF": round(random.uniform(1.1, 1.3), 6),
                "CNY": round(random.uniform(7.5, 8.0), 6),
                "GBP": round(random.uniform(0.85, 0.9), 6),
                "JPY": round(random.uniform(125, 135), 6),
                "USD": round(random.uniform(1.1, 1.3), 6),
            },
        }

        # Send JSON response
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        response = json.dumps(response_data)

        print("Response: ", response)
        self.wfile.write(response.encode("utf-8"))

    def log_message(self, format, *args):
        # Override to prevent default logging to stderr
        pass


def run(server_class=HTTPServer, handler_class=SimpleJSONServer, port=8080):
    server_address = ("", port)
    httpd = server_class(server_address, handler_class)
    print(f"Starting server on port {port}...")
    httpd.serve_forever()


if __name__ == "__main__":
    run()
