PROFILE ?= release
build: remove_inc
ifeq ($(PROFILE), debug)
	LOG=DEBUG cargo build
else
	LOG=INFO cargo build --$(PROFILE)
endif
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/$(PROFILE)/os -O binary target/riscv64gc-unknown-none-elf/$(PROFILE)/os.bin

remove_inc:
	rm -rf target/riscv64gc-unknown-none-elf/$(PROFILE)/incremental/

run: build
	qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/$(PROFILE)/os.bin,addr=0x80200000

debug: build
	qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/$(PROFILE)/os.bin,addr=0x80200000 -S -s