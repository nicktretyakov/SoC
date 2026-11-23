//! Minimal MPU6050 driver over the ESP-IDF I2C peripheral.
//!
//! Only the accelerometer is used (the on-board gyro and DMP are out of scope
//! for activity recognition by acceleration magnitude). The driver wakes the
//! chip, then repeatedly reads the six `ACCEL_*OUT` registers and converts the
//! signed 16-bit values into *g* using the default +/-2 g full-scale range
//! (sensitivity 16384 LSB/g).

use std::time::Duration;

use esp_idf_hal::i2c::{BLOCKING, I2cDriver};

use crate::error::{AppError, AppResult};
use crate::sensor::traits::Sensor;
use crate::sensor::types::AccelSample;
use crate::util::now_ms;

/// MPU6050 register addresses we touch. See the InvenSense datasheet, section
/// 4.28/4.7.
const REG_PWR_MGMT_1: u8 = 0x6B;
const REG_WHO_AM_I: u8 = 0x75;
const REG_ACCEL_XOUT_H: u8 = 0x3B;

/// Expected `WHO_AM_I` reply. 0x68 for the MPU6050, 0x70 for the MPU6500 clone.
const WHO_AM_I_MPU6050: u8 = 0x68;

/// LSB per g at the default +/-2 g full-scale range (`AFS_SEL = 0`).
const LSB_PER_G_2G: f32 = 16384.0;

/// The driver. Owns the I2C bus exclusively; the lifetime `'d` is inherited
/// from the `I2cDriver` (i.e. from the borrowed peripherals / config).
pub struct Mpu6050<'d> {
    bus: I2cDriver<'d>,
    addr: u8,
}

impl<'d> Mpu6050<'d> {
    /// Create and initialise the sensor: claim the bus, wake the chip from
    /// sleep, and sanity-check the `WHO_AM_I` register so a wiring fault fails
    /// fast instead of producing zeroes.
    pub fn new(bus: I2cDriver<'d>, addr: u8) -> AppResult<Self> {
        let mut me = Self { bus, addr };

        // Wake: write 0 to PWR_MGMT_1 (clears SLEEP). A small delay lets the
        // internal PLL stabilise before the first read.
        me.write_reg(REG_PWR_MGMT_1, 0x00)?;
        std::thread::sleep(Duration::from_millis(100));

        let who = me.read_reg(REG_WHO_AM_I)?;
        if who != WHO_AM_I_MPU6050 {
            // Not fatal: the MPU9250 / clones reply 0x71/0x73. Warn, keep going.
            log::warn!(
                "MPU6050: WHO_AM_I=0x{:02X} (expected 0x{:02X}); continuing",
                who,
                WHO_AM_I_MPU6050
            );
        }
        log::info!("MPU6050 initialised at I2C 0x{:02X}", addr);
        Ok(me)
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> AppResult<()> {
        self.bus
            .write(self.addr, &[reg, value], BLOCKING)
            .map_err(AppError::i2c)
    }

    fn read_reg(&mut self, reg: u8) -> AppResult<u8> {
        let mut buf = [0u8; 1];
        self.bus
            .write_read(self.addr, &[reg], &mut buf, BLOCKING)
            .map_err(AppError::i2c)?;
        Ok(buf[0])
    }

    /// Read all six accelerometer bytes in one I2C transaction.
    fn read_accel_bytes(&mut self) -> AppResult<[u8; 6]> {
        let mut buf = [0u8; 6];
        self.bus
            .write_read(self.addr, &[REG_ACCEL_XOUT_H], &mut buf, BLOCKING)
            .map_err(AppError::i2c)?;
        Ok(buf)
    }

    /// Decode two big-endian bytes into signed g.
    #[inline]
    fn raw_to_g(hi: u8, lo: u8) -> f32 {
        let raw = i16::from_be_bytes([hi, lo]);
        raw as f32 / LSB_PER_G_2G
    }
}

impl<'d> Sensor for Mpu6050<'d> {
    fn read_accel(&mut self) -> AppResult<AccelSample> {
        let b = self.read_accel_bytes()?;
        Ok(AccelSample {
            ax_g: Self::raw_to_g(b[0], b[1]),
            ay_g: Self::raw_to_g(b[2], b[3]),
            az_g: Self::raw_to_g(b[4], b[5]),
            t_ms: now_ms(),
        })
    }
}
