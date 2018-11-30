/* Linker script for medium-density flash STM32F103xB MCUs */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}
