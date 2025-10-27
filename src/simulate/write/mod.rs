use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use reda_unit::{Capacitance, Temperature, Time, Voltage};
use crate::YouRAMResult;
use crate::simulate::{Meas, SimulateError};

use super::SpiceExector;

pub struct SpiceWritor {
    simulate_path: PathBuf,
    file: File,
    measurements: Vec<Box<dyn Meas>>, 
}

impl SpiceWritor {
    pub fn open<P: Into<PathBuf>>(simulate_path: P) -> YouRAMResult<Self> {
        let simulate_path = simulate_path.into();
        let file = File::create(&simulate_path)?;
        Ok(Self {
            simulate_path: simulate_path.to_path_buf(),
            file,
            measurements: vec![],
        })
    }

    pub fn close(mut self) -> YouRAMResult<SpiceExector> {
        self.file.flush()?;
        Ok(SpiceExector {
            simulate_path: self.simulate_path,
            measurements: self.measurements
        })
    }
}

#[allow(dead_code)]
impl SpiceWritor {
    pub fn simulate_path(&self) -> &Path {
        &self.simulate_path
    }

    pub fn write_content(&mut self, content: impl AsRef<str>) -> YouRAMResult<()> {
        write!(self.file, "{}", content.as_ref())?;
        Ok(())
    }

    pub fn write_include<P: AsRef<Path>>(&mut self, path: P) -> YouRAMResult<()> {
        writeln!(self.file, ".include {}", path.as_ref().display())?;
        Ok(())
    }

    pub fn write_end(&mut self) -> YouRAMResult<()> {
        writeln!(self.file, ".end")?;
        Ok(())
    }

    pub fn write_comment(&mut self, comment: impl AsRef<str>) -> YouRAMResult<()> {
        writeln!(self.file, "* {}", comment.as_ref())?;
        Ok(())
    }

    pub fn write_temperature(&mut self, temp: impl Into<Temperature>) -> YouRAMResult<()> {
        let temp: Temperature = temp.into();
        writeln!(self.file, ".TEMP {}", temp.value())?;
        Ok(())
    }

    pub fn write_instance(
        &mut self,
        module_name: impl AsRef<str>,
        instance_name: impl AsRef<str>,
        nets: impl Iterator<Item = impl AsRef<str>>,
    ) -> YouRAMResult<()> {
        write!(self.file, "X{}", instance_name.as_ref())?;
        for net in nets {
            write!(self.file, " {}", net.as_ref())?;
        }
        writeln!(self.file, " {}", module_name.as_ref())?;
        Ok(())
    }

    pub fn write_pwl_voltage(
        &mut self,
        voltage_name: impl AsRef<str>,
        net_name: impl AsRef<str>,
        times: impl ExactSizeIterator<Item = Time>,
        voltages: impl ExactSizeIterator<Item = Voltage>,
    ) -> YouRAMResult<()> {
        if times.len() != voltages.len() {
            return Err(SimulateError::TimesAndVoltageUnmatch(times.len(), voltages.len()))?;
        }

        write!(self.file, "V{} {} 0 PWL (", voltage_name.as_ref(), net_name.as_ref())?;
        for (t, v) in times.zip(voltages) {
            write!(self.file, "{} {} ", t, v)?;
        }
        writeln!(self.file, ")")?;

        Ok(())
    }

    pub fn write_square_wave_voltage(
        &mut self,
        voltage_name: impl AsRef<str>,
        net_name: impl AsRef<str>,
        times: &[Time],
        voltages: &[Voltage],
        slew: impl Into<Time>,
    ) -> YouRAMResult<()> {
        let slew = slew.into();
        if times.len() != voltages.len() {
            return Err(SimulateError::TimesAndVoltageUnmatch(times.len(), voltages.len()))?;
        }
        
        write!(self.file, "V{} {} 0 PWL (", voltage_name.as_ref(), net_name.as_ref())?;
        if times.is_empty() {
            writeln!(self.file, ")")?;
            return Ok(())
        }

        write!(self.file, "{} {} ", times[0], voltages[0])?;
        
        for i in 1..times.len() {
            write!(self.file, "{} {} ", times[i] - slew, voltages[i - 1])?;
            write!(self.file, "{} {} ", times[i] + slew, voltages[i])?
        }
        
        writeln!(self.file, ")")?;
        Ok(())
    }

    pub fn write_pulse_voltage(
        &mut self,
        voltage_name: impl AsRef<str>,
        net_name: impl AsRef<str>,
        init_voltage: impl Into<Voltage>,
        pulse_voltage: impl Into<Voltage>,
        delay: impl Into<Time>,
        rise: impl Into<Time>,
        fall: impl Into<Time>,
        width: impl Into<Time>,
        period: impl Into<Time>,
    ) -> YouRAMResult<()> {
        writeln!(
            self.file,
            "V{} {} 0 PULSE({} {} {} {} {} {} {})",
            voltage_name.as_ref(),
            net_name.as_ref(),
            init_voltage.into(), pulse_voltage.into(),
            delay.into(), rise.into(), fall.into(),
            width.into(), period.into()
        )?;

        Ok(())
    }

    pub fn write_dc_voltage(
        &mut self,
        voltage_name: impl AsRef<str>,
        net_name: impl AsRef<str>,
        voltage: impl Into<Voltage>,
    ) -> YouRAMResult<()> {
        writeln!(self.file, "V{} {} 0 {}", voltage_name.as_ref(), net_name.as_ref(), voltage.into())?;
        Ok(())
    }

    pub fn write_capacitance(
        &mut self,
        name: impl AsRef<str>,
        n1: impl AsRef<str>,
        n2: impl AsRef<str>,
        value: impl Into<Capacitance>,
    ) -> YouRAMResult<()> {
        writeln!(self.file, "C{} {} {} {}", name.as_ref(), n1.as_ref(), n2.as_ref(), value.into())?;
        Ok(())
    }

    pub fn write_trans(&mut self, step: impl Into<Time>, start: impl Into<Time>, end: impl Into<Time>) -> YouRAMResult<()> {
        writeln!(self.file, ".TRAN {} {} {}", step.into(), end.into(), start.into())?;
        Ok(())
    }

    pub fn write_measurement(&mut self, meas: Box<dyn Meas>) -> YouRAMResult<()> {
        meas.write_command(&mut self.file)?;
        self.measurements.push(meas);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use reda_unit::{num, t, v};
    use tempfile::NamedTempFile;
    use std::fs::OpenOptions;

    fn read_file_to_string(path: &PathBuf) -> String {
        let mut f = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();
        content
    }

    #[test]
    fn test_basic_commands_written() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let file = OpenOptions::new().read(true).write(true).open(&path).unwrap();
        let mut sim = SpiceWritor {
            simulate_path: path.clone(),
            file,
            measurements: vec![],
        };

        sim.write_include("model.sp").unwrap();
        sim.write_comment("test comment").unwrap();
        sim.write_temperature(num!(27)).unwrap();
        sim.write_end().unwrap();
        sim.file.flush().unwrap();

        let content = read_file_to_string(&path);
        assert!(content.contains(".include model.sp"));
        assert!(content.contains("* test comment"));
        assert!(content.contains(".TEMP 27"));
        assert!(content.contains(".end"));
    }

    #[test]
    fn test_voltage_sources() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let file = OpenOptions::new().read(true).write(true).open(&path).unwrap();

        let mut sim = SpiceWritor {
            simulate_path: path.clone(),
            file,
            measurements: vec![],
        };

        sim.write_dc_voltage("VDD", "vdd", num!(1.2)).unwrap();
        sim.write_pulse_voltage(
            "CLK", "clk", num!(0.0), num!(1.8),
            num!(1 n), num!(0.1 n), num!(0.1 n),
            num!(4.9 n), num!(10 n)
        ).unwrap();

        sim.write_pwl_voltage(
            "IN", "in", 
            [t!(0), t!(1 n)].into_iter(), 
            [v!(0), v!(1.8)].into_iter(), 
        ).unwrap();

        sim.file.flush().unwrap();
        let content = read_file_to_string(&path);
        println!("{}", content);
        assert!(content.contains("VDD vdd 0 1.2"));
        assert!(content.contains("PULSE("));
        assert!(content.contains("PWL"));
    }

    #[test]
    fn test_instance_and_cap() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let file = OpenOptions::new().read(true).write(true).open(&path).unwrap();

        let mut sim = SpiceWritor {
            simulate_path: path.clone(),
            file,
            measurements: vec![],
        };

        sim.write_instance("inv", "inv0", ["in", "out", "vdd", "gnd"].iter()).unwrap();
        sim.write_capacitance("load", "out", "gnd", num!(0.01)).unwrap();
        sim.file.flush().unwrap();

        let content = read_file_to_string(&path);
        assert!(content.contains("Xinv0 in out vdd gnd inv"));
        assert!(content.contains("Cload out gnd 0.01"));
    }
}
