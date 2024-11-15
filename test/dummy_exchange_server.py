import json
import random
import time
from datetime import datetime
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs


class SimpleJSONServer(BaseHTTPRequestHandler):
    API_KEY = "YOUR_API_KEY"  # Replace with your desired API key for validation

    def do_GET(self):
        # Parse URL and query parameters
        url_parts = urlparse(self.path)
        query_params = parse_qs(url_parts.query)
        path = url_parts.path.strip("/")
        
        # Validate API key
        access_key = query_params.get("access_key", [None])[0]
        if access_key != self.API_KEY:
            self.send_error(401, "Unauthorized: Invalid API key.")
            return

        # Determine if the request is for the latest or historical rates
        if path == "latest":
            self.handle_latest(query_params)
        else:
            try:
                requested_date = datetime.strptime(path, "%Y-%m-%d").strftime("%Y-%m-%d")
                self.handle_historical(query_params, requested_date)
            except ValueError:
                self.send_error(400, "Invalid endpoint. Use /latest or /YYYY-MM-DD.")
                return

    def handle_latest(self, query_params):
        """Handle the /latest endpoint"""
        base_currency = query_params.get("base", ["EUR"])[0]
        symbols = query_params.get("symbols", None)
        self.generate_response(base_currency, symbols)

    def handle_historical(self, query_params, date):
        """Handle the /YYYY-MM-DD endpoint"""
        base_currency = query_params.get("base", ["EUR"])[0]
        symbols = query_params.get("symbols", None)
        self.generate_response(base_currency, symbols, date)

    def generate_response(self, base_currency, symbols, date=None):
        """Generate the response for both latest and historical rates"""
        # Generate random exchange rates
        rates = {
            "USD": round(random.uniform(1.1, 1.3), 6),
            "CAD": round(random.uniform(1.4, 1.6), 6),
            "EUR": round(random.uniform(1.0, 1.0), 6),  # Fixed for EUR as the base
            "GBP": round(random.uniform(0.85, 0.9), 6),
            "JPY": round(random.uniform(125, 135), 6),
            "AUD": round(random.uniform(1.4, 1.6), 6),
            "CHF": round(random.uniform(1.1, 1.3), 6),
            "CNY": round(random.uniform(7.5, 8.0), 6),
        }

        # Filter rates if symbols are provided
        if symbols:
            symbols_list = symbols[0].split(",")
            rates = {symbol: rate for symbol, rate in rates.items() if symbol in symbols_list}

        # Use current date if no date is provided
        response_date = date or datetime.now().strftime("%Y-%m-%d")

        # Prepare the response
        response_data = {
            "success": True,
            "timestamp": int(time.time()),
            "base": base_currency,
            "date": response_date,
            "rates": rates,
        }

        # Send JSON response
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        response = json.dumps(response_data)
        print("Response:", response)  # Log the response
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
