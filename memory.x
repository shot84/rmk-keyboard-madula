/* RMK SDC layout. Overwrites legacy SoftDevice; preserve MBR and bootloader.
 * Cornix bootloader compatibility remains unverified. */
MEMORY {
  FLASH : ORIGIN = 0x00001000, LENGTH = 844K
  RAM : ORIGIN = 0x20000008, LENGTH = 255K
}
/* Application ends at 0xD4000. RMK storage begins at 0xE0000.
 * RMK storage overlaps existing ZMK settings: migration/backup REQUIRED. */
