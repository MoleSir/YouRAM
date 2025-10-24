use youram_macro::module;
use crate::{check_arg, circuit::CircuitFactory, format_shr, YouRAMResult};

use super::WordlineDriverArg;

#[module(
    wordline_input: ("wl_in{wordline_size}", Input),
    wordline:       ("wl{wordline_size}", Input),
    vdd:            ("vdd", Vdd),
    gnd:            ("gnd", Gnd),
)]
pub struct WordlineDriverArray {
    pub fanout: usize,
    pub wordline_size: usize,
}

impl WordlineDriverArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.fanout > 0, "Fanout size '{}' less than 1", self.args.fanout);
        check_arg!(self.args.wordline_size > 0, "wordline size size '{}' less than 1", self.args.wordline_size);

        let wordline = self.add_module(WordlineDriverArg::new(self.args.fanout), factory)?;
        for wordline_index in 0..self.args.wordline_size {
            self.link_module_instance(
                format_shr!("wordline_driver{}", wordline_index), 
                wordline.clone(), [
                    Self::wordline_input_pn(wordline_index),
                    Self::wordline_input_pn(wordline_index),
                    Self::vdd_pn(),
                    Self::gnd_pn(),
                ].into_iter()
            )?;
        }

        Ok(())
    }
}