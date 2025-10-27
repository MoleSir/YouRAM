mod error;
mod cells;
mod types;
mod config;
mod information;
use cells::PdkCells;
pub use error::*;
use information::PdkInformation;
use reda_unit::{Capacitance, Temperature, Time, Voltage};
pub use types::*;
pub use config::*;

use std::path::{Path, PathBuf};
use reda_lib::model::LibLibrary;
use reda_sp::Spice;
use crate::{circuit::{Dff, DriveStrength, Leafcell, LogicGate, LogicGateKind, Shr}, ErrorContext, YouRAMError, YouRAMResult};

pub struct Pdk {
    config: PdkConfig,
    infomation: PdkInformation,
    cells: PdkCells,
}

// Interface for config
impl Pdk {
    pub fn nmos_model_path(&self, process: Process) -> Result<PathBuf, PdkError> {
        self.config.nmos_model_path(process)
            .ok_or_else(|| PdkError::NmosModelNotFound(process))
    }

    pub fn pmos_model_path(&self, process: Process) -> Result<PathBuf, PdkError> {
        self.config.pmos_model_path(process)
            .ok_or_else(|| PdkError::NmosModelNotFound(process))
    }

    #[inline]
    pub fn pdk_root_path(&self) -> &Path {
        &self.config.pdk_path
    }

    #[inline]
    pub fn stdcell_liberty_path(&self) -> PathBuf {
        self.config.stdcell_liberty_path()
    }

    #[inline]
    pub fn stdcell_spice_path(&self) -> PathBuf {
        self.config.stdcell_spice_path()
    }

    #[inline]
    pub fn leafcell_spice_path(&self) -> PathBuf {
        self.config.leafcell_spice_path()
    }
}

// Interface for infomation
impl Pdk {
    #[inline]
    pub fn name(&self) -> &str {
        &self.infomation.name
    }

    #[inline]
    pub fn pvt(&self) -> &Pvt {
        &self.infomation.pvt
    }

    #[inline]
    pub fn nom_process(&self) -> Option<f64> {
        self.infomation.nom_process
    }

    #[inline]
    pub fn nom_temperature(&self) -> Option<Temperature> {
        self.infomation.nom_temperature
    }

    #[inline]
    pub fn nom_voltage(&self) -> Option<Voltage> {
        self.infomation.nom_voltage
    }

    #[inline]
    pub fn default_inout_pin_cap(&self) -> Option<Capacitance> {
        self.infomation.default_inout_pin_cap
    }

    #[inline]
    pub fn default_input_pin_cap(&self) -> Option<Capacitance> {
        self.infomation.default_input_pin_cap
    }

    #[inline]
    pub fn default_output_pin_cap(&self) -> Option<Capacitance> {
        self.infomation.default_output_pin_cap
    }

    #[inline]
    pub fn default_fanout_load(&self) -> Option<Capacitance> {
        self.infomation.default_fanout_load
    }

    #[inline]
    pub fn default_max_transition(&self) -> Option<Time> {
        self.infomation.default_max_transition
    }

    #[inline]
    pub fn slew_lower_threshold_pct_fall(&self) -> f64 {
        self.infomation.slew_lower_threshold_pct_fall
    }

    #[inline]
    pub fn slew_lower_threshold_pct_rise(&self) -> f64 {
        self.infomation.slew_lower_threshold_pct_rise
    }

    #[inline]
    pub fn slew_upper_threshold_pct_rise(&self) -> f64 {
        self.infomation.slew_upper_threshold_pct_rise
    }

    #[inline]
    pub fn slew_upper_threshold_pct_fall(&self) -> f64 {
        self.infomation.slew_upper_threshold_pct_fall
    }

    #[inline]
    pub fn input_threshold_pct_fall(&self) -> f64 {
        self.infomation.input_threshold_pct_fall
    }

    #[inline]
    pub fn input_threshold_pct_rise(&self) -> f64 {
        self.infomation.input_threshold_pct_rise
    }

    #[inline]
    pub fn output_threshold_pct_fall(&self) -> f64 {
        self.infomation.output_threshold_pct_fall
    }

    #[inline]
    pub fn output_threshold_pct_rise(&self) -> f64 {
        self.infomation.output_threshold_pct_rise
    }

    #[inline]
    pub fn timing_input_net_transitions(&self) -> &[Time] {
        &self.infomation.timing_input_net_transitions
    }

    #[inline]
    pub fn timing_output_net_capacitances(&self) -> &[Capacitance] {
        &self.infomation.timing_output_net_capacitances
    }
}

// Interface for cells
impl Pdk {
    #[inline]
    pub fn get_logicgate(&self, kind: LogicGateKind, drive_strength: DriveStrength) -> Option<Shr<LogicGate>> {
        self.cells.logicgates.get(&(kind, drive_strength)).cloned()
    }

    pub fn get_and(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<LogicGate>> {
        let kind = LogicGateKind::And(input_size);
        self.get_logicgate(kind, drive_strength)
    }

    pub fn get_nand(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<LogicGate>> {
        let kind = LogicGateKind::Nand(input_size);
        self.get_logicgate(kind, drive_strength)
    }

    pub fn get_or(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<LogicGate>> {
        let kind = LogicGateKind::Or(input_size);
        self.get_logicgate(kind, drive_strength)
    }

    pub fn get_nor(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<LogicGate>> {
        let kind = LogicGateKind::Nor(input_size);
        self.get_logicgate(kind, drive_strength)
    }

    pub fn get_inv(&self, drive_strength: DriveStrength) -> Option<Shr<LogicGate>> {
        let kind = LogicGateKind::Inv;
        self.get_logicgate(kind, drive_strength)
    }

    #[inline]
    pub fn get_dff(&self, drive_strength: DriveStrength) -> Option<Shr<Dff>> {
        self.cells.dffs.get(&drive_strength).cloned()
    }

    #[inline]
    pub fn get_bitcell(&self) -> Shr<Leafcell> {
        self.cells.bitcell.clone()
    }

    #[inline]
    pub fn get_sense_amp(&self) -> Shr<Leafcell> {
        self.cells.sense_amp.clone()
    }

    #[inline]
    pub fn get_write_driver(&self) -> Shr<Leafcell> {
        self.cells.write_driver.clone()
    }

    #[inline]
    pub fn get_column_trigate(&self) -> Shr<Leafcell> {
        self.cells.column_trigate.clone()
    }

    #[inline]
    pub fn get_precharge(&self) -> Shr<Leafcell> {
        self.cells.precharge.clone()
    }
}

impl Pdk {
    pub fn load<P: AsRef<Path>>(pdk_path: P) -> YouRAMResult<Self> {
        // load config
        let pdk_path: &Path = pdk_path.as_ref();
        let config = PdkConfig::load(pdk_path)?;

        // load file
        let library = LibLibrary::load_file(config.stdcell_liberty_path()).map_err(PdkError::Liberty)?;
        let stdcell_spice = Spice::load_from(config.stdcell_spice_path()).map_err(|e| YouRAMError::Message(e.to_string()))?;
        let leafcell_spice = Spice::load_from(config.leafcell_spice_path()).map_err(|e| YouRAMError::Message(e.to_string()))?;

        // extract logicgates & dff
        let cells = PdkCells::load(&library, &stdcell_spice, &leafcell_spice).context("load cells")?;

        // extract infomation 
        let infomation = PdkInformation::load(&library, &cells)?;

        Ok(Self {
            config,
            cells,
            infomation,
        })
    }
}

#[cfg(test)]
mod test {
    use reda_sp::ToSpice;
    use crate::{circuit::{DriveStrength, Primitive}, pdk::Process};

    use super::Pdk;

    #[test]
    fn test_load_pdk() {
        let pdk = Pdk::load("./platforms/nangate45").unwrap();
        
        let and2_x2 = pdk.get_and(2, DriveStrength::X2).unwrap();
        println!("{}", and2_x2.read().netlist().to_spice());

        let bitcell = pdk.get_bitcell();
        println!("{}", bitcell.read().netlist().to_spice());

        println!("{:?}", pdk.config.models.get(&Process::TypeType).unwrap());

        println!("{:?}", pdk.nmos_model_path(Process::TypeType).unwrap());
        println!("{:?}", pdk.pdk_root_path());
        println!("{}", pdk.name());
        println!("{}", pdk.pvt());

        // println!("{:?}", pdk.get_dff(DriveStrength::X1).unwrap().read().setup_rising_timing);

        println!("{:?}", pdk.timing_input_net_transitions());
        println!("{:?}", pdk.timing_output_net_capacitances());
    }
}