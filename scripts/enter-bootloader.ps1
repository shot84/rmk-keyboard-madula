# Reset only. Does not copy firmware or guess which COM port is the keyboard.
[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param(
    [Parameter(Mandatory)][ValidatePattern('^COM[0-9]+$')][string]$Port,
    [Parameter(Mandatory)][string]$ExpectedPnpDeviceId
)
$ErrorActionPreference = 'Stop'
$device = @(Get-CimInstance Win32_SerialPort | Where-Object DeviceID -EQ $Port)
if ($device.Count -ne 1 -or $device[0].PNPDeviceID -ne $ExpectedPnpDeviceId) {
    throw 'COM identity mismatch or device missing. No reset performed.'
}
if (-not $PSCmdlet.ShouldProcess("$Port ($ExpectedPnpDeviceId)", '1200-baud bootloader reset')) {
    return
}
$serial = [System.IO.Ports.SerialPort]::new($Port, 1200)
try {
    $serial.Handshake = [System.IO.Ports.Handshake]::None
    $serial.DtrEnable = $false
    $serial.Open()
    $serial.DtrEnable = $true
    Start-Sleep -Milliseconds 500
    try { $serial.DtrEnable = $false }
    catch [System.IO.IOException] {
        Write-Warning 'Device disconnected during DTR drop; verify bootloader enumeration.'
    }
    Start-Sleep -Milliseconds 200
} finally {
    try { $serial.Dispose() } catch [System.IO.IOException] {
        Write-Warning 'Serial device disappeared during reset cleanup.'
    }
}
Write-Output 'Reset requested, not verified. Inspect INFO_UF2.TXT on the bootloader volume. No firmware was written.'
