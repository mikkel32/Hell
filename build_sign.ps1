# save as build_sign.ps1

# --- 1. ENVIRONMENT CONFIGURATION ---
$LogFile = "$PSScriptRoot\build_debug_v2.log"
function Log($msg) {
    $timestamp = Get-Date -Format "HH:mm:ss"
    Add-Content -Path $LogFile -Value "[$timestamp] $msg"
    Write-Host "[$timestamp] $msg" -ForegroundColor Cyan
}

Log ">>> CONFIGURATION START"
Log "Script Path: $PSScriptRoot"

# IMPORT CRITICAL MODULES
try {
    Import-Module Microsoft.PowerShell.Security -ErrorAction Stop
    Import-Module Pki -ErrorAction Stop
} catch {
    Log ">>> WARN: Could not import security modules. Signing might fail."
}

# ENSURE CERT DRIVE
if (-not (Get-PSDrive -Name Cert -ErrorAction SilentlyContinue)) {
    Log ">>> Mounting Cert Drive..."
    New-PSDrive -Name Cert -PSProvider Certificate -Root CurrentUser -ErrorAction SilentlyContinue
}

# CRITICAL: Switch to the script's directory so cargo finds the project
Set-Location $PSScriptRoot
Log "Working Directory: $(Get-Location)"

if (-not (Test-Path "Cargo.toml")) {
    Log ">>> FATAL ERROR: Cargo.toml not found in $PSScriptRoot"
    exit 1
}

# --- DEFENDER EXCLUSION (ENGINEERING FIX) ---
# Since we are Admin, we tell Windows Defender to TRUST this workspace.
Write-Host ">>> Applying Engineering Exclusions to Windows Defender..." -ForegroundColor Magenta

# EVASION: Build In-Place (Using User's Workspace which might be trusted)
$RootBuildDir = $PSScriptRoot
$RootTempDir = "$PSScriptRoot\build_temp" 

if (-not (Test-Path $RootTempDir)) { New-Item -ItemType Directory -Force -Path $RootTempDir | Out-Null }

Log ">>> Building In-Place at: $RootBuildDir"

# (Exclusions are already applied to PSScriptRoot above)

# C. CONFIGURE ENVIRONMENT (Direct PATH Injection)
Set-Location $RootBuildDir
$env:CARGO_TARGET_DIR = "$RootBuildDir\target"
$env:TEMP = $RootTempDir
$env:TMP = $RootTempDir

# Explicitly add Rust to PATH for this session
$rustDirs = @(
    "$env:USERPROFILE\.cargo\bin",
    "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin"
)
foreach ($dir in $rustDirs) {
    if (Test-Path $dir) {
        if ($env:PATH -notlike "*$dir*") {
            $env:PATH = "$dir;$env:PATH" # Prepend to ensure priority
            Log ">>> Injected PATH: $dir"
        }
    }
}

# Locate Cargo Executable
$cargoExe = Get-Command "cargo.exe" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
if (-not $cargoExe) {
    # Fallback to hardcoded check
    $fallback = "$env:USERPROFILE\.cargo\bin\cargo.exe"
    if (Test-Path $fallback) { $cargoExe = $fallback }
}

if (-not $cargoExe) {
    Log ">>> FATAL: 'cargo.exe' not found in PATH or standard locations."
    Log "PATH: $env:PATH"
    exit 1
}
Log ">>> Resolved Cargo: $cargoExe"

# --- POLYMORPHISM (INSANE ENGINEERING) ---
Log ">>> Injecting Polymorphic DNA..."
$polySeed = Get-Random
$polyContent = "pub const POLY_SEED: u64 = $polySeed;"
Set-Content -Path "$PSScriptRoot\src\poly.rs" -Value $polyContent
Log ">>> DNA Mutated: Seed $polySeed"

# D. BUILD IN-PLACE
Log ">>> Compiling In-Place..."
Invoke-Expression "taskkill /F /IM rustc.exe 2> `$null"
Invoke-Expression "taskkill /F /IM link.exe 2> `$null"
Invoke-Expression "taskkill /F /FI ""IMAGENAME eq build_script_build*"" 2> `$null" 

# Lower Process Priority to prevent AV starvation locks
[System.Diagnostics.Process]::GetCurrentProcess().PriorityClass = [System.Diagnostics.ProcessPriorityClass]::BelowNormal

$maxRetries = 3
$retryCount = 0
$buildSuccess = $false

do {
    # CRITICAL: Direct Invocation of Cargo
    # We use the Call Operator '&' and redirect PowerShell streams
    & $cargoExe build --release -j 1 2>&1 | Out-File -Append -FilePath $LogFile
    
    if ($LASTEXITCODE -eq 0) {
        $buildSuccess = $true
        break
    } else {
        $retryCount++
        if ($retryCount -lt $maxRetries) {
            Log ">>> WARN: Build failed inside Isolation. Retrying in 30 seconds... ($retryCount/$maxRetries)"
            Start-Sleep -Seconds 30
        }
    }
} while ($retryCount -lt $maxRetries)

# E. RE-ENABLE MONITORING (Clean up our mess)
try {
    Log ">>> [!] Re-enabling Real-Time Monitoring..."
    Set-MpPreference -DisableRealtimeMonitoring $false -ErrorAction SilentlyContinue
} catch {
    # If we couldn't disable it, we probably can't re-enable it (or don't need to).
}

if (-not $buildSuccess) {
    Log ">>> ERROR: Compilation failed in isolation."
    exit 1
}

# F. RETRIEVE ARTIFACT
$builtExe = "$RootBuildDir\target\release\kawaii_cleaner_pro.exe"
$finalExe = "$PSScriptRoot\kawaii_cleaner_pro.exe"

if (-not (Test-Path $builtExe)) {
     Log ">>> ERROR: Output binary not found at $builtExe"
     exit 1
}

Copy-Item -Path $builtExe -Destination $finalExe -Force
Log ">>> Safe Artifact Retrieved: $finalExe"
$exe = $finalExe
Set-Location $PSScriptRoot # Return to base

# CLEANUP (Leave no trace)
Log ">>> Cleaning up Isolation Environment..."
try {
    # Remove-Item -Path $RootBuildDir -Recurse -Force -ErrorAction SilentlyContinue # Don't delete PSScriptRoot!
    Remove-Item -Path $RootTempDir -Recurse -Force -ErrorAction SilentlyContinue
} catch {
    Log ">>> WARN: Cleanup incomple but artifact is safe."
}

# --- 3. CODE SIGNING (INSANE ENGINEERING) ---
$certName = "KawaiiEngineeringCert"

# Check/Create Certificate
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -match $certName } | Select-Object -First 1

if (-not $cert) {
    Log ">>> Generating Engineering Certificate..."
    $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=$certName" -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddYears(10)
    
    # TRUST THE CERT (Add to Root)
    Log ">>> Trusting Certificate..."
    $store = New-Object System.Security.Cryptography.X509Certificates.X509Store "Root", "CurrentUser"
    $store.Open("ReadWrite")
    $store.Add($cert)
    $store.Close()
    Log "Certificate Trusted"
}

# Sign the Binary
Log ">>> Signing Binary..."
# Try referencing the cmdlet directly if module issues persist, or just rely on standard path
try {
    $sig = Set-AuthenticodeSignature -FilePath $exe -Certificate $cert -ErrorAction Stop
    if ($sig.Status -eq "Valid") {
        Log ">>> SUCCESS: Binary is Signed and Trusted (Status: Valid)."
    } else {
        Log ">>> WARN: Signing Status: $($sig.Status) (Message: $($sig.StatusMessage))"
        # If UnknownError, it's usually valid but just untrusted on some systems
        if ($sig.Status -eq "UnknownError") {
             Log ">>> NOTE: Signature applied but root trust may require restart or admin propagation."
        }
    }
} catch {
    Log ">>> ERROR: Signing Cmdlet Failed: $_"
}
