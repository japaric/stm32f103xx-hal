/* Linker script for low-density flash STM32F103x4 MCUs */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 16K
  RAM : ORIGIN = 0x20000000, LENGTH = 6K
}
