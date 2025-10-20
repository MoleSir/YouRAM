use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use reda_unit::Number;
use crate::{ErrorContext, YouRAMResult};
use super::{Executer, Meas, SimulateError};

pub struct Simulator {
    pub simulate_path: PathBuf,
    pub file: File,
    pub measurements: Vec<Box<dyn Meas>>, 
}

impl Simulator {
    pub fn create<P: AsRef<Path>>(simulate_path: P) -> YouRAMResult<Self> {
        let simulate_path = simulate_path.as_ref();
        let file = File::create(simulate_path)?;
        Ok(Self {
            simulate_path: simulate_path.to_path_buf(),
            file,
            measurements: vec![],
        })
    }
}

impl Simulator {
    pub fn simulate(&mut self, exceuter: Executer, temp_folder: impl AsRef<Path>) -> YouRAMResult<HashMap<String, Number>> {
        let result_path = exceuter.execute(&self.simulate_path, temp_folder).context("Execute simualte")?;
        self.get_meas_results(&result_path).context("Get meas result")
    }   

    fn get_meas_results(&mut self, result_path: impl AsRef<Path>) -> YouRAMResult<HashMap<String, Number>> {
        self.file.flush()?;
        let result_path = result_path.as_ref();
        let content = std::fs::read_to_string(result_path).context(format!("read result file '{:?}'", result_path))?;

        let mut results = HashMap::new();
        for meas in self.measurements.iter() {
            let value = meas.get_result(&content).map_err(SimulateError::MeasError)?;
            results.insert(meas.name().to_string(), value);
        }

        Ok(results)
    }
}

impl Simulator {
    pub fn write_content(&mut self, content: impl AsRef<str>) -> YouRAMResult<()> {
        write!(self.file, "{}", content.as_ref())?;
        Ok(())
    }

    pub fn write_char(&mut self, ch: char) -> YouRAMResult<()> {
        write!(self.file, "{}", ch)?;
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

    pub fn write_temperature(&mut self, temp: Number) -> YouRAMResult<()> {
        writeln!(self.file, ".TEMP {}", temp)?;
        Ok(())
    }

    pub fn write_instance(
        &mut self,
        module_name: impl AsRef<str>,
        instance_name: impl AsRef<str>,
        nets: &[impl AsRef<str>],
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
        times: &[Number],
        voltages: &[Number],
    ) -> YouRAMResult<()> {
        if times.len() != voltages.len() {
            return Err(SimulateError::TimesAndVoltageUnmatch(times.len(), voltages.len()))?;
        }

        write!(self.file, "V{} {} 0 PWL (", voltage_name.as_ref(), net_name.as_ref())?;
        for (t, v) in times.iter().zip(voltages) {
            write!(self.file, "{} {} ", t, v)?;
        }
        writeln!(self.file, ")")?;

        Ok(())
    }

    pub fn write_square_wave_voltage(
        &mut self,
        voltage_name: impl AsRef<str>,
        net_name: impl AsRef<str>,
        times: &[Number],
        voltages: &[Number],
        slew: Number,
    ) -> YouRAMResult<()> {
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
        init_voltage: Number,
        pulse_voltage: Number,
        delay: Number,
        rise: Number,
        fall: Number,
        width: Number,
        period: Number,
    ) -> YouRAMResult<()> {
        writeln!(
            self.file,
            "V{} {} 0 PULSE({} {} {} {} {} {} {})",
            voltage_name.as_ref(),
            net_name.as_ref(),
            init_voltage, pulse_voltage,
            delay, rise, fall,
            width, period
        )?;

        Ok(())
    }

    pub fn write_dc_voltage(
        &mut self,
        voltage_name: impl AsRef<str>,
        net_name: impl AsRef<str>,
        voltage: Number,
    ) -> YouRAMResult<()> {
        writeln!(self.file, "V{} {} 0 {}", voltage_name.as_ref(), net_name.as_ref(), voltage)?;
        Ok(())
    }

    pub fn write_capacitance(
        &mut self,
        name: impl AsRef<str>,
        n1: impl AsRef<str>,
        n2: impl AsRef<str>,
        value: Number,
    ) -> YouRAMResult<()> {
        writeln!(self.file, "C{} {} {} {}", name.as_ref(), n1.as_ref(), n2.as_ref(), value)?;
        Ok(())
    }

    pub fn write_trans(&mut self, step: Number, start: Number, end: Number) -> YouRAMResult<()> {
        writeln!(self.file, ".TRAN {} {} {}", step, end, start)?;
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
    use crate::simulate::MeasError;

    use super::*;
    use std::io::Read;
    use reda_unit::num;
    use tempfile::NamedTempFile;
    use std::fs::OpenOptions;

    struct DummyMeas;

    impl Meas for DummyMeas {
        fn write_command(&self, out: &mut dyn std::io::Write) -> std::io::Result<()> {
            writeln!(out, ".MEAS TRAN dummy FIND V(out) AT=5n")
        }

        fn get_result(&self, _content: &str) -> Result<Number, MeasError> {
            Ok(num!( 42 n ))
        }

        fn name(&self) -> &str {
            "dummy"
        }
    }

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
        let mut sim = Simulator {
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

        let mut sim = Simulator {
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
            &[num!(0), num!(1 n)], 
            &[num!(0), num!(1.8)]
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

        let mut sim = Simulator {
            simulate_path: path.clone(),
            file,
            measurements: vec![],
        };

        sim.write_instance("inv", "inv0", &["in", "out", "vdd", "gnd"]).unwrap();
        sim.write_capacitance("load", "out", "gnd", num!(0.01)).unwrap();
        sim.file.flush().unwrap();

        let content = read_file_to_string(&path);
        assert!(content.contains("Xinv0 in out vdd gnd inv"));
        assert!(content.contains("Cload out gnd 0.01"));
    }

    #[test]
    fn test_measurement_written_and_parsed() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let file = OpenOptions::new().read(true).write(true).open(&path).unwrap();

        let mut sim = Simulator {
            simulate_path: path.clone(),
            file,
            measurements: vec![],
        };

        sim.write_measurement(Box::new(DummyMeas)).unwrap();
        sim.file.flush().unwrap();

        let content = read_file_to_string(&path);
        assert!(content.contains(".MEAS TRAN dummy FIND"));

        // simulate dummy
        let results = sim.get_meas_results(path).unwrap();
        assert_eq!(results["dummy"], num!(42 n));
    }
}
