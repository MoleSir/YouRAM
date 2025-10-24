use youram_macro::module;
use crate::{check_arg, circuit::CircuitFactory, YouRAMResult};

#[module(
    bitline:      ("bl{column_size}", InOut),
    bitline_bar:  ("br{column_size}", InOut),
    wordline:     ("wl{row_size}", Input),
    vdd:          ("vdd", Vdd),
    gnd:          ("gnd", Gnd),
)]
pub struct BitcellArray {
    pub row_size: usize,
    pub column_size: usize,   
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
        check_arg!(self.args.column_size >= 1, "column size {} < 1", self.args.column_size);

        for row in 0..self.args.row_size {
            for col in 0..self.args.column_size {
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