use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;

pub(crate) const DISP_LCD_SET_BRIGHTNESS: libc::c_ulong = 0x102;
pub(crate) const DISP_LCD_GET_BRIGHTNESS: libc::c_ulong = 0x103;
pub(crate) const DISP_LCD_BACKLIGHT_ENABLE: libc::c_ulong = 0x104;
pub(crate) const DISP_LCD_BACKLIGHT_DISABLE: libc::c_ulong = 0x105;

const DISP_DEVICE: &str = "/dev/disp";
const LCD_SCREEN: libc::c_ulong = 0;
const FALLBACK_RESTORE_BRIGHTNESS: u8 = 80;

pub struct ScreenBacklight {
    saved_brightness: Option<u8>,
}

impl ScreenBacklight {
    pub fn new() -> Self {
        Self {
            saved_brightness: None,
        }
    }

    pub fn lock(&mut self) {
        match Disp::open() {
            Ok(disp) => {
                if self.saved_brightness.is_none() {
                    self.saved_brightness = disp
                        .get_brightness()
                        .ok()
                        .filter(|value| *value > 0)
                        .or(Some(FALLBACK_RESTORE_BRIGHTNESS));
                }
                if let Err(err) = disp.set_brightness(0) {
                    eprintln!("display: failed to turn off lcd backlight: {err}");
                } else {
                    eprintln!("display: lcd backlight off");
                }
            }
            Err(err) => {
                eprintln!("display: failed to open {DISP_DEVICE}: {err}");
            }
        }
    }

    pub fn unlock(&mut self) {
        let brightness = self
            .saved_brightness
            .take()
            .unwrap_or(FALLBACK_RESTORE_BRIGHTNESS);
        match Disp::open() {
            Ok(disp) => {
                if let Err(err) = disp.set_brightness(brightness) {
                    eprintln!("display: failed to restore lcd backlight: {err}");
                } else {
                    eprintln!("display: lcd backlight restored raw={brightness}");
                }
            }
            Err(err) => {
                eprintln!("display: failed to open {DISP_DEVICE}: {err}");
            }
        }
    }
}

struct Disp {
    file: File,
}

impl Disp {
    fn open() -> io::Result<Self> {
        Ok(Self {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .open(DISP_DEVICE)?,
        })
    }

    fn set_brightness(&self, value: u8) -> io::Result<()> {
        let mut args = lcd_brightness_args(value);
        ioctl_ok(self.file.as_raw_fd(), DISP_LCD_SET_BRIGHTNESS, &mut args)
    }

    fn get_brightness(&self) -> io::Result<u8> {
        let mut args = [LCD_SCREEN, 0, 0, 0];
        let ret = ioctl_ret(self.file.as_raw_fd(), DISP_LCD_GET_BRIGHTNESS, &mut args)?;
        Ok(ret.clamp(0, u8::MAX as i32) as u8)
    }
}

fn lcd_brightness_args(value: u8) -> [libc::c_ulong; 4] {
    [LCD_SCREEN, value as libc::c_ulong, 0, 0]
}

fn ioctl_ok(fd: libc::c_int, cmd: libc::c_ulong, args: &mut [libc::c_ulong; 4]) -> io::Result<()> {
    let ret = unsafe { libc::ioctl(fd, cmd as _, args.as_mut_ptr()) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn ioctl_ret(
    fd: libc::c_int,
    cmd: libc::c_ulong,
    args: &mut [libc::c_ulong; 4],
) -> io::Result<i32> {
    let ret = unsafe { libc::ioctl(fd, cmd as _, args.as_mut_ptr()) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunxi_lcd_ioctl_constants_match_nextui_and_disp2_docs() {
        assert_eq!(DISP_LCD_SET_BRIGHTNESS, 0x102);
        assert_eq!(DISP_LCD_GET_BRIGHTNESS, 0x103);
        assert_eq!(DISP_LCD_BACKLIGHT_ENABLE, 0x104);
        assert_eq!(DISP_LCD_BACKLIGHT_DISABLE, 0x105);
    }

    #[test]
    fn brightness_ioctl_args_use_screen_zero_and_requested_value() {
        assert_eq!(lcd_brightness_args(37), [0, 37, 0, 0]);
    }
}
