//! Cross-test flock for SPANDA_LIVE_RADAR env mutations across parallel test binaries.
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

pub struct RadarEnvLock {
    #[cfg(unix)]
    _file: std::fs::File,
}

impl RadarEnvLock {
    pub fn acquire() -> io::Result<Self> {
        #[cfg(unix)]
        {
            let path = radar_lock_path();
            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)?;
            let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
            if rc != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { _file: file })
        }
        #[cfg(not(unix))]
        {
            Ok(Self {})
        }
    }
}

fn radar_lock_path() -> PathBuf {
    std::env::temp_dir().join("spanda-radar-env.lock")
}
