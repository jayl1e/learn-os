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
    binmap = []
    for (idx, name) in enumerate(apps):
        code = os.system("make target/riscv64gc-unknown-none-elf/release/%s" % (name,))
        if code!=0:
            raise Exception("shell exit with %d" % code)
        binmap.append(name)
    bins = []
    for name in binmap:
        bins.append('[[bin]]\n')
        bins.append('name="%s"\n' % name)
        bins.append('file="target/riscv64gc-unknown-none-elf/release/%s"\n' % name)
        bins.append('\n')
    with open('binmap.toml', 'w') as f:
        f.writelines(bins)

if __name__ == "__main__":
    main()