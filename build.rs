fn main() {
    windows::build! {
        Windows::Win32::Foundation::PWSTR,
        Windows::Win32::Media::Audio::CoreAudio::*,
        Windows::Win32::Storage::StructuredStorage::{PROPVARIANT, STGM_READ},
        Windows::Win32::System::Com::*,
        Windows::Win32::System::Console::GetConsoleWindow,
        Windows::Win32::System::PropertiesSystem::{IPropertyStore, PROPERTYKEY},
        Windows::Win32::System::SystemServices::*,
        Windows::Win32::UI::WindowsAndMessaging::ShowWindow,
    };
}
