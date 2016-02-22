rust-sysfs-pwm
==============

[![Build Status](https://travis-ci.org/posborne/rust-sysfs-pwm.svg?branch=master)](https://travis-ci.org/posborne/rust-sysfs-pwm)
[![Version](https://img.shields.io/crates/v/sysfs-pwm.svg)](https://crates.io/crates/sysfs-pwm)
[![License](https://img.shields.io/crates/l/sysfs-pwm.svg)](https://github.com/posborne/rust-sysfs-pwm/blob/master/README.md#license)

- [API Documentation](http://posborne.github.io/rust-sysfs-pwm/)

rust-sysfs-pwm is a rust library/crate providing access to the [Linux
sysfs PWM interface](https://www.kernel.org/doc/Documentation/pwm.txt).
It seeks to provide an API that is safe, convenient, and efficient.

Install/Use
-----------

To use `sysfs-pwm`, first add this to your `Cargo.toml`:

```toml
[dependencies]
# or latest version
sysfs-pwm = "^0.1.0"
```

Then, add this to your crate root:

```rust
extern crate sysfs_pwm;
```

Example/API
-----------

Controlling a PWM (this example works on the Rasbperry Pi):

```rust
extern crate sysfs_pwm;
use sysfs_pwm::{Pwm};

const RPI_PWM_CHIP: u32 = 1;

fn pwm_increase_to_max(pwm: &Pwm,
                       duration_ms: u32,
                       update_period: u32) {
    let mut step: f32 = duration_ms / update_period;
    let duty_cycle: f32 = 0.0;
    while duty_cycle < 1.0 {
        pwm.set_duty_cycle(duty_cycle);
        duty_cycle += step;
    }
    pwm.set_duty_cycle(1.0);
}

fn pwm_decrease_to_minimum(pwm: &Pwm,
                           duration_ms: u32,
                           update_period: u32) {
    let mut step: f32 = duration_ms / update_period;
    let mut duty_cycle = 1.0;
    while duty_cycle > 0.0 {
        pwm.set_duty_cycle(duty_cycle);
        duty_cycle -= step;
    }
    pwm.set_duty_cycle(0.0)
}

/// Make an LED "breathe" by increasing and
/// decreasing the brightness
fn main() {
    let my_pwm = Pwm::new(1, 127); // number depends on chip, etc.
    my_pwm.with_exported(|| {
        loop {
            pwm_increase_to_max(pwm, 1000, 20);
            pwm_decrease_to_minimum(pwm, 1000, 20);
        }
    }).unwrap();
}
```

Features
--------

...

Cross Compiling
---------------

Most likely, the machine you are running on is not your development
machine (although it could be).  In those cases, you will need to
cross-compile.  The [instructions here][rust-cross] provide great details on cross
compiling for your platform.

[rust-cross]: https://github.com/japaric/rust-cross

Running the Example
-------------------

Cross-compiling can be done by specifying an appropriate target.  You
can then move that to your device by whatever means and run it.

```
$ cargo build --target=arm-unknown-linux-gnueabihf --example breathe
$ scp target/arm-unknown-linux-gnueabihf/debug/examples/breathe ...
```

License
-------

```
Copyright (c) 2016, Paul Osborne <ospbau@gmail.com>

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
http://www.apache.org/license/LICENSE-2.0> or the MIT license
<LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
option.  This file may not be copied, modified, or distributed
except according to those terms.
```
