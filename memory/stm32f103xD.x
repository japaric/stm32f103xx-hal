/* Linker script for high-density flash STM32F103xD MCUs */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 384K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}
