/* Linker script for XL-density flash STM32F103xG MCUs */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 1M
  RAM : ORIGIN = 0x20000000, LENGTH = 96K
}
