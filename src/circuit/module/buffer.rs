use anyhow::Context;
use youram_macro::module;
use crate::circuit::{CircuitFactory, DriveStrength, StdcellKind, StdcellPort};

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
    pub fn build(&mut self, factory: &mut CircuitFactory) -> anyhow::Result<()> {
        let inv = self.add_stdcell(StdcellKind::Inv, self.args.strength, factory).context("create inv")?;

        let instance1 = self.add_instance("inv1", inv.clone()).context("create inv1 instance")?;
        self.connect_instance(instance1.clone(), [
            (StdcellPort::Input(0), Self::input_pn()),
            (StdcellPort::Output,   "Z_bar".into()),
            (StdcellPort::Vdd,      Self::vdd_pn()),
            (StdcellPort::Gnd,      Self::gnd_pn()),
        ].into_iter()).context("connect with inv1")?;

        let instance2 = self.add_instance("inv2", inv.clone()).context("create inv2 instance")?;
        self.connect_instance(instance2.clone(), [
            (StdcellPort::Input(0), "Z_bar".into()),
            (StdcellPort::Output,   Self::output_pn()),
            (StdcellPort::Vdd,      Self::vdd_pn()),
            (StdcellPort::Gnd,      Self::gnd_pn()),
        ].into_iter()).context("connect with inv2")?;

        Ok(())
    }
}
