use youram_macro::module;
use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};

#[module(
    bitline:       ("bl{column_size}", InOut),
    bitline_bar:   ("br{column_size}", InOut),
    enable:        ("p_en_bar", Input),
    vdd:           ("vdd", Vdd),
)]
pub struct PrechargeArray {
    pub column_size: usize,
}

impl PrechargeArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        for sa_index in 0..self.args.column_size {
            self.link_precharge_instance(
                factory, 
                format_shr!("precharge{}", sa_index), 
                Self::bitline_pn(sa_index), 
                Self::bitline_bar_pn(sa_index), 
                Self::enable_pn(), 
                Self::vdd_pn(), 
            )?;  
        }

        Ok(())
    }
}