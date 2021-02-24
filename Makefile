all: target/armv7-unknown-linux-gnueabihf/release/pyramid-player

target/armv7-unknown-linux-gnueabihf/release/pyramid-player:
	docker build -t crossbuild:local .
	cross build --release --target=armv7-unknown-linux-gnueabihf
