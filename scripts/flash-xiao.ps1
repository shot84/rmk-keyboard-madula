# XIAO-only diagnostic path; not a Cornix bootloader profile.
[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param(
    [Parameter(Mandatory)][string]$Firmware,
    [Parameter(Mandatory)][string]$Port,
    [Parameter(Mandatory)][string]$ExpectedPnpDeviceId,
    [ValidatePattern('^[A-Z]$')][string]$BootDrive = 'H',
    [switch]$MigrateToRmkSdc
)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path -LiteralPath $Firmware).ProviderPath
$bytes = [IO.File]::ReadAllBytes($source)
if ($bytes.Length -eq 0 -or $bytes.Length % 512 -ne 0) { throw 'Invalid UF2 size' }
$blocks = $bytes.Length / 512
$baseAddress = if ($MigrateToRmkSdc) { 0x1000 } else { 0x27000 }
if ($MigrateToRmkSdc) { Write-Warning 'RMK SDC migration overwrites legacy SoftDevice. ZMK rollback requires SoftDevice restoration.' }
for ($offset = 0; $offset -lt $bytes.Length; $offset += 512) {
    $word = { param($n) [BitConverter]::ToUInt32($bytes, $offset + $n) }
    $address = & $word 12
    $size = & $word 16
    if ((& $word 0) -ne 0x0A324655 -or (& $word 4) -ne 0x9E5D5157L -or
        (& $word 508) -ne 0x0AB16F30 -or (& $word 8) -ne 0x2000 -or
        (& $word 28) -ne 0xADA52840L -or $size -ne 256 -or
        (& $word 20) -ne ($offset / 512) -or (& $word 24) -ne $blocks -or
        $address -ne ($baseAddress + ($offset / 512) * 256) -or
        ($address + $size) -gt 0xD4000) { throw 'Unsafe or unsupported UF2 block' }
}
$stack = [BitConverter]::ToUInt32($bytes, 32)
$entry = [BitConverter]::ToUInt32($bytes, 36)
if ($stack -lt 0x20000000 -or $stack -gt 0x20040000 -or
    $entry -lt ($baseAddress + 1) -or $entry -ge ($baseAddress + $blocks * 256) -or ($entry % 2) -ne 1) {
    throw 'Invalid application vector table'
}
$root = "${BootDrive}:\"
if (Test-Path -LiteralPath $root) { throw 'Boot drive already exists; refusing ambiguous automatic reset.' }
if (-not $PSCmdlet.ShouldProcess("$Port -> $root", "Reset and flash $source")) { return }
& "$PSScriptRoot/enter-bootloader.ps1" -Port $Port -ExpectedPnpDeviceId $ExpectedPnpDeviceId -Confirm:$false
$deadline = (Get-Date).AddSeconds(20)
while (-not (Test-Path -LiteralPath "${root}INFO_UF2.TXT")) {
    if ((Get-Date) -gt $deadline) { throw 'Bootloader timeout. No firmware copied.' }
    Start-Sleep -Milliseconds 250
}
$info = Get-Content -LiteralPath "${root}INFO_UF2.TXT" -Raw
$softdeviceAllowed = ($info -match 'SoftDevice: S140 version 7.3.0') -or
    ($MigrateToRmkSdc -and $info -match 'SoftDevice: not found')
if ($info -notmatch '(?m)^Board-ID: Seeed_XIAO_nRF52840_Sense\s*$' -or
    -not $softdeviceAllowed) { throw 'Unexpected bootloader; no firmware copied.' }
$backup = Join-Path $PSScriptRoot "../target/backups/xiao-$(Get-Date -Format yyyyMMdd-HHmmss-fff)"
New-Item -ItemType Directory -Path $backup | Out-Null
Copy-Item -LiteralPath "${root}CURRENT.UF2" -Destination "$backup/CURRENT.UF2"
Copy-Item -LiteralPath "${root}INFO_UF2.TXT" -Destination "$backup/INFO_UF2.TXT"
Copy-Item -LiteralPath $source -Destination "${root}firmware.uf2"
Write-Output "Firmware copied; backup: $backup. Application startup still requires verification."
