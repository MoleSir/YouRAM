use youram_macro::module;
use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};

#[module(
    select:               ("sel{select_size}", Input),
    bitline:              ("bl{select_size}", InOut),
    bitline_bar:          ("br{select_size}", InOut),
    bitline_selected:     ("bl", InOut),
    bitline_bar_selected: ("br", InOut),
    vdd:                  ("vdd", Source),
    gnd:                  ("gnd", Source),
)]
pub struct ColumnMux {
    pub select_size: usize,
}

impl ColumnMux {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        for i in 0..self.args.select_size {
            self.link_column_trigate_instance(
                factory, 
                format_shr!("column_mux_{}", i), 
                Self::bitline_pn(i),
                Self::bitline_bar_pn(i), 
                Self::bitline_selected_pn(), 
                Self::bitline_bar_selected_pn(), 
                Self::select_pn(i), 
                Self::vdd_pn(),
                Self::gnd_pn()
            )?;
        }
        Ok(())
    }
}