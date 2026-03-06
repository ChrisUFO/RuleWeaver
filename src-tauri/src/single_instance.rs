use std::io;

pub struct SingleInstanceGuard {
    #[cfg(target_os = "windows")]
    handle: *mut std::ffi::c_void,

    #[cfg(unix)]
    _file: std::fs::File,
}

pub fn try_acquire(name: &str) -> io::Result<Option<SingleInstanceGuard>> {
    imp::try_acquire(name)
}

pub fn show_existing_window(title: &str) {
    imp::show_existing_window(title);
}

#[cfg(target_os = "windows")]
mod imp {
    use super::SingleInstanceGuard;
    use std::io;
    use std::ptr;

    type Bool = i32;
    type Handle = *mut std::ffi::c_void;
    type Hwnd = *mut std::ffi::c_void;

    const ERROR_ALREADY_EXISTS: u32 = 183;
    const SW_SHOW: i32 = 5;
    const SW_RESTORE: i32 = 9;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CloseHandle(object: Handle) -> Bool;
        fn CreateMutexW(
            attributes: *mut std::ffi::c_void,
            initial_owner: Bool,
            name: *const u16,
        ) -> Handle;
        fn GetLastError() -> u32;
        fn ReleaseMutex(mutex: Handle) -> Bool;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn FindWindowW(class_name: *const u16, window_name: *const u16) -> Hwnd;
        fn SetForegroundWindow(window: Hwnd) -> Bool;
        fn ShowWindow(window: Hwnd, command: i32) -> Bool;
    }

    pub(super) fn try_acquire(name: &str) -> io::Result<Option<SingleInstanceGuard>> {
        let wide_name = wide_null(name);
        let handle = unsafe { CreateMutexW(ptr::null_mut(), 1, wide_name.as_ptr()) };

        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }

        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Ok(None);
        }

        Ok(Some(SingleInstanceGuard { handle }))
    }

    pub(super) fn show_existing_window(title: &str) {
        let wide_title = wide_null(title);
        let window = unsafe { FindWindowW(ptr::null(), wide_title.as_ptr()) };

        if window.is_null() {
            return;
        }

        unsafe {
            let _ = ShowWindow(window, SW_SHOW);
            let _ = ShowWindow(window, SW_RESTORE);
            let _ = SetForegroundWindow(window);
        }
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    impl Drop for SingleInstanceGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = ReleaseMutex(self.handle);
                let _ = CloseHandle(self.handle);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::try_acquire;

        #[test]
        fn prevents_duplicate_acquisition_for_same_name() {
            let name = format!("Local\\ruleweaver-test-{}", uuid::Uuid::new_v4());

            let first = try_acquire(&name).expect("first acquisition should succeed");
            assert!(first.is_some(), "first acquisition should own the mutex");

            let second = try_acquire(&name).expect("second acquisition should succeed");
            assert!(
                second.is_none(),
                "second acquisition should detect an existing instance"
            );

            drop(first);

            let third = try_acquire(&name).expect("third acquisition should succeed");
            assert!(
                third.is_some(),
                "mutex should be available again after guard drop"
            );
        }
    }
}

#[cfg(unix)]
mod imp {
    use super::SingleInstanceGuard;
    use std::fs::OpenOptions;
    use std::io;
    use std::os::fd::AsRawFd;

    type CInt = std::ffi::c_int;

    const LOCK_EX: CInt = 2;
    const LOCK_NB: CInt = 4;

    unsafe extern "C" {
        fn flock(fd: CInt, operation: CInt) -> CInt;
    }

    pub(super) fn try_acquire(name: &str) -> io::Result<Option<SingleInstanceGuard>> {
        let lock_path =
            std::env::temp_dir().join(format!("ruleweaver-{}.lock", sanitize_name(name)));
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path)?;

        let result = unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) };
        if result == 0 {
            return Ok(Some(SingleInstanceGuard { _file: file }));
        }

        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::WouldBlock {
            return Ok(None);
        }

        Err(error)
    }

    pub(super) fn show_existing_window(_title: &str) {}

    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::try_acquire;

        #[test]
        fn prevents_duplicate_acquisition_for_same_name() {
            let name = format!("ruleweaver-test-{}", uuid::Uuid::new_v4());

            let first = try_acquire(&name).expect("first acquisition should succeed");
            assert!(
                first.is_some(),
                "first acquisition should own the file lock"
            );

            let second = try_acquire(&name).expect("second acquisition should succeed");
            assert!(
                second.is_none(),
                "second acquisition should detect an existing instance"
            );

            drop(first);

            let third = try_acquire(&name).expect("third acquisition should succeed");
            assert!(
                third.is_some(),
                "lock should be available again after guard drop"
            );
        }
    }
}

#[cfg(not(any(target_os = "windows", unix)))]
mod imp {
    use super::SingleInstanceGuard;
    use std::io;

    pub(super) fn try_acquire(_name: &str) -> io::Result<Option<SingleInstanceGuard>> {
        Ok(Some(SingleInstanceGuard {}))
    }

    pub(super) fn show_existing_window(_title: &str) {}
}
