use std::str::FromStr;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::macros::support::Future;

use crate::error;
use crate::Error;

use crate::Pwm;
use crate::PwmChip;
pub type Result<T> = ::std::result::Result<T, error::Error>;

/// Open the specified entry name as a readable file
async fn pwm_file_ro_async(chip: &PwmChip, pin: u32, name: &str) -> Result<File> {
    let f = File::open(format!(
        "/sys/class/pwm/pwmchip{}/pwm{}/{}",
        chip.number, pin, name
    ))
    .await?;
    Ok(f)
}

/// Get the u32 value from the given entry
async fn pwm_file_parse_async<T: FromStr>(chip: &PwmChip, pin: u32, name: &str) -> Result<T> {
    let mut s = String::with_capacity(10);
    let mut f = pwm_file_ro_async(chip, pin, name).await?;
    f.read_to_string(&mut s).await?;
    match s.trim().parse::<T>() {
        Ok(r) => Ok(r),
        Err(_) => Err(Error::Unexpected(format!(
            "Unexpeted value file contents: {:?}",
            s
        ))),
    }
}

/// Get the two u32 from capture file descriptor
async fn pwm_capture_parse_async<T: FromStr>(
    chip: &PwmChip,
    pin: u32,
    name: &str,
) -> Result<Vec<T>> {
    let mut s = String::with_capacity(10);
    let mut f = pwm_file_ro_async(chip, pin, name).await?;
    f.read_to_string(&mut s).await?;
    s = s.trim().to_string();
    let capture = s.split_whitespace().collect::<Vec<_>>();
    let mut vec: Vec<T> = vec![];
    for s in capture.iter() {
        if let Ok(j) = s.parse::<T>() {
            vec.push(j);
        }
    }
    Ok(vec)
}

impl PwmChip {
    pub async fn count_async(&self) -> Result<u32> {
        let npwm_path = format!("/sys/class/pwm/pwmchip{}/npwm", self.number);
        let mut npwm_file = File::open(&npwm_path).await?;
        let mut s = String::new();
        npwm_file.read_to_string(&mut s).await?;
        match s.parse::<u32>() {
            Ok(n) => Ok(n),
            Err(_) => Err(Error::Unexpected(format!(
                "Unexpected npwm contents: {:?}",
                s
            ))),
        }
    }

    pub async fn export_async(&self, number: u32) -> Result<()> {
        // only export if not already exported
        if fs::metadata(&format!(
            "/sys/class/pwm/pwmchip{}/pwm{}",
            self.number, number
        ))
        .await
        .is_err()
        {
            let path = format!("/sys/class/pwm/pwmchip{}/export", self.number);
            let mut export_file = File::create(&path).await?;
            let _ = export_file.write_all(format!("{}", number).as_bytes());
        }
        Ok(())
    }

    pub async fn unexport_async(&self, number: u32) -> Result<()> {
        if fs::metadata(&format!(
            "/sys/class/pwm/pwmchip{}/pwm{}",
            self.number, number
        ))
        .await
        .is_ok()
        {
            let path = format!("/sys/class/pwm/pwmchip{}/unexport", self.number);
            let mut export_file = File::create(&path).await?;
            let _ = export_file.write_all(format!("{}", number).as_bytes());
        }
        Ok(())
    }
}
impl Pwm {
    /// Run a closure with the GPIO exported
    #[inline]
    pub async fn with_exported_async<F>(&self, closure: impl Future<Output = F>) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        self.export_async().await?;
        let y = closure.await;
        match y() {
            Ok(()) => self.unexport_async().await,
            Err(e) => match self.unexport_async().await {
                Ok(()) => Err(e),
                Err(ue) => Err(error::Error::Unexpected(format!(
                    "Failed unexporting due to:\n{}\nwhile handling:\n{}",
                    ue, e
                ))),
            },
        }
    }

    /// Export the Pwm for use
    pub async fn export_async(&self) -> Result<()> {
        self.chip.export_async(self.number).await
    }

    /// Unexport the PWM
    pub async fn unexport_async(&self) -> Result<()> {
        self.chip.unexport_async(self.number).await
    }

    /// Query the state of enable for a given PWM pin
    pub async fn get_enabled_async(&self) -> Result<bool> {
        pwm_file_parse_async::<u32>(&self.chip, self.number, "enable")
            .await
            .map(|enable_state| match enable_state {
                1 => true,
                0 => false,
                _ => panic!("enable != 1|0 should be unreachable"),
            })
    }

    /// Get the capture
    pub async fn get_capture_async(&self) -> Result<(u32, u32)> {
        let t = pwm_capture_parse_async::<u32>(&self.chip, self.number, "capture").await?;
        if t.len() == 2 {
            Ok((t[0], t[1]))
        } else {
            Err(error::Error::Unexpected(format!("Failed exporting")))
        }
    }

    /// Get the currently configured duty_cycle as percentage of period
    pub async fn get_duty_cycle_async(&self) -> Result<f32> {
        Ok((self.get_duty_cycle_ns()? as f32) / (self.get_period_ns()? as f32))
    }
}
