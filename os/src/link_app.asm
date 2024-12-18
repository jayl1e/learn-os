
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 8

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
    

    .quad app_5_name
    .quad app_5_start
    .quad app_5_end
    

    .quad app_6_name
    .quad app_6_start
    .quad app_6_end
    

    .quad app_7_name
    .quad app_7_start
    .quad app_7_end
    

    .quad app_8_name
    .quad app_8_start
    .quad app_8_end
    

    .global app_1_name
    .global app_1_start
    .global app_1_end
app_1_name:
    .asciz "00hello"
    .align 3
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/00hello"
app_1_end:

    .global app_2_name
    .global app_2_start
    .global app_2_end
app_2_name:
    .asciz "01print_stack"
    .align 3
app_2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01print_stack"
app_2_end:

    .global app_3_name
    .global app_3_start
    .global app_3_end
app_3_name:
    .asciz "02sleep"
    .align 3
app_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/02sleep"
app_3_end:

    .global app_4_name
    .global app_4_start
    .global app_4_end
app_4_name:
    .asciz "03priv_inst"
    .align 3
app_4_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/03priv_inst"
app_4_end:

    .global app_5_name
    .global app_5_start
    .global app_5_end
app_5_name:
    .asciz "04csr"
    .align 3
app_5_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/04csr"
app_5_end:

    .global app_6_name
    .global app_6_start
    .global app_6_end
app_6_name:
    .asciz "file_test"
    .align 3
app_6_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/file_test"
app_6_end:

    .global app_7_name
    .global app_7_start
    .global app_7_end
app_7_name:
    .asciz "init"
    .align 3
app_7_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/init"
app_7_end:

    .global app_8_name
    .global app_8_start
    .global app_8_end
app_8_name:
    .asciz "shell"
    .align 3
app_8_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/shell"
app_8_end:
