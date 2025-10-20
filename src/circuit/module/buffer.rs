use youram_macro::module;
use crate::{circuit::{CircuitFactory, DriveStrength, LogicGateKind}, YouRAMResult};

#[module(
    input:  ("A", Input),
    output: ("Z", Output),
    vdd:    ("vdd", Source),
    gnd:    ("gnd", Source),
)]
pub struct Buffer {
    pub strength: DriveStrength,
}

impl Buffer {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let inv = self.add_logicgate(LogicGateKind::Inv, self.args.strength, factory)?;
        self.link_inv_instance("inv1", inv.clone(), [Self::input_pn(), "Z_bar".into(), Self::vdd_pn(), Self::gnd_pn()])?;
        self.link_inv_instance("inv2", inv.clone(), ["Z_bar".into(), Self::output_pn(), Self::vdd_pn(), Self::gnd_pn()])?;
        Ok(())
    }
}
