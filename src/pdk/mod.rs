mod error;
mod cell;
pub use error::*;

use std::{collections::HashMap, path::Path};
use reda_lib::model::LibLibrary;
use reda_sp::Spice;
use serde::{Deserialize, Serialize};
use crate::{circuit::{DriveStrength, Leafcell, Shr, Stdcell, StdcellKind}, ErrorContext, YouRAMError, YouRAMResult};

pub struct Pdk {
    pub config: PdkConfig,
    pub stdcells: HashMap<(StdcellKind, DriveStrength), Shr<Stdcell>>,
    pub bitcell: Shr<Leafcell>,
    pub sense_amp: Shr<Leafcell>,
    pub write_driver: Shr<Leafcell>,
    pub column_trigate: Shr<Leafcell>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdkConfig {
    pub stdcell_liberty: String,
    pub stdcell_spice: String,
    pub leafcell_spice: String,
}

impl Pdk {
    pub fn get_stdcell(&self, kind: StdcellKind, drive_strength: DriveStrength) -> Option<Shr<Stdcell>> {
        self.stdcells.get(&(kind, drive_strength)).cloned()
    }

    pub fn get_and(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Stdcell>> {
        let kind = StdcellKind::And(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_nand(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Stdcell>> {
        let kind = StdcellKind::Nand(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_or(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Stdcell>> {
        let kind = StdcellKind::Or(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_nor(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Stdcell>> {
        let kind = StdcellKind::Nor(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_inv(&self, drive_strength: DriveStrength) -> Option<Shr<Stdcell>> {
        let kind = StdcellKind::Inv;
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_bitcell(&self) -> Shr<Leafcell> {
        self.bitcell.clone()
    }

    pub fn get_sense_amp(&self) -> Shr<Leafcell> {
        self.sense_amp.clone()
    }

    pub fn get_write_driver(&self) -> Shr<Leafcell> {
        self.write_driver.clone()
    }

    pub fn get_column_trigate(&self) -> Shr<Leafcell> {
        self.column_trigate.clone()
    }
}

impl Pdk {
    pub fn load<P: AsRef<Path>>(path: P) -> YouRAMResult<Self> {
        // load config
        let path: &Path = path.as_ref();
        let config_path = path.join("config.json");
        let config = std::fs::read_to_string(config_path).context("read pdk config")?;
        let config: PdkConfig = serde_json::from_str(&config).context("parse pdk config")?;

        // load file
        let stdcell_liberty_path = path.join(&config.stdcell_liberty);
        let stdcell_spice_path = path.join(&config.stdcell_spice);
        let leafcell_spice_path = path.join(&config.leafcell_spice);
        let library = LibLibrary::load_file(stdcell_liberty_path).map_err(PdkError::Liberty)?;
        let stdcell_spice = Spice::load_from(stdcell_spice_path).map_err(|e| YouRAMError::Message(e.to_string()))?;
        let leafcell_spice = Spice::load_from(leafcell_spice_path).map_err(|e| YouRAMError::Message(e.to_string()))?;

        // extract stdcells
        let mut stdcells = HashMap::new();
        for cell in library.cells.iter() {
            if let Some(stdcell) = Self::extract_stdcell(cell, &stdcell_spice) {
                let key = (stdcell.kind, stdcell.drive_strength);
                stdcells.insert(key, Shr::new(stdcell));
            }
        }

        // extract bitcell
        let bitcell 
            = Shr::new(Self::extract_bitcell(&leafcell_spice).context("extract bitcell")?.into());
        let sense_amp
            = Shr::new(Self::extract_sense_amp(&leafcell_spice).context("extract sens_amp")?.into());
        let write_driver
            = Shr::new(Self::extract_write_driver(&leafcell_spice).context("extract write_driver")?.into());
        let column_trigate
            = Shr::new(Self::extract_column_trigate(&leafcell_spice).context("extract column_trigate")?.into());    

        Ok(Self {
            config,
            stdcells,
            bitcell,
            sense_amp,
            write_driver,
            column_trigate,
        })
    }
}

#[cfg(test)]
mod test {
    use reda_sp::ToSpice;
    use crate::circuit::{DriveStrength, Primitive};

    use super::Pdk;

    #[test]
    fn test_load_pdk() {
        let pdk = Pdk::load("./platforms/nangate45").unwrap();
        
        let and2_x2 = pdk.get_and(2, DriveStrength::X2).unwrap();
        println!("{}", and2_x2.read().netlist().to_spice());

        let bitcell = pdk.get_bitcell();
        println!("{}", bitcell.read().netlist().to_spice());

    }
}