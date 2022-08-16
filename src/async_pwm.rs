use std::str::FromStr;
use tokio::fs;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::macros::support::Future;

use crate::error;
use crate::Error;

#[derive(Debug)]
pub struct PwmAsync {
    chip: PwmChipAsync,
    number: u32,
}

#[derive(Debug)]
pub struct PwmChipAsync {
    pub number: u32,
}

/// Open the specified entry name as a writable file
async fn pwm_file_wo(chip: &PwmChipAsync, pin: u32, name: &str) -> Result<File, error::Error> {
    let f = OpenOptions::new()
        .write(true)
        .open(format!(
            "/sys/class/pwm/pwmchip{}/pwm{}/{}",
            chip.number, pin, name
        ))
        .await?;
    Ok(f)
}

/// Open the specified entry name as a readable file
async fn pwm_file_ro(chip: &PwmChipAsync, pin: u32, name: &str) -> Result<File, error::Error> {
    let f = File::open(format!(
        "/sys/class/pwm/pwmchip{}/pwm{}/{}",
        chip.number, pin, name
    ))
    .await?;
    Ok(f)
}

/// Get the u32 value from the given entry
async fn pwm_file_parse<T: FromStr>(
    chip: &PwmChipAsync,
    pin: u32,
    name: &str,
) -> Result<T, error::Error> {
    let mut s = String::with_capacity(10);
    let mut f = pwm_file_ro(chip, pin, name).await?;
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
async fn pwm_capture_parse<T: FromStr>(
    chip: &PwmChipAsync,
    pin: u32,
    name: &str,
) -> Result<Vec<T>, error::Error> {
    let mut s = String::with_capacity(10);
    let mut f = pwm_file_ro(chip, pin, name).await?;
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

impl PwmChipAsync {
    pub async fn new(number: u32) -> Result<PwmChipAsync, error::Error> {
        fs::metadata(&format!("/sys/class/pwm/pwmchip{}", number)).await?;
        Ok(PwmChipAsync { number: number })
    }

    pub async fn count(&self) -> Result<u32, error::Error> {
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

    pub async fn export(&self, number: u32) -> Result<(), error::Error> {
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
            let _ = export_file
                .write_all(format!("{}", number).as_bytes())
                .await;
            let _ = export_file.sync_all().await;
        }
        Ok(())
    }

    pub async fn unexport(&self, number: u32) -> Result<(), error::Error> {
        if fs::metadata(&format!(
            "/sys/class/pwm/pwmchip{}/pwm{}",
            self.number, number
        ))
        .await
        .is_ok()
        {
            let path = format!("/sys/class/pwm/pwmchip{}/unexport", self.number);
            let mut export_file = File::create(&path).await?;
            let _ = export_file
                .write_all(format!("{}", number).as_bytes())
                .await;
            let _ = export_file.sync_all().await;
        }
        Ok(())
    }
}
impl PwmAsync {
    /// Create a new Pwm wiht the provided chip/number
    ///
    /// This function does not export the Pwm pin
    pub async fn new(chip: u32, number: u32) -> Result<PwmAsync, error::Error> {
        let chip: PwmChipAsync = PwmChipAsync::new(chip).await?;
        Ok(PwmAsync {
            chip: chip,
            number: number,
        })
    }

    /// Run a closure with the GPIO exported
    #[inline]
    pub async fn with_exported<F>(
        &self,
        closure: impl Future<Output = F>,
    ) -> Result<(), error::Error>
    where
        F: FnOnce() -> Result<(), error::Error>,
    {
        self.export().await?;
        let y = closure.await;
        match y() {
            Ok(()) => self.unexport().await,
            Err(e) => match self.unexport().await {
                Ok(()) => Err(e),
                Err(ue) => Err(error::Error::Unexpected(format!(
                    "Failed unexporting due to:\n{}\nwhile handling:\n{}",
                    ue, e
                ))),
            },
        }
    }

    /// Export the Pwm for use
    pub async fn export(&self) -> Result<(), error::Error> {
        self.chip.export(self.number).await
    }

    /// Unexport the PWM
    pub async fn unexport(&self) -> Result<(), error::Error> {
        self.chip.unexport(self.number).await
    }

    /// Query the state of enable for a given PWM pin
    pub async fn get_enabled(&self) -> Result<bool, error::Error> {
        pwm_file_parse::<u32>(&self.chip, self.number, "enable")
            .await
            .map(|enable_state| match enable_state {
                1 => true,
                0 => false,
                _ => panic!("enable != 1|0 should be unreachable"),
            })
    }

    /// Get the capture
    pub async fn get_capture(&self) -> Result<(u32, u32), error::Error> {
        let t = pwm_capture_parse::<u32>(&self.chip, self.number, "capture").await?;
        if t.len() == 2 {
            Ok((t[0], t[1]))
        } else {
            Err(error::Error::Unexpected(format!("Failed exporting")))
        }
    }

    /// Get the currently configured duty_cycle as percentage of period
    pub async fn get_duty_cycle_async(&self) -> Result<f32, error::Error> {
        Ok((self.get_duty_cycle_ns().await? as f32) / (self.get_period_ns().await? as f32))
    }

    /// Get the currently configured duty_cycle in nanoseconds
    pub async fn get_duty_cycle_ns(&self) -> Result<u32, error::Error> {
        pwm_file_parse::<u32>(&self.chip, self.number, "duty_cycle").await
    }

    /// Get the currently configured period in nanoseconds
    pub async fn get_period_ns(&self) -> Result<u32, error::Error> {
        pwm_file_parse::<u32>(&self.chip, self.number, "period").await
    }

    /// The period of the PWM signal in Nanoseconds
    pub async fn set_period_ns(&self, period_ns: u32) -> Result<(), error::Error> {
        let mut period_file = pwm_file_wo(&self.chip, self.number, "period").await?;
        period_file
            .write_all(format!("{}", period_ns).as_bytes())
            .await?;
        let _ = period_file.sync_all().await;
        Ok(())
    }

    /// The active time of the PWM signal
    ///
    /// Value is in nanoseconds and must be less than the period.
    pub async fn set_duty_cycle_ns(&self, duty_cycle_ns: u32) -> Result<(), error::Error> {
        // we'll just let the kernel do the validation
        let mut duty_cycle_file = pwm_file_wo(&self.chip, self.number, "duty_cycle").await?;
        duty_cycle_file
            .write_all(format!("{}", duty_cycle_ns).as_bytes())
            .await?;
        let _ = duty_cycle_file.sync_all().await;
        Ok(())
    }

    /// Enable/Disable the PWM Signal
    pub async fn enable(&self, enable: bool) -> Result<(), error::Error> {
        let mut enable_file = pwm_file_wo(&self.chip, self.number, "enable").await?;
        let contents = if enable { "1" } else { "0" };
        enable_file.write_all(contents.as_bytes()).await?;
        let _ = enable_file.sync_all().await;
        Ok(())
    }
}
