use youram_macro::module;

use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};

use super::{ColumnMuxArrayArg, SenseAmpArrayArg, WriteDriverArrayArg};

#[module(
    sense_amp_enable:    ("sa_en", Input),
    write_driver_enable: ("we_en", Input),

    bitline:             ("bl{column_size}", InOut),
    bitline_bar:         ("br{column_size}", InOut),

    select:              ("sel{column_sel_size}", Input, "column_sel_size > 1"),

    data_input:          ("din{word_width}", Input),
    data_output:         ("dout{word_width}", Input),
    
    vdd:                 ("vdd", Source),
    gnd:                 ("gnd", Source),
)]
pub struct DataPath {
    pub word_width: usize,
    pub column_sel_size: usize,

    #[new(value = "word_width * column_sel_size")]
    pub column_size: usize,
}

impl DataPath {
    /*
    
         |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | 
        +----+----+----+----+----+----+----+----+----+----+----+
        |    |    |    |   ...   |    |    |    |    |    |    |
        | PR | PR | PR |   ...   | PR | PR | PR | PR | PR | PR |
        |    |    |    |   ...   |    |    |    |    |    |    |
        +----+----+----+----+----+----+----+----+----+----+----+
         |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | 
         |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | 
        +----+----+----+----+----+----+----+----+----+----+----+
        |    |    |    |   ...   |    |    |    |    |    |    |
        | CL | CL | CL |   ...   | CL | CL | CL | CL | CL | CL |
        |    |    |    |   ...   |    |    |    |    |    |    |
        +----+----+----+----+----+----+----+----+----+----+----+ 
         |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | |  | 
        --------------------------------------------------------
        --------------------------------------------------------
        --------------------------------------------------------
        --------------------------------------------------------
         |  |      |  |      |  |      |  |      |  |      |  |
        +----+    +----+    +----+    +----+    +----+    +----+
        |    |    |    |    |    |    |    |    |    |    |    |
        |    |    |    |    |    |    |    |    |    |    |    |
        | SA |    | SA |    | SA |    | SA |    | SA |    | SA |
        |    |    |    |    |    |    |    |    |    |    |    |
        |    |    |    |    |    |    |    |    |    |    |    |
        +----+    +----+    +----+    +----+    +----+    +----+
         |  |      |  |      |  |      |  |      |  |      |  |
         |  +------------------------------------------------------
         |  |      |  +--------------------------------------------
         |  |      |  |      |  +----------------------------------
         |  |      |  |      |  |      |  +------------------------
         |  |      |  |      |  |      |  |      |  +--------------
         |  |      |  |      |  |      |  |      |  |      |  +----
         |  |      |  |      |  |      |  |      |  |      |  |
        +----+    +----+    +----+    +----+    +----+    +----+
        |    |    |    |    |    |    |    |    |    |    |    |
        |    |    |    |    |    |    |    |    |    |    |    |
        | WD |    | WD |    | WD |    | WD |    | WD |    | WD |
        |    |    |    |    |    |    |    |    |    |    |    |
        |    |    |    |    |    |    |    |    |    |    |    |
        +----+    +----+    +----+    +----+    +----+    +----+

    */
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let write_array = self.add_module(WriteDriverArrayArg::new(self.args.word_width, self.args.column_sel_size - 1), factory)?;
        let senseamp_array = self.add_module(SenseAmpArrayArg::new(self.args.word_width, self.args.column_sel_size - 1), factory)?;

        let mut out_bl_nets = vec![];
        let mut out_br_nets = vec![]; 

        if self.use_mux() {
            let colmux_array = self.add_module(ColumnMuxArrayArg::new(self.args.column_sel_size, self.args.word_width), factory)?;
            
            for i in 0..self.args.word_width {
                out_bl_nets.push(format_shr!("out_bl{}", i));
                out_br_nets.push(format_shr!("out_br{}", i));
            }

            let mut bl_nets = vec![];
            let mut br_nets = vec![];
            let mut coladdr_nets = vec![];

            for i in 0..self.args.column_sel_size {
                coladdr_nets.push(Self::select_pn(i));
            }

            for i in 0..self.args.column_size {
                bl_nets.push(Self::bitline_pn(i));
                br_nets.push(Self::bitline_bar_pn(i));
            }

            // create and mux array
            let mut muxarray_nets = vec![];

            // "sel{select_size}"
            for net in coladdr_nets.iter() {
                muxarray_nets.push(net.clone());
            }
            // "bl{mux_size}_{select_size}"
            for mux in 0..self.args.word_width {
                for i in 0..self.args.column_sel_size {
                    muxarray_nets.push(bl_nets[mux * self.args.column_sel_size + i].clone());
                }
            }
            // "br{mux_size}_{select_size}"
            for mux in 0..self.args.word_width {
                for i in 0..self.args.column_sel_size {
                    muxarray_nets.push(br_nets[mux * self.args.column_sel_size + i].clone());
                }
            }
            // "bl{mux_size}"
            for mux in 0..self.args.word_width {
                muxarray_nets.push(out_bl_nets[mux].clone());
            }
            // "br{mux_size}"
            for mux in 0..self.args.word_width {
                muxarray_nets.push(out_br_nets[mux].clone());
            }

            muxarray_nets.push(Self::vdd_pn());
            muxarray_nets.push(Self::gnd_pn());
    
            self.link_module_instance("colmux_array", colmux_array, muxarray_nets.into_iter())?;
    
        } else {
            for word_index in 0..self.args.word_width {
                out_bl_nets.push(Self::bitline_pn(word_index));
                out_bl_nets.push(Self::bitline_bar_pn(word_index));
            }
        }   
        
        // sense amp
        {
            let mut nets = vec![];
            for word_index in 0..self.args.word_width {
                nets.push(out_bl_nets[word_index].clone());
            }
            for word_index in 0..self.args.word_width {
                nets.push(out_br_nets[word_index].clone());
            }
            for word_index in 0..self.args.word_width {
                nets.push(Self::data_output_pn(word_index));
            }
            nets.push(Self::sense_amp_enable_pn());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("senseamp_array", senseamp_array, nets.into_iter())?;
        }

        // write driver
        {
            let mut nets = vec![];
            for word_index in 0..self.args.word_width {
                nets.push(Self::data_input_pn(word_index));
            }
            for word_index in 0..self.args.word_width {
                nets.push(out_bl_nets[word_index].clone());
            }
            for word_index in 0..self.args.word_width {
                nets.push(out_br_nets[word_index].clone());
            }
            nets.push(Self::write_driver_enable_pn());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("write_array", write_array, nets.into_iter())?;
        }

        Ok(())
    }

    pub fn use_mux(&self) -> bool {
        self.args.column_sel_size > 1
    }
}