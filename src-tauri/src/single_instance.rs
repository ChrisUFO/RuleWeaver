use std::io;

pub struct SingleInstanceGuard {
    #[cfg(target_os = "windows")]
    handle: *mut std::ffi::c_void,

    #[cfg(unix)]
    _file: std::fs::File,
}

// `io::Result` is already `#[must_use]`, so callers get compiler help if they
// ignore the acquisition outcome.
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
    use std::fs::{self, DirBuilder, OpenOptions};
    use std::io;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    use std::path::{Path, PathBuf};

    type CInt = std::ffi::c_int;
    type Uid = std::ffi::c_uint;

    const LOCK_EX: CInt = 2;
    const LOCK_NB: CInt = 4;
    const PRIVATE_DIR_MODE: u32 = 0o700;

    unsafe extern "C" {
        fn flock(fd: CInt, operation: CInt) -> CInt;
        fn geteuid() -> Uid;
    }

    pub(super) fn try_acquire(name: &str) -> io::Result<Option<SingleInstanceGuard>> {
        let lock_path = lock_dir()?.join(format!("ruleweaver-{}.lock", sanitize_name(name)));
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

    pub(super) fn show_existing_window(_title: &str) {
        log::debug!("Single-instance handoff on Unix is not implemented yet");
    }

    fn lock_dir() -> io::Result<PathBuf> {
        if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from) {
            if is_secure_directory(&runtime_dir)? {
                return Ok(runtime_dir);
            }

            log::warn!(
                "Ignoring insecure XDG_RUNTIME_DIR for single-instance lock: {}",
                runtime_dir.display()
            );
        }

        let home_dir = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;

        let state_dir = home_dir.join(".local").join("state").join("ruleweaver");
        ensure_private_directory(&state_dir)?;
        Ok(state_dir)
    }

    fn is_secure_directory(path: &Path) -> io::Result<bool> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        };

        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Ok(false);
        }

        if metadata.uid() != current_euid() {
            return Ok(false);
        }

        Ok((metadata.mode() & 0o777) == PRIVATE_DIR_MODE)
    }

    fn ensure_private_directory(path: &Path) -> io::Result<()> {
        if let Ok(metadata) = fs::symlink_metadata(path) {
            return ensure_private_directory_metadata(path, &metadata);
        }

        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder.mode(PRIVATE_DIR_MODE);
        builder.create(path)?;

        let metadata = fs::symlink_metadata(path)?;
        ensure_private_directory_metadata(path, &metadata)
    }

    fn ensure_private_directory_metadata(path: &Path, metadata: &fs::Metadata) -> io::Result<()> {
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "single-instance lock directory is a symlink: {}",
                    path.display()
                ),
            ));
        }

        if !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "single-instance lock directory is not a directory: {}",
                    path.display()
                ),
            ));
        }

        if metadata.uid() != current_euid() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "single-instance lock directory is not owned by the current user: {}",
                    path.display()
                ),
            ));
        }

        let mode = metadata.mode() & 0o777;
        if mode != PRIVATE_DIR_MODE {
            fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIR_MODE))?;
        }

        Ok(())
    }

    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect()
    }

    fn current_euid() -> u32 {
        unsafe { geteuid() }
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

    // Intentionally permissive on unsupported platforms until we add a native
    // single-instance primitive there.
    pub(super) fn try_acquire(_name: &str) -> io::Result<Option<SingleInstanceGuard>> {
        Ok(Some(SingleInstanceGuard {}))
    }

    pub(super) fn show_existing_window(_title: &str) {}
}
