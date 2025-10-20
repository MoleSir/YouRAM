use youram_macro::module;
use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};

#[module(
    data_input:          ("din{column_size}", Input),
    bitline:             ("bl{column_size}", InOut),
    bitline_bar:         ("br{column_size}", InOut),
    enable:              ("we_en", Input),
    vdd:                 ("vdd", Source),
    gnd:                 ("gnd", Source),
)]
pub struct WriteDriverArray {
    pub column_size: usize,
    pub spare_column_size: usize,
}

impl WriteDriverArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        for wd_index in 0..self.args.column_size {
            self.link_writedriver_instance(
                factory, 
                format_shr!("write_driver{}", wd_index), 
                Self::data_input_pn(wd_index), 
                Self::bitline_pn(wd_index), 
                Self::bitline_bar_pn(wd_index), 
                Self::enable_pn(), 
                Self::vdd_pn(), 
                Self::gnd_pn()
            )?;   
        }

        Ok(())
    }
}