mod meas;
mod execute;
mod write;
mod error;
pub use write::*;
pub use meas::*;
pub use error::*;
pub use execute::*;

use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};
use reda_unit::{t, v, Number, Time, Voltage};
use itertools::Itertools;
use crate::{circuit::{PortDirection, ShrCircuit}, export, pdk::{Enviroment, Pdk}, ErrorContext, YouRAMResult};

pub struct CircuitSimulator {
    pub writor: SpiceWritor,
    pub circuit: ShrCircuit,
    pub env: Enviroment,
    pub pdk: Arc<Pdk>,
    pub circuit_path: PathBuf,
}

impl CircuitSimulator {
    pub const VDD_PORT_NAME: &'static str = "VDD";
    pub const GND_PORT_NAME: &'static str = "VSS";
    pub const CLOSK_PORT_NAME: &'static str = "CLK";

    /// Create a circuit simulator, and write these auto:
    /// - include file
    /// - vdd/gnd source
    /// - temperature
    /// - instance of this circuit(all net has the same name with circuit's port)
    /// 
    /// after `create`, you may need to write:
    /// - input stimulate 
    /// - meas
    /// - trans command
    /// 
    /// and last call `simulate` method to get result  
    pub fn create<P1, P2, C>(
        circuit: C, 
        env: Enviroment,
        pdk: Arc<Pdk>,
        simulate_path: P1,
        circuit_path: P2, 
    ) -> YouRAMResult<Self> 
    where 
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        C: Into<ShrCircuit>,
    {        
        let writor = SpiceWritor::open(simulate_path)?;
        let mut simulator = Self { writor, circuit: circuit.into(), env, pdk, circuit_path: circuit_path.as_ref().into() };
        simulator.init()?;
        Ok(simulator)
    }

    fn init(&mut self) -> YouRAMResult<()> {
        // write circuit to a spice file
        export::write_spice(self.circuit.clone(), &self.circuit_path)
            .with_context(|| format!("write circuit '{}' to '{:?}'", self.circuit.name(), self.circuit_path))?;
        let nmos_model_path = self.pdk.nmos_model_path(self.env.process())?;
        let pmos_model_path = self.pdk.pmos_model_path(self.env.process())?;

        // write includes 
        self.writor.write_content("\n")?;
        self.writor.write_include(nmos_model_path)?;
        self.writor.write_include(pmos_model_path)?;
        self.writor.write_include(&self.circuit_path)?;
        self.writor.write_content("\n")?;

        // write enviroment
        self.write_dc_stimulate(Self::VDD_PORT_NAME, self.env.voltage())?;
        self.write_dc_stimulate(Self::GND_PORT_NAME, 0.0)?;
        self.writor.write_temperature(self.env.temperature())?;

        // write circuit instance
        let mut nets = vec![];
        for port in self.circuit.ports().iter() {
            match port.read().direction {
                PortDirection::Vdd => nets.push(Self::VDD_PORT_NAME.to_string()),
                PortDirection::Gnd => nets.push(Self::GND_PORT_NAME.to_string()),
                _ => nets.push(port.read().name.to_string()),
                
            }
        }
        self.writor.write_instance(self.circuit.name(), self.circuit.name(), nets.into_iter())?;
        for port in self.circuit.ports().iter() {
            self.writor.write_capacitance(&port.read().name, &port.read().name, Self::GND_PORT_NAME, self.env.output_load())?;
        }
        self.writor.write_content("\n")?;

        Ok(())
    }

    pub fn simulate(self, execute: impl ExecuteCommand, temp_folder: impl AsRef<Path>) -> YouRAMResult<HashMap<String, Number>> {
        let mut executor = self.writor.close()?;
        executor.simulate(execute, temp_folder.as_ref())
    }
}

impl CircuitSimulator {
    pub fn logic1_voltage(&self) -> Voltage {
        self.env.voltage()
    } 

    pub fn logic0_voltage(&self) -> Voltage {
        0.0.into()
    }

    pub fn logic_voltage(&self, bit: bool) -> Voltage {
        if bit { self.logic1_voltage() } else { self.logic0_voltage() }
    }
}

impl CircuitSimulator {
    pub fn write_clock(&mut self, period: impl Into<Time>) -> YouRAMResult<()> {
        let period = period.into();
        self.writor.write_pulse_voltage(
            Self::CLOSK_PORT_NAME, 
            Self::CLOSK_PORT_NAME,
            self.env.voltage(),
            v!(0),
            t!(0),
            self.env.input_slew(),
            self.env.input_slew(),
            period / 2.0 - self.env.input_slew(),
            period
        )?;
        Ok(())
    }

    pub fn write_dc_stimulate(&mut self, port_name: impl AsRef<str>, voltage: impl Into<Voltage>) -> YouRAMResult<()> {
        let port_name = port_name.as_ref();
        self.writor.write_dc_voltage(port_name, port_name, voltage)
    }

    pub fn write_period_stimulate(
        &mut self,
        port_name: impl AsRef<str>,
        voltages: &[Voltage],
        period: impl Into<Time>,
        time_bias: impl Into<Time>, 
    ) -> YouRAMResult<()> {
        let port_name = port_name.as_ref();
        let period = period.into();
        let time_bias = time_bias.into();

        let times: Vec<_> = voltages.iter()
            .enumerate()
            .map(|(period_index, _)| {
                let time: Time = period_index as f64 * period + time_bias;
                time.max(t!(0))
            })  
            .collect();
        self.writor.write_square_wave_voltage(port_name, port_name, &times, voltages, self.env.input_slew())
    }

    pub fn write_square_wave_stimulate(
        &mut self,
        port_name: impl AsRef<str>,
        time_voltages: impl Iterator<Item = (Time, Voltage)>,
    ) -> YouRAMResult<()> {
        let port_name = port_name.as_ref();
        let (times, voltages): (Vec<_>, Vec<_>) = time_voltages.multiunzip();
        self.writor.write_square_wave_voltage(port_name, port_name, &times, &voltages, self.env.input_slew())
    }

    pub fn write_pwl_stimulate(
        &mut self,
        port_name: impl AsRef<str>,
        time_voltages: impl Iterator<Item = (Time, Voltage)>,
    ) -> YouRAMResult<()> {
        let port_name = port_name.as_ref();
        let (times, voltages): (Vec<_>, Vec<_>) = time_voltages.multiunzip();
        self.writor.write_pwl_voltage(port_name, port_name, times.into_iter(), voltages.into_iter())
    }

    #[inline]
    pub fn write_logic1_stimulate(&mut self, port_name: impl AsRef<str>) -> YouRAMResult<()> {
        self.write_dc_stimulate(port_name, self.env.voltage())
    }

    #[inline]
    pub fn write_logic0_stimulate(&mut self, port_name: impl AsRef<str>) -> YouRAMResult<()> {
        self.write_dc_stimulate(port_name, 0.0)
    }

    #[inline]
    pub fn write_measurement(&mut self, meas: Box<dyn Meas>) -> YouRAMResult<()> {
        self.writor.write_measurement(meas)
    }

    #[inline]
    pub fn write_trans(&mut self, step: impl Into<Time>, start: impl Into<Time>, end: impl Into<Time>) -> YouRAMResult<()> {
        self.writor.write_trans(step, start, end)
    }
}