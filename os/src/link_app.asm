
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 2

    .quad app_1_name
    .quad app_1_start
    .quad app_1_end
    

    .quad app_2_name
    .quad app_2_start
    .quad app_2_end
    

    .global app_1_name
    .global app_1_start
    .global app_1_end
app_1_name:
    .asciz "01print_stack"
    .align 3
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01print_stack"
app_1_end:

    .global app_2_name
    .global app_2_start
    .global app_2_end
app_2_name:
    .asciz "init"
    .align 3
app_2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/init"
app_2_end:
