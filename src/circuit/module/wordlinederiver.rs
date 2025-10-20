use youram_macro::module;
use crate::{check_arg, circuit::{CircuitFactory, DriveStrength}, YouRAMResult};

use super::BufferArg;

#[module(
    wordline_input:  ("wl_in", Input),
    wordline:        ("wl", Output),
    vdd:             ("vdd", Source),
    gnd:             ("gnd", Source),
)]
pub struct WordlineDriver {
    pub fanout: usize,
}

impl WordlineDriver {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.fanout > 0, "Fanout size '{}' less than 1", self.args.fanout);
        
        let strength = match self.args.fanout {
            fanout if fanout > 1 => DriveStrength::X1,
            fanout if fanout > 16 => DriveStrength::X2,
            _ => DriveStrength::X4,
        };

        let buffer = self.add_module(BufferArg::new(strength), factory)?;
        self.link_module_instance("buffer", buffer, [
            Self::wordline_input_pn(),
            Self::wordline_pn(),
            Self::vdd_pn(),
            Self::gnd_pn(),
        ].into_iter())?;

        Ok(())
    }
}
