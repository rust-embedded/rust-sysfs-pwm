// Copyright 2016, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
//
// Portions of this implementation are based on work by Nat Pryce:
// https://github.com/npryce/rusty-pi/blob/master/src/pi/gpio.rs

#![crate_type = "lib"]
#![crate_name = "sysfs_pwm"]

//! PWM access under Linux using the PWM sysfs interface

use std::io::prelude::*;
use std::os::unix::prelude::*;
use std::fs::File;
use std::fs;

mod error;
pub use error::Error;

#[derive(Debug)]
pub struct PwmChip {
    chip: u32
}

#[derive(Debug)]
pub struct Pwm {
    chip: PwmChip,
    number: u32,
}

pub type Result<T> = ::std::result::Result<T, error::Error>;

impl PwmChip {
    pub fn new(chip: u32) -> Result<PwmChip> {
        try!(fs::metadata(&format!("/sys/class/pwm/pwmchip{}", chip)));
        Ok(PwmChip { chip: chip })
    }

    pub fn count(&self) -> Result<u32> {
        let npwm_path = format!("/sys/class/pwm/pwmchip{}/npwm", self.chip);
        let mut npwm_file = try!(File::open(&npwm_path));
        let mut s = String::new();
        try!(npwm_file.read_to_string(&mut s));
        match s.parse::<u32>() {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::Unexpected(
                format!("Unexpected npwm contents: {:?}", s))),
        }
    }

    pub fn export(&self, number: u32) -> Result<()> {
        // only export if not already exported
        if let Err(_) = fs::metadata(&format!("/sys/class/pwm/pwmchip{}/pwm{}", self.chip, number)) {
            let path = format!("/sys/class/pwm/pwmchip{}/export", self.chip);
            let mut export_file = try!(File::create(&path));
            let _ = export_file.write_all(format!("{}", number).as_bytes());
        }
        Ok(())
    }

    pub fn unexport(&self, number: u32) -> Result<()> {
        if let Ok(_) = fs::metadata(&format!("/sys/class/pwm/pwmchip{}/pwm{}", self.chip, number)) {
            let path = format!("/sys/class/pwm/pwmchip{}/unexport", self.chip);
            let mut export_file = try!(File::create(&path));
            let _ = export_file.write_all(format!("{}", number).as_bytes());
        }
        Ok(())
    }
}

impl Pwm {

    /// Create a new Pwm wiht the provided chip/number
    ///
    /// This function does not export the Pwm pin
    pub fn new(chip: u32, number: u32) -> Result<Pwm> {
        let chip: PwmChip = try!(PwmChip::new(chip));
        Ok(Pwm { chip: chip, number: number })
    }

    /// Run a closure with the GPIO exported
    #[inline]
    pub fn with_exported<F>(&self, closure: F) -> Result<()> where
        F: FnOnce() -> Result<()>
    {
        try!(self.export());
        match closure() {
            Ok(()) => self.unexport(),
            Err(_) => self.unexport(),
        }
    }

    /// Export the Pwm for use
    pub fn export(&self) -> Result<()> {
        self.chip.export(self.number)
    }

    /// Unexport the PWM
    pub fn unexport(&self) -> Result<()> {
        self.chip.unexport(self.number)
    }

}
