# Blue Yeti Volume Sync

`yeti.exe` is a small utility program that 
syncs the volume level of the Blue Yeti with
Windows' device volume settings.

This was mostly developed as fun exercise to reverse USB HID protocols
and play around with Rust's [`windows-rs`](https://github.com/microsoft/windows-rs).

## Usage

Download `yeti.exe` from the [releases page](https://github.com/mhils/yeti-volume-sync/releases).

To sync volume settings permanently, place yeti.exe somewhere on your system
and then create a shortcut to `C:\your\path\to\yeti.exe --hide` in
`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`.

## Compatibility

This code was tested with a Blue Yeti X only, 
but maybe also works with the regular Yeti.
