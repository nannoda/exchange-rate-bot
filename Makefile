# run: build
# ifeq ($(OS),Windows_NT)
# 	.\target\release\exchange-rate-bot.exe
# else
# 	./target/release/exchange-rate-bot
# endif

run:
	cargo run

build:
	cargo build --release

build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-linux-arm64:
	cargo build --release --target aarch64-unknown-linux-gnu
	

zip:
	git archive -o ./exchange_rate_bot.zip HEAD

rustup_init_unix:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh