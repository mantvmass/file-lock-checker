# File Lock Checker (FLC)

A powerful Windows file lock detection and unlock utility.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)

## Features

- **🔍 Lock Detection**: Identify which processes are locking files or folders
- **🔓 Force Unlock**: Terminate processes that are locking files (with confirmation)
- **📊 Real-time Monitoring**: Monitor file locks with configurable intervals
- **🖱️ Context Menu Integration**: Right-click integration for easy access
- **⚡ Fast & Lightweight**: Built with Rust for optimal performance
- **🛡️ Safe Operations**: Confirmation prompts before terminating processes

## Installation

### Prerequisites

- Windows 10/11  
- Administrator privileges (required for context menu installation)

---

### Download (Pre-built Release)

If you don't want to build the project yourself, you can download the latest pre-built executable:

1. Go to the [Releases page](https://github.com/mantvmass/file-lock-checker/releases)
2. Download the latest `file-lock-checker.exe`
3. Place it anywhere you prefer, or add it to your `PATH` for easy access via terminal

---

### Build from Source

If you prefer to build the project manually:

```bash
# Clone the repository
git clone https://github.com/mantvmass/file-lock-checker.git
cd file-lock-checker

# Build the project in release mode
cargo build --release

# Copy the built executable to the project root (optional)
copy target\release\file-lock-checker.exe .
```

---

### Context Menu Installation (Optional)

1. **Run as Administrator**:
   ```cmd
   # Right-click on install_context_menu.bat
   # Select "Run as administrator"
   ```

2. **Or use command line**:
   ```cmd
   # Open Command Prompt as Administrator
   install_context_menu.bat
   ```

3. **Or use file-lock-checker.exe option**:
   ```cmd
   # Open Command Prompt as Administrator
   file-lock-checker.exe install
   ```

This adds three options to your right-click context menu:
- **FLC Check** - Check file locks
- **FLC Unlock** - Unlock files
- **FLC Monitor** - Monitor files

## Usage

### Command Line Interface

```bash
# Check which processes are locking a file
file-lock-checker.exe check "C:\path\to\your\file.txt"

# Unlock a file by terminating locking processes
file-lock-checker.exe unlock "C:\path\to\your\file.txt"

# Monitor file locks in real-time (default: 2 second intervals)
file-lock-checker.exe monitor "C:\path\to\your\file.txt"

# Monitor with custom interval (5 seconds)
file-lock-checker.exe monitor "C:\path\to\your\file.txt" 5
```

---

### Context Menu Usage

1. **Right-click** on any file or folder
2. Select one of the FLC options:
   - **FLC Check**: Opens terminal showing locking processes
   - **FLC Unlock**: Prompts for confirmation before killing processes
   - **FLC Monitor**: Starts real-time monitoring in terminal

## Examples

### Basic File Lock Check
```bash
file-lock-checker.exe check "C:\temp\document.docx"
```

**Output:**
```
File 'C:\temp\document.docx' is locked by 2 process(es):
--------------------------------------------------------------------------------
Process #1
  PID: 1234
  Name: WINWORD.EXE
  Path: C:\Program Files\Microsoft Office\OFFICE16\WINWORD.EXE

Process #2
  PID: 5678
  Name: explorer.exe
  Path: C:\Windows\explorer.exe
```

---

### Unlocking a File
```bash
file-lock-checker.exe unlock "C:\temp\document.docx"
```

**Output:**
```
File 'C:\temp\document.docx' is locked by 1 process(es):
--------------------------------------------------------------------------------
Process #1
  PID: 1234
  Name: WINWORD.EXE
  Path: C:\Program Files\Microsoft Office\OFFICE16\WINWORD.EXE

Do you want to kill these processes? (y/N): y
Successfully killed process: WINWORD.EXE (PID: 1234)

Rechecking file locks...
File 'C:\temp\document.docx' is now unlocked
```

---

### Real-time Monitoring
```bash
file-lock-checker.exe monitor "C:\temp\document.docx" 3
```

**Output:**
```
Monitoring file locks for: C:\temp\document.docx
Update interval: 3 seconds
Press Ctrl+C to stop monitoring

[2024-08-07 14:30:15] Lock status changed:
  File is LOCKED by 1 process(es):
    PID: 1234 | Name: WINWORD.EXE | Path: C:\Program Files\Microsoft Office\OFFICE16\WINWORD.EXE

[2024-08-07 14:30:45] Lock status changed:
  File is now UNLOCKED
```

## Technical Details

### How It Works

File Lock Checker uses the **Windows Restart Manager API** to identify processes that have handles to specific files. This is the same underlying technology used by Windows Update and other system tools to determine which applications need to be restarted.

**Key APIs used:**
- `RmStartSession` - Initialize Restart Manager session
- `RmRegisterResources` - Register files to check
- `RmGetList` - Get list of processes using the resources
- `TerminateProcess` - Force-kill locking processes

### Limitations

1. **Kernel-level Locks**: Some system files locked by kernel drivers may not be detected
2. **System Services**: Critical Windows services may not appear in results
3. **Administrative Rights**: Some operations require elevated privileges

### Supported File Types

- ✅ Regular files (.txt, .docx, .xlsx, .pdf, etc.)
- ✅ Executable files (.exe, .dll, .msi, etc.)
- ✅ Media files (.mp4, .avi, .mp3, etc.)
- ✅ Database files (.db, .sqlite, .mdb, etc.)
- ⚠️ System files (.sys, kernel drivers) - limited detection
- ⚠️ Registry files - limited detection

## Configuration

### Cargo.toml Dependencies

```toml
[dependencies]
winapi = { version = "0.3.9", features = [
    "handleapi",           # CloseHandle, INVALID_HANDLE_VALUE
    "processthreadsapi",   # OpenProcess, TerminateProcess, GetCurrentProcess
    "psapi",               # EnumProcesses, GetModuleBaseNameW, GetProcessImageFileNameW
    "restartmanager",      # RmStartSession, RmEndSession, RmRegisterResources, RmGetList
    "winnt",               # PROCESS_* constants, HANDLE
    "minwindef",           # DWORD, FALSE, TRUE
    "ntdef",               # HANDLE definition
    "winreg",              # Registry functions
    "winnt",               # Registry constants
    "winerror",            # ERROR_MORE_DATA และ error constants อื่นๆ
] }
chrono = "0.4.41"
```

## Troubleshooting

### Common Issues

**1. "Access Denied" errors**
- Run as Administrator
- Some system processes cannot be terminated

**2. No processes found but file still locked**
- File may be locked by kernel drivers
- Try using `handle.exe` from Sysinternals for deeper analysis

**3. Context menu not appearing**
- Ensure installation script ran as Administrator
- Try logging out and back in
- Check Windows Registry entries

**4. False negatives for .sys files**
- System driver files use kernel-level locks
- Restart Manager API has limitations with kernel resources

### Manual Registry Cleanup

If context menu installation fails, manually remove these registry keys:

```cmd
reg delete "HKCR\*\shell\FLC_Check" /f
reg delete "HKCR\*\shell\FLC_Unlock" /f  
reg delete "HKCR\*\shell\FLC_Monitor" /f
reg delete "HKCR\Directory\shell\FLC_Check" /f
reg delete "HKCR\Directory\shell\FLC_Monitor" /f
```

## Uninstallation

### Remove Context Menu
```cmd
# Run as Administrator
uninstall_context_menu.bat

# or use file-lock-checker.exe option
file-lock-checker.exe uninstall
```

### Remove Program
Simply delete the executable and associated files:
```cmd
del file-lock-checker.exe
del install_context_menu.bat
del uninstall_context_menu.bat
```

## Development

### Building from Source

```bash
# Prerequisites
rustup install stable
rustup default stable

# Clone and build
git clone https://github.com/mantvmass/file-lock-checker.git
cd file-lock-checker
cargo build --release
```

### Running Tests

```bash
# Test with a file you know is locked
python lock_test.py "C:\path\to\your\test.txt"
# In another terminal:
cargo run -- check "C:\path\to\your\test.txt"
```

### Adding Features

The codebase is structured for easy extension:
- `src/main.rs` - Core logic and CLI interface
- Context menu integration via Windows Registry
- Real-time monitoring with configurable intervals

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- Uses Windows Restart Manager API
- Built with [Rust](https://www.rust-lang.org/) and [winapi-rs](https://github.com/retep998/winapi-rs)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.