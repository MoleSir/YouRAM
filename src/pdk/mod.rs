mod error;
mod cell;
pub use error::*;

use std::{collections::HashMap, path::Path};
use anyhow::Context;
use reda_lib::model::LibLibrary;
use reda_sp::Spice;
use serde::{Deserialize, Serialize};
use crate::circuit::{Circuit, DriveStrength, Shr, StdcellKind};

pub struct Pdk {
    pub config: PdkConfig,
    pub stdcells: HashMap<(StdcellKind, DriveStrength), Shr<Circuit>>,
    pub bitcell: Shr<Circuit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdkConfig {
    pub stdcell_liberty: String,
    pub stdcell_spice: String,
    pub leafcell_spice: String,
}

impl Pdk {
    pub fn get_stdcell(&self, kind: StdcellKind, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.stdcells.get(&(kind, drive_strength)).cloned()
    }

    pub fn get_and(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        let kind = StdcellKind::And(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_nand(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        let kind = StdcellKind::Nand(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_or(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        let kind = StdcellKind::Or(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_nor(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        let kind = StdcellKind::Nor(input_size);
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_inv(&self, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        let kind = StdcellKind::Inv;
        self.get_stdcell(kind, drive_strength)
    }

    pub fn get_bitcell(&self) -> Shr<Circuit> {
        self.bitcell.clone()
    }
}

impl Pdk {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // load config
        let path: &Path = path.as_ref();
        let config_path = path.join("config.json");
        let config = std::fs::read_to_string(config_path).context("read pdk config")?;
        let config: PdkConfig = serde_json::from_str(&config).context("parse pdk config")?;

        // load file
        let stdcell_liberty_path = path.join(&config.stdcell_liberty);
        let stdcell_spice_path = path.join(&config.stdcell_spice);
        let leafcell_spice_path = path.join(&config.leafcell_spice);
        let library = LibLibrary::load_file(stdcell_liberty_path)?;
        let stdcell_spice = Spice::load_from(stdcell_spice_path)?;
        let leafcell_spice = Spice::load_from(leafcell_spice_path)?;

        // extract stdcells
        let mut stdcells = HashMap::new();
        for cell in library.cells.iter() {
            if let Some(stdcell) = Self::extract_stdcell(cell, &stdcell_spice) {
                let key = (stdcell.kind, stdcell.drive_strength);
                stdcells.insert(key, Shr::new(Circuit::Stdcell(stdcell)));
            }
        }

        // extract bitcell
        let bitcell 
            = Shr::new(Circuit::Leafcell(Self::extract_bitcell(&leafcell_spice).context("extract bitcell")?.into()));

        Ok(Self {
            config,
            stdcells,
            bitcell
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
        println!("{}", and2_x2.read().stdcell().unwrap().netlist().to_spice());

        let bitcell = pdk.get_bitcell();
        println!("{}", bitcell.read().leafcell().unwrap().netlist().to_spice());

    }
}