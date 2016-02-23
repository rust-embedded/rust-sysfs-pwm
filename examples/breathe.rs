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

extern crate sysfs_pwm;
use sysfs_pwm::{Pwm, Result};

const RPI_PWM_CHIP: u32 = 1;
const RPI_PWM_NUMBER: u32 = 127;

fn pwm_increase_to_max(pwm: &Pwm,
                       duration_ms: u32,
                       update_period: u32) -> Result<()> {
    let step: f32 = duration_ms as f32 / update_period as f32;
    let mut duty_cycle: f32 = 0.0;
    while duty_cycle < 1.0 {
        try!(pwm.set_duty_cycle(duty_cycle));
        duty_cycle += step;
    }
    pwm.set_duty_cycle(1.0)
}

fn pwm_decrease_to_minimum(pwm: &Pwm,
                           duration_ms: u32,
                           update_period: u32) -> Result<()> {
    let step: f32 = duration_ms as f32 / update_period as f32;
    let mut duty_cycle = 1.0;
    while duty_cycle > 0.0 {
        try!(pwm.set_duty_cycle(duty_cycle));
        duty_cycle -= step;
    }
    pwm.set_duty_cycle(0.0)
}

/// Make an LED "breathe" by increasing and
/// decreasing the brightness
fn main() {
    let pwm = Pwm::new(RPI_PWM_CHIP, RPI_PWM_NUMBER).unwrap(); // number depends on chip, etc.
    pwm.with_exported(|| {
        pwm.set_active(true).unwrap();
        loop {
            pwm_increase_to_max(&pwm, 1000, 20).unwrap();
            pwm_decrease_to_minimum(&pwm, 1000, 20).unwrap();
        }
    }).unwrap();
}
