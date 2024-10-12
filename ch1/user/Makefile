all: target/riscv64gc-unknown-none-elf/release/00hello.bin target/riscv64gc-unknown-none-elf/release/03priv_inst.bin target/riscv64gc-unknown-none-elf/release/04csr.bin target/riscv64gc-unknown-none-elf/release/01print_stack.bin

target/riscv64gc-unknown-none-elf/release/%.bin: target/riscv64gc-unknown-none-elf/release/%
	rust-objcopy --strip-all $< -O binary $@

target/riscv64gc-unknown-none-elf/release/% : src/bin/%.rs
	cargo build --release --bin  $*
