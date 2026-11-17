use crate::domain::validation::DriveInfo;
use crate::platform::win32::{
    CloseHandle, CreateFileW, DRIVE_FIXED, DeviceIoControl, FILE_SHARE_READ, FILE_SHARE_WRITE,
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW,
    GetWindowsDirectoryW, INVALID_HANDLE_VALUE, IOCTL_STORAGE_GET_DEVICE_NUMBER, OPEN_EXISTING,
    STORAGE_DEVICE_NUMBER, from_wide_slice, to_wide_null,
};
use std::ffi::c_void;

pub fn get_system_drive_letter() -> Option<char> {
    let mut buf = [0u16; 260];
    let len = unsafe { GetWindowsDirectoryW(buf.as_mut_ptr(), buf.len() as u32) };
    if len == 0 || (len as usize) >= buf.len() {
        return None;
    }
    let path = String::from_utf16_lossy(&buf[..len as usize]);
    path.chars().next().map(|c| c.to_ascii_uppercase())
}

fn get_physical_disk_number(letter: char) -> Option<u32> {
    let device_path = to_wide_null(&format!("\\\\.\\{}:", letter.to_ascii_uppercase()));
    unsafe {
        let handle = CreateFileW(
            device_path.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return None;
        }
        let mut sdn = STORAGE_DEVICE_NUMBER::default();
        let mut bytes_returned: u32 = 0;
        let ok = DeviceIoControl(
            handle,
            IOCTL_STORAGE_GET_DEVICE_NUMBER,
            std::ptr::null(),
            0,
            (&raw mut sdn).cast::<c_void>(),
            std::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
            &raw mut bytes_returned,
            std::ptr::null_mut(),
        );
        CloseHandle(handle);
        if ok != 0 {
            Some(sdn.DeviceNumber)
        } else {
            None
        }
    }
}

/// 固定ドライブ（DRIVE_FIXED）のみを列挙して詳細情報を返す。
pub fn enumerate_fixed_drives() -> Vec<DriveInfo> {
    let sys_drive = get_system_drive_letter();
    let mut buf = [0u16; 512];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 || (len as usize) > buf.len() {
        return Vec::new();
    }

    let slice = &buf[..len as usize];
    let mut drives = Vec::new();

    for root in slice.split(|&c| c == 0) {
        let Some(&first) = root.first() else {
            continue;
        };
        if first > 0x7F {
            continue;
        }
        let letter = (first as u8 as char).to_ascii_uppercase();
        if !letter.is_ascii_uppercase() {
            continue;
        }
        let root_w = [letter as u16, b':' as u16, b'\\' as u16, 0];
        let dtype = unsafe { GetDriveTypeW(root_w.as_ptr()) };
        if dtype != DRIVE_FIXED {
            continue;
        }

        let mut vol_name = [0u16; 261];
        let mut fs_name = [0u16; 64];
        let ok_vol = unsafe {
            GetVolumeInformationW(
                root_w.as_ptr(),
                vol_name.as_mut_ptr(),
                vol_name.len() as u32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                fs_name.as_mut_ptr(),
                fs_name.len() as u32,
            )
        };
        let label = if ok_vol != 0 {
            from_wide_slice(&vol_name)
        } else {
            String::new()
        };
        let fs = if ok_vol != 0 {
            from_wide_slice(&fs_name)
        } else {
            String::new()
        };

        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;
        unsafe {
            GetDiskFreeSpaceExW(
                root_w.as_ptr(),
                &raw mut free_bytes,
                &raw mut total_bytes,
                std::ptr::null_mut(),
            );
        }

        let disk_number = get_physical_disk_number(letter);
        let is_system = sys_drive == Some(letter);

        drives.push(DriveInfo {
            letter,
            label,
            fs_name: fs,
            free_bytes,
            total_bytes,
            is_system,
            disk_number,
        });
    }

    drives
}
