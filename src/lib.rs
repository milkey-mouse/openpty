#![cfg(any(target_os = "linux", target_os = "android"))]

#[macro_use]
extern crate nix;
extern crate libc;

use nix::errno::Errno;
use std::error;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

mod ioctl {
    ioctl_write_int!(tiocsptlck, 'T', 0x31);
    ioctl_read!(tiocgptn, 'T', 0x30, libc::c_uint);
    ioctl_write_ptr_bad!(tiocswinsz, libc::TIOCSWINSZ, libc::winsize);
}

pub struct OpenedPty {
    master: File,
    slave: File,
    name: String,
}

pub fn openpty(
    termios: Option<&libc::termios>,
    winsize: Option<&libc::winsize>,
    name: Option<String>,
) -> Result<OpenedPty, Box<error::Error>> {
    let master = OpenOptions::new()
        .mode((libc::O_RDWR | libc::O_NOCTTY) as u32)
        .open("/dev/ptmx")?;

    let mut pts_number = 0;
    unsafe {
        ioctl::tiocsptlck(master.as_raw_fd(), 0)?;
        ioctl::tiocgptn(master.as_raw_fd(), &mut pts_number)?;
    }

    let name = name.unwrap_or_else(|| format!("/dev/pts/{}", pts_number));
    let slave = OpenOptions::new()
        .mode((libc::O_RDWR | libc::O_NOCTTY) as u32)
        .open(&name)?;

    if let Some(tio) = termios {
        Errno::result(unsafe { libc::tcsetattr(slave.as_raw_fd(), libc::TCSANOW, tio) })?;
    }

    if let Some(ws) = winsize {
        unsafe {
            ioctl::tiocswinsz(slave.as_raw_fd(), ws)?;
        }
    }

    Ok(OpenedPty {
        master,
        slave,
        name,
    })
}
