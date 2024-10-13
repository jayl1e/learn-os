    .align 3
    .section .data
    .global _num_app

_num_app:
    .quad 4
    .quad app_1_name
    .quad app_1_start
    .quad app_1_end

    .quad app_2_name
    .quad app_2_start
    .quad app_2_end

    .quad app_3_name
    .quad app_3_start
    .quad app_3_end

    .quad app_4_name
    .quad app_4_start
    .quad app_4_end



    .section .data
    .global app_1_name
    .global app_1_start
    .global app_1_end
app_1_name:
    .asciz "00hello"
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/00hello.bin"
app_1_end:


    .section .data
    .global app_2_name
    .global app_2_start
    .global app_2_end
app_2_name:
    .asciz "04csr"
app_2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/04csr.bin"
app_2_end:

    .section .data
    .global app_3_name
    .global app_3_start
    .global app_3_end
app_3_name:
    .asciz "01print_stack"
app_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01print_stack.bin"
app_3_end:

    .section .data
    .global app_4_name
    .global app_4_start
    .global app_4_end
app_4_name:
    .asciz "03priv_inst"
app_4_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/03priv_inst.bin"
app_4_end: