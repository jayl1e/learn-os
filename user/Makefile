rebuild: clean build

build:
	python3 build.py

clean:
	if [ -d target/riscv64gc-unknown-none-elf/release ]; then find target/riscv64gc-unknown-none-elf/release -maxdepth 1 -type f -delete; fi;

target/riscv64gc-unknown-none-elf/release/%.bin: target/riscv64gc-unknown-none-elf/release/%
	rust-objcopy --strip-all $< -O binary $@

target/riscv64gc-unknown-none-elf/release/% : src/bin/%.rs
	cargo build --release --bin  $*
