/* QEMU mps2-an386 (Cortex-M4F): code in SSRAM1 at 0, data in SSRAM2 at 0x20000000. Owner: Agastya. */
ENTRY(Reset_Handler)
MEMORY { FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 4M
         RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 2M }
_estack = ORIGIN(RAM) + LENGTH(RAM);
/* the stack grows down from _estack; this much is kept clear of .data and .bss (the deepest path
 * of the flight software is measured against it: tools/fsw_stack.py) */
_stack_size = 0x8000;
SECTIONS {
  .isr_vector : { KEEP(*(.isr_vector)) } > FLASH
  .text : { *(.text*) *(.rodata*) KEEP(*(.init)) KEEP(*(.fini)) . = ALIGN(4); } > FLASH
  .ARM.exidx : { *(.ARM.exidx*) } > FLASH
  _sidata = LOADADDR(.data);
  .data : { _sdata = .; *(.data*) . = ALIGN(4); _edata = .; } > RAM AT > FLASH
  .bss (NOLOAD) : { _sbss = .; *(.bss*) *(COMMON) . = ALIGN(4); _ebss = .; } > RAM
  end = .; _end = .;
  ASSERT(_ebss + _stack_size <= _estack, "RAM: .data + .bss leave less than _stack_size for the stack")
}
