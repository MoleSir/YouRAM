use youram_macro::module;
use crate::{circuit::{CircuitFactory, DriveStrength, LogicGateKind}, format_shr, YouRAMResult};

#[module(
    input:  ("A{size}", Input),
    enbale: ("en", Input),
    output: ("Z{size}", Output),
    vdd:    ("vdd", Source),
    gnd:    ("gnd", Source),
)]
pub struct AndArray {
    pub size: usize,
}

impl AndArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let and = self.add_logicgate(LogicGateKind::And(2), DriveStrength::X2, factory)?;
        for and_index in 0..self.args.size {
            self.link_logicgate_instance(
                format_shr!("and{}", and_index), 
                and.clone(), 
                vec![Self::input_pn(and_index), Self::enbale_pn()], 
                Self::output_pn(and_index), 
                Self::vdd_pn(), 
                Self::gnd_pn(),
            )?;
        }

        Ok(())
    }
}