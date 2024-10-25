rebuild: clean build

build:
	python3 build.py

clean:
	find target/riscv64gc-unknown-none-elf/release -depth 1 -type file -delete

target/riscv64gc-unknown-none-elf/release/%.bin: target/riscv64gc-unknown-none-elf/release/%
	rust-objcopy --strip-all $< -O binary $@

target/riscv64gc-unknown-none-elf/release/% : src/bin/%.rs
	cargo build --release --bin  $*
