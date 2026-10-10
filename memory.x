/* Compile-only candidate. Verify all three bootloaders before flashing.
 * Do not overwrite MBR/SoftDevice, ZMK storage or the bootloader. */
MEMORY {
  FLASH : ORIGIN = 0x00027000, LENGTH = 692K
  RAM : ORIGIN = 0x20000008, LENGTH = 255K
}
/* Application ends at 0xD4000. RMK storage begins at 0xE0000.
 * RMK storage overlaps existing ZMK settings: migration/backup REQUIRED. */
