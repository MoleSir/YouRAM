use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Context;
use tracing::info;
use crate::pdk::Pdk;
use super::{Circuit, DriveStrength, Module, Shr, ShrString, StdcellKind};

pub trait ModuleArg: Sized {
    fn module_name(&self) -> ShrString;
    fn create_module(self, factory: &mut CircuitFactory) -> anyhow::Result<Module<Self>>;
}

pub struct CircuitFactory {
    pub pdk: Arc<Pdk>,
    modules: HashMap<ShrString, Shr<Circuit>>,
}

impl CircuitFactory {
    pub fn new(pdk: Arc<Pdk>) -> Self {
        Self {
            pdk,
            modules: HashMap::new(),
        }
    }

    pub fn module<A: ModuleArg + 'static>(&mut self, arg: A) -> anyhow::Result<Shr<Circuit>> {
        let name = arg.module_name();
        if let Some(circuit) = self.modules.get(&name) {
            Ok(circuit.clone())
        } else {
            info!("Create circuit '{}'", name);
            let module = arg
                .create_module(self)
                .context(format!("Create circuit '{}'", name))?;
            let module = Shr::new(Circuit::Module(Box::new(module)));
            self.modules.insert(name, module.clone());
            Ok(module)
        }
    }

    pub fn stdcell(&self, kind: StdcellKind, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.pdk.get_stdcell(kind, drive_strength)
    }

    pub fn and(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.pdk.get_and(input_size, drive_strength)
    }

    pub fn nand(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.pdk.get_nand(input_size, drive_strength)
    }

    pub fn or(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.pdk.get_or(input_size, drive_strength)
    }

    pub fn nor(&self, input_size: usize, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.pdk.get_or(input_size, drive_strength)
    }

    pub fn inv(&self, drive_strength: DriveStrength) -> Option<Shr<Circuit>> {
        self.pdk.get_inv(drive_strength)
    }

    pub fn bitcell(&self) -> Shr<Circuit> {
        self.pdk.get_bitcell().clone()
    }
}