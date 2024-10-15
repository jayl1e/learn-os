import os

BASE_ADDRESS=0x80400000
STEP = 0x20000
LINKER='src/linker.ld'
LINKER_SRC='src/linker_src.ld'
RUST_BIN_FILE='.rs'

def main():
    apps = map(
        lambda x: x.removesuffix(RUST_BIN_FILE),
            filter(
                lambda x: x.endswith(RUST_BIN_FILE), 
                os.listdir("src/bin")
            )
        )
    try:
        os.remove('binmap.txt')
    except FileNotFoundError:
        pass
    apps = sorted(apps)
    lines = ["# generated from linker_src.ld\n"]
    with open(LINKER_SRC) as f:
        lines.extend(f.readlines())
    original_address = hex(BASE_ADDRESS)
    address_line = 0
    original_line = ''
    for idx, line in enumerate(lines):
        if original_address in line:
            address_line = idx
            original_line = line
    if not original_line:
        raise Exception('address not found')
    binmap = []
    for (idx, name) in enumerate(apps):
        naddress = BASE_ADDRESS + STEP * idx
        new_address = hex(naddress)
        lines[address_line] = original_line.replace(original_address, new_address)
        with open(LINKER, mode='w') as f:
            f.writelines(lines)
        code = os.system("make target/riscv64gc-unknown-none-elf/release/%s.bin" % (name,))
        if code!=0:
            raise Exception("shell exit with %d" % code)
        binmap.append((name, new_address, hex(naddress+STEP)))
    bins = []
    for (name,sa, ea) in binmap:
        bins.append('[[bin]]\n')
        bins.append('name="%s"\n' % name)
        bins.append('start="%s"\n' % sa)
        bins.append('end="%s"\n' % ea)
        bins.append('file="target/riscv64gc-unknown-none-elf/release/%s.bin"\n' % name)
        bins.append('\n')
    with open('binmap.toml', 'w') as f:
        f.writelines(bins)

if __name__ == "__main__":
    main()