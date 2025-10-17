use youram_macro::module;
use crate::{check_arg, circuit::CircuitFactory, YouRAMResult};

#[module(
    bitline:      ("bl{colum_size}", InOut),
    bitline_bar:  ("br{colum_size}", InOut),
    wordline:     ("wl{row_size}", Input),
    vdd:          ("vdd", Source),
    gnd:          ("gnd", Source),
)]
pub struct BitcellArray {
    pub row_size: usize,
    pub colum_size: usize,   
}

impl BitcellArray {

    /*
    
         +----------------------------+
     wln |                            |
         |                            |
                                      |
         .                            |
         .                            |
         .                            |
                                      |
     wl2 |                            |
         |                            |
     wl1 |                            |
         |                            |
     wl0 |                            |
         |                            |
         +----------------------------+
            bl0 bl1 bl2   ...     bln
    
    */     
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.row_size >= 1, "row size {} < 1", self.args.row_size);
        check_arg!(self.args.colum_size >= 1, "column size {} < 1", self.args.colum_size);

        for row in 0..self.args.row_size {
            for col in 0..self.args.colum_size {
                self.link_bitcell_instance(
                    factory, 
                    format!("bitcell_{}_{}", row, col), 
                    Self::bitline_pn(col), 
                    Self::bitline_bar_pn(col),
                    Self::wordline_pn(row), 
                    Self::vdd_pn(),
                    Self::gnd_pn(),
                )?;
            }
        }

        Ok(())
    }
}