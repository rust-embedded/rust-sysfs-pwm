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
use std::str::FromStr;
use std::cmp::{min, max};
use std::path::Path;

mod error;
pub use error::Error;

#[derive(Debug)]
pub struct PwmChip {
    pub number: u32,
}

#[derive(Debug)]
pub struct Pwm {
    chip: PwmChip,
    number: u32,
}

#[derive(Debug)]
pub enum Polarity {
    Normal,
    Inverse,
}

pub type Result<T> = ::std::result::Result<T, error::Error>;

/// Open the specified entry name as a writable file
fn pwm_file_rw(chip: &PwmChip, pin: u32, name: &str) -> Result<File> {
    let f = try!(File::create(format!("/sys/class/pwm/pwmchip{}/pwm{}/{}",
                                      chip.number,
                                      pin,
                                      name)));
    Ok(f)
}

/// Open the specified entry name as a readable file
fn pwm_file_ro(chip: &PwmChip, pin: u32, name: &str) -> Result<File> {
    let f = try!(File::open(format!("/sys/class/pwm/pwmchip{}/pwm{}/{}", chip.number, pin, name)));
    Ok(f)
}

/// Get the u32 value from the given entry
fn pwm_file_parse<T: FromStr>(chip: &PwmChip, pin: u32, name: &str) -> Result<T> {
    let mut s = String::with_capacity(10);
    let mut f = try!(pwm_file_ro(chip, pin, name));
    try!(f.read_to_string(&mut s));
    match s.parse::<T>() {
        Ok(r) => Ok(r),
        Err(_) => Err(Error::Unexpected(format!("Unexpeted value file contents: {:?}", s))),
    }
}


impl PwmChip {
    pub fn new(number: u32) -> Result<PwmChip> {
        try!(fs::metadata(&format!("/sys/class/pwm/pwmchip{}", number)));
        Ok(PwmChip { number: number })
    }

    pub fn count(&self) -> Result<u32> {
        let npwm_path = format!("/sys/class/pwm/pwmchip{}/npwm", self.number);
        let mut npwm_file = try!(File::open(&npwm_path));
        let mut s = String::new();
        try!(npwm_file.read_to_string(&mut s));
        match s.parse::<u32>() {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::Unexpected(format!("Unexpected npwm contents: {:?}", s))),
        }
    }

    pub fn export(&self, number: u32) -> Result<()> {
        // only export if not already exported
        if let Err(_) = fs::metadata(&format!("/sys/class/pwm/pwmchip{}/pwm{}",
                                              self.number,
                                              number)) {
            let path = format!("/sys/class/pwm/pwmchip{}/export", self.number);
            let mut export_file = try!(File::create(&path));
            let _ = export_file.write_all(format!("{}", number).as_bytes());
        }
        Ok(())
    }

    pub fn unexport(&self, number: u32) -> Result<()> {
        if let Ok(_) = fs::metadata(&format!("/sys/class/pwm/pwmchip{}/pwm{}",
                                             self.number,
                                             number)) {
            let path = format!("/sys/class/pwm/pwmchip{}/unexport", self.number);
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
        Ok(Pwm {
            chip: chip,
            number: number,
        })
    }

    /// Run a closure with the GPIO exported
    #[inline]
    pub fn with_exported<F>(&self, closure: F) -> Result<()>
        where F: FnOnce() -> Result<()>
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

    /// Enable/Disable the PWM Signal
    pub fn set_active(&self, active: bool) -> Result<()> {
        let mut active_file = try!(pwm_file_rw(&self.chip, self.number, "active"));
        let contents = if active {
            "1"
        } else {
            "0"
        };
        try!(active_file.write_all(contents.as_bytes()));
        Ok(())
    }

    /// Get the currently configured duty cycle as a percentage
    pub fn get_duty_cycle(&self) -> Result<f32> {
        let raw_duty_cycle = try!(pwm_file_parse::<u32>(&self.chip, self.number, "duty"));
        Ok((raw_duty_cycle as f32) / 1000.0)
    }

    /// Set the duty cycle as a percentage of time active
    ///
    /// This value is expected to be a floating point value
    /// between 0.0 and 1.0.  It maps to a value with resolution 0 -
    /// 1000.  Values < 0 or > 1.0 are capped at the minimum or
    /// maximum respectively.
    pub fn set_duty_cycle(&self, percent: f32) -> Result<()> {
        let raw_percent_adj: u32 = (percent * 1000.0).floor() as u32;
        let percent_adj: u32 = max(0, min(raw_percent_adj, 1000));
        let mut dc_file = try!(pwm_file_rw(&self.chip, self.number, "duty"));
        try!(dc_file.write_all(format!("{}", percent_adj).as_bytes()));
        Ok(())
    }

    /// Get the currently configured duty_cycle in nanoseconds
    pub fn get_duty_cycle_ns(&self, duty_cycle_ns: u32) -> Result<u32> {
        pwm_file_parse::<u32>(&self.chip, self.number, "duty_cycle")
    }

    /// The active time of the PWM signal
    ///
    /// Value is in nanoseconds and must be less than the period.
    pub fn set_duty_cycle_ns(&self, duty_cycle_ns: u32) -> Result<()> {
        // we'll just let the kernel do the validation
        let mut duty_cycle_file = try!(pwm_file_rw(&self.chip, self.number, "duty_cycle"));
        try!(duty_cycle_file.write_all(format!("{}", duty_cycle_ns).as_bytes()));
        Ok(())
    }

    /// Get the currently configured period in nanoseconds
    pub fn get_period_ns(&self) -> Result<u32> {
        pwm_file_parse::<u32>(&self.chip, self.number, "period")
    }

    /// The period of the PWM signal in Nanoseconds
    pub fn set_period_ns(&self, period_ns: u32) -> Result<()> {
        let mut period_file = try!(pwm_file_rw(&self.chip, self.number, "period"));
        try!(period_file.write_all(format!("{}", period_ns).as_bytes()));
        Ok(())
    }
}
