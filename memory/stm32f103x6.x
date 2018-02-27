/* Linker script for low-density flash STM32F103x6 MCUs */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 32K
  RAM : ORIGIN = 0x20000000, LENGTH = 10K
}
