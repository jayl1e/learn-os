PROFILE ?= debug
build:
ifeq ($(PROFILE), debug)
	LOG=DEBUG cargo build
else
	LOG=INFO cargo build --$(PROFILE)
endif
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/$(PROFILE)/os -O binary target/riscv64gc-unknown-none-elf/$(PROFILE)/os.bin

run: build
	qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/$(PROFILE)/os.bin,addr=0x80200000

debug: build
	qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/$(PROFILE)/os.bin,addr=0x80200000 -S -s