use youram_macro::module;

use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};

use super::ColumnMuxArg;

#[module(
    select:               ("sel{select_size}", Input),

    bitline:              ("bl{mux_size}_{select_size}", InOut),
    bitline_bar:          ("br{mux_size}_{select_size}", InOut),

    bitline_selected:     ("bl{mux_size}", InOut),
    bitline_bar_selected: ("br{mux_size}", InOut),
 
    vdd:                  ("vdd", Source),
    gnd:                  ("gnd", Source),
)]
pub struct ColumnMuxArray {
    pub select_size: usize,
    pub mux_size: usize,
}

impl ColumnMuxArray {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let mux = self.add_module(ColumnMuxArg::new(self.args.select_size), factory)?;
        for mux_index in 0..self.args.mux_size {
            let mut nets = vec![];
            
            for sel_index in 0..self.args.select_size {
                nets.push(Self::select_pn(sel_index));
            }

            for sel_index in 0..self.args.select_size {
                nets.push(Self::bitline_pn(mux_index, sel_index));
            }

            for sel_index in 0..self.args.select_size {
                nets.push(Self::bitline_bar_pn(mux_index, sel_index));
            }
        
            nets.push(Self::bitline_selected_pn(mux_index));
            nets.push(Self::bitline_bar_selected_pn(mux_index));

            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance(format_shr!("mux{}", mux_index), mux.clone(), nets.into_iter())?;
        }

        Ok(())
    }
}