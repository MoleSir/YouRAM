use youram_macro::module;
use crate::{check_arg, circuit::CircuitFactory, format_shr, YouRAMResult};

#[module(
    replical_bitline:     ("rbl", InOut),
    replical_bitline_bar: ("rbr", InOut),
    wordline_enbale:      ("wl", Input),
    vdd:                  ("vdd", Source),
    gnd:                  ("gnd", Source),
)]
pub struct ReplicaBitcellArray {
    pub bitcell_size: usize,
}

const LINKED_BITCELL_SIZE: usize = 2;

impl ReplicaBitcellArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.bitcell_size >= LINKED_BITCELL_SIZE, "Bitcell size < {}", LINKED_BITCELL_SIZE);
        
        for bitcell_index in 0..self.args.bitcell_size {
            self.link_bitcell_instance(
                factory, 
                format_shr!("bitcell{}", bitcell_index), 
                Self::replical_bitline_pn(),
                Self::replical_bitline_bar_pn(),
                if bitcell_index < LINKED_BITCELL_SIZE { Self::wordline_enbale_pn() } else { Self::gnd_pn() }, 
                Self::vdd_pn(), 
                Self::gnd_pn(),
            )?;
        }

        Ok(())
    }
}