use youram_macro::module;
use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};

#[module(
    bitline:       ("bl{column_size}", InOut),
    bitline_bar:   ("br{column_size}", InOut),
    data_output:   ("dout{column_size}", Output),
    enable:        ("sa_en", Input),
    vdd:           ("vdd", Vdd),
    gnd:           ("gnd", Gnd),
)]
pub struct SenseAmpArray {
    pub column_size: usize,
    pub spare_column_size: usize,
}

impl SenseAmpArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        for sa_index in 0..self.args.column_size {
            self.link_senseamp_instance(
                factory, 
                format_shr!("sense_amp{}", sa_index), 
                Self::bitline_pn(sa_index), 
                Self::bitline_bar_pn(sa_index), 
                Self::data_output_pn(sa_index), 
                Self::enable_pn(), 
                Self::vdd_pn(), 
                Self::gnd_pn()
            )?;  
        }

        Ok(())
    }
}