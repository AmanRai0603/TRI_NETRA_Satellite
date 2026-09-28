/* obc_qemu -- the virtual OBC on an emulated Cortex-M4F (QEMU mps2-an386): the flight
 * software (C or Rust build) behind adcs-link/1 on CMSDK UART0 (QEMU -serial stdio).
 * Bare metal: vector table, .data/.bss init, FPU on, polled UART, semihosting exit.
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#include <stdint.h>
#include <stddef.h>
#include "adcs_link.h"

#define UART0 0x40004000u
#define REG(o) (*(volatile uint32_t *)(UART0 + (o)))
#define U_DATA 0x00u
#define U_STATE 0x04u
#define U_CTRL 0x08u
#define U_BAUD 0x10u

extern uint32_t _sidata, _sdata, _edata, _sbss, _ebss, _estack;
int main(void);

void Reset_Handler(void)
{
    uint32_t *s = &_sidata, *d = &_sdata;
    while (d < &_edata) *d++ = *s++;
    for (d = &_sbss; d < &_ebss; ) *d++ = 0;
    *(volatile uint32_t *)0xE000ED88u |= (0xFu << 20);           /* CPACR: CP10/CP11 full access */
    __asm volatile ("dsb\n isb");
    main();
    for (;;) {}
}
static void Default_Handler(void) { for (;;) {} }
__attribute__((section(".isr_vector"), used)) static void (*const vectors[16])(void) = {
    (void (*)(void))(&_estack), Reset_Handler, Default_Handler, Default_Handler, Default_Handler, Default_Handler,
    Default_Handler, 0, 0, 0, 0, Default_Handler, Default_Handler, 0, Default_Handler, Default_Handler };

static int ugetc(void *c) { (void)c; while (!(REG(U_STATE) & 2u)) {} return (int)(REG(U_DATA) & 0xFFu); }
static void uwrite(void *c, const uint8_t *b, size_t n)
{
    size_t i; (void)c;
    for (i = 0; i < n; i++) { while (REG(U_STATE) & 1u) {} REG(U_DATA) = b[i]; }
}

static void semihost_exit(int code)
{
    /* SYS_EXIT_EXTENDED (0x20): ADP_Stopped_ApplicationExit with a status code */
    volatile uint32_t blk[2] = { 0x20026u, (uint32_t)code };
    register uint32_t r0 __asm("r0") = 0x20u;
    register volatile uint32_t *r1 __asm("r1") = blk;
    __asm volatile ("bkpt 0xAB" : : "r"(r0), "r"(r1) : "memory");
}

/* SysTick on the processor clock (25 MHz on mps2-an386), free-running over its 24 bits.
 * It counts down; the link wants an up-counter, so return the complement. */
#define SYST_CSR (*(volatile uint32_t *)0xE000E010u)
#define SYST_RVR (*(volatile uint32_t *)0xE000E014u)
#define SYST_CVR (*(volatile uint32_t *)0xE000E018u)
static uint32_t systick(void *c) { (void)c; return 0xFFFFFFu - (SYST_CVR & 0xFFFFFFu); }

int main(void)
{
    adcs_link_io_t io = { ugetc, uwrite, 0, systick, 25000000u, 0xFFFFFFu };
    SYST_RVR = 0xFFFFFFu; SYST_CVR = 0u; SYST_CSR = 5u;           /* ENABLE | CLKSOURCE = processor */
    REG(U_BAUD) = 16u;
    REG(U_CTRL) = 3u;                                             /* TX + RX enable */
    semihost_exit(adcs_link_serve(&io) == 0 ? 0 : 1);
    return 0;
}

/* newlib's libm may reach these through errno / abort paths */
void _exit(int c) { semihost_exit(c); for (;;) {} }
int *__errno(void) { static int e; return &e; }
