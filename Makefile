BINARY=target/armv7-unknown-linux-gnueabihf/release/pyramid-player

install: $(BINARY)

clean:
	rm $(BINARY) 

$(BINARY): src/main.rs
	docker build -t crossbuild:local .
	cross build --release --target=armv7-unknown-linux-gnueabihf
	scp $(BINARY) pi@$(PI):/home/pi/pyramid-player
