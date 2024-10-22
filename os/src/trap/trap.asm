.altmacro

.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .global __alltraps
    .global __restore
    .align 2
__alltraps:
    # sp->trap_context, sscratch->user sp
    csrrw sp, sscratch, sp
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    csrr t0, sstatus
    csrr t1, sepc
    csrr t2, sscratch
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    sd t2, 2*8(sp)
    # recover kernel from ctx
    # satp
    ld t0, 34*8(sp)
    # trap_handler
    ld t1, 36*8(sp)
    # ksp
    ld sp, 35*8(sp)
    csrw satp, t0
    sfence.vma
    # mv a0, sp
    # call trap_handler
    jr t1

__restore:
    # mv sp, a0, switch jump here without setting proper a0
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    ld sp, 2*8(sp)
    sret