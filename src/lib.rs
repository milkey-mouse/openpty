#![cfg(any(target_os = "linux", target_os = "android"))]

#[macro_use]
extern crate nix;
extern crate libc;

use nix::errno::Errno;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::AsRawFd};

mod ioctl {
    ioctl_write_ptr!(tiocsptlck, 'T', 0x31, libc::c_int);
    ioctl_read!(tiocgptn, 'T', 0x30, libc::c_uint);
    ioctl_write_ptr_bad!(tiocswinsz, libc::TIOCSWINSZ, libc::winsize);
}

/// Creates a new pseudo terminal in /dev/pts/ and returns the name and the master / slave file
/// descriptors.
///
/// # Examples
/// ```
/// let (master, slave, name) = openpty::openpty(None, None, None)
///     .expect("Creating pty failed");
/// ```
pub fn openpty(
    termios: Option<&libc::termios>,
    winsize: Option<&libc::winsize>,
    name: Option<String>,
) -> Result<(File, File, String), Box<dyn Error>> {
    let master = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open("/dev/ptmx")?;

    let mut pts_number = 0;
    unsafe {
        ioctl::tiocsptlck(master.as_raw_fd(), &(pts_number as libc::c_int))?;
        ioctl::tiocgptn(master.as_raw_fd(), &mut pts_number)?;
    }

    let name = name.unwrap_or_else(|| format!("/dev/pts/{}", pts_number));
    let slave = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open(&name)?;

    if let Some(tio) = termios {
        Errno::result(unsafe { libc::tcsetattr(slave.as_raw_fd(), libc::TCSANOW, tio) })?;
    }

    if let Some(ws) = winsize {
        unsafe {
            ioctl::tiocswinsz(slave.as_raw_fd(), ws)?;
        }
    }

    Ok((master, slave, name))
}
