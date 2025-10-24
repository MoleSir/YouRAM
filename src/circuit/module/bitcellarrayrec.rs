use youram_macro::module;
use crate::{check_arg, circuit::CircuitFactory, format_shr, YouRAMResult};

#[module(
    bitline:      ("bl{column_size}", InOut),
    bitline_bar:  ("br{column_size}", InOut),
    wordline:     ("wl{row_size}", Input),
    vdd:          ("vdd", Vdd),
    gnd:          ("gnd", Gnd),
)]
pub struct BitcellArrayRecursive {
    pub row_size: usize,
    pub column_size: usize,   
}

impl BitcellArrayRecursive {

    /*

        Divide bitcellarray to subarray and some bitcell. For 
        - row size => rowSize = rowSubSize * rowSubCount + rowRemainder
        - col size => colSize = colSubSize * colSubCount + colRemainder

        So, there are four kinds of subarray:
        - 1. rowSubSize * colSubSize (rowSubCount * colSubCount)
        - 2. rowRemainder * colSubSize (colSubSize) 
        - 3. rowSubSize * colRemainder (rowSubSize)
        - 4. rowRemainder * colRemainder (1)

        +-----------+-----------+-----------+---------------+-----+
        |           |           |           |               |     |
        |     1     |     1     |     1     |    .......    |  3  |
        |           |           |           |               |     |
        +-----------+-----------+-----------+---------------+-----+
        |           |           |           |               |     |
        |     1     |     1     |     1     |    .......    |  3  |
        |           |           |           |               |     |
        +-----------+-----------+-----------+---------------+-----+
        |           |           |           |               |     |
        |     1     |     1     |     1     |    .......    |  3  |
        |           |           |           |               |     |
        +-----------+-----------+-----------+---------------+-----+
        |           |           |           |   .           |     |
        |     .     |     .     |     .     |     .         |  .  |
        |     .     |     .     |     .     |       .       |  .  |
        |     .     |     .     |     .     |         .     |  .  |
        |           |           |           |           .   |     |
        +-----------+-----------+-----------+---------------+-----+
        |     2     |     2     |     2     |    .......    |  4  |
        +-----------+-----------+-----------+---------------+-----+

    */ 
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.row_size >= 1, "row size {} < 1", self.args.row_size);
        check_arg!(self.args.column_size >= 1, "column size {} < 1", self.args.column_size);

        if self.args.row_size >= 4 || self.args.column_size >= 4 {
            let row_sub_array_info = SubArrayInfo::from_size(self.args.row_size);
            let col_sub_array_info = SubArrayInfo::from_size(self.args.column_size);

            let row_col_sub_array = 
                self.add_module(BitcellArrayRecursiveArg::new(row_sub_array_info.size, col_sub_array_info.size), factory)?;

            for row in 0..row_sub_array_info.count {
                for col in 0..col_sub_array_info.count {
                    let mut nets = vec![];
                    let base_row_index = row * row_sub_array_info.size;
                    let base_col_index = col * col_sub_array_info.size;
                    
                    for i in 0..col_sub_array_info.size {
                        nets.push(Self::bitline_pn(base_col_index + i));
                    }

                    for i in 0..col_sub_array_info.size {
                        nets.push(Self::bitline_bar_pn(base_col_index + i));
                    }

                    for i in 0..row_sub_array_info.size {
                        nets.push(Self::wordline_pn(base_row_index + i));
                    }
                    
                    nets.push(Self::vdd_pn());
                    nets.push(Self::gnd_pn());

                    let inst_name = format_shr!("rowcol_subarray_{}_{}", row, col);
                    let instance = self.add_instance(inst_name, row_col_sub_array.clone())?;
                    self.connect_instance(instance, nets.into_iter())?;
                }
            }

            if row_sub_array_info.remainder >= 1 {
                let row_sub_array = self.add_module(BitcellArrayRecursiveArg::new(row_sub_array_info.remainder, col_sub_array_info.size), factory)?;
                let base_row_index = row_sub_array_info.count * row_sub_array_info.size;
                for col in 0..col_sub_array_info.count {
                    let mut nets = vec![];
                    let base_col_index = col * col_sub_array_info.size;

                    for i in 0..col_sub_array_info.size {
                        nets.push(Self::bitline_pn(base_col_index + i));
                    }

                    for i in 0..col_sub_array_info.size {
                        nets.push(Self::bitline_bar_pn(base_col_index + i));
                    }

                    for i in 0..row_sub_array_info.remainder {
                        nets.push(Self::wordline_pn(base_row_index + i));
                    }

                    nets.push(Self::vdd_pn());
                    nets.push(Self::gnd_pn());

                    let inst_name = format_shr!("row_subarray_{}", col);
                    let instance = self.add_instance(inst_name, row_sub_array.clone())?;
                    self.connect_instance(instance, nets.into_iter())?;
                }
            }

            if col_sub_array_info.remainder >= 1 {
                let col_sub_array = self.add_module(BitcellArrayRecursiveArg::new(row_sub_array_info.size, col_sub_array_info.remainder), factory)?;
                let base_col_index = col_sub_array_info.count * col_sub_array_info.size;
                for row in 0..row_sub_array_info.count {
                    let mut nets = vec![];
                    let base_row_index = row * row_sub_array_info.size;

                    for i in 0..col_sub_array_info.remainder {
                        nets.push(Self::bitline_pn(base_col_index + i));
                    }

                    for i in 0..col_sub_array_info.remainder {
                        nets.push(Self::bitline_bar_pn(base_col_index + i));
                    }

                    for i in 0..row_sub_array_info.size {
                        nets.push(Self::wordline_pn(base_row_index + i));
                    }

                    nets.push(Self::vdd_pn());
                    nets.push(Self::gnd_pn());

                    let inst_name = format_shr!("col_subarray_{}", row);
                    let instance = self.add_instance(inst_name, col_sub_array.clone())?;
                    self.connect_instance(instance, nets.into_iter())?;
                }

            }

            if row_sub_array_info.remainder >= 1 && col_sub_array_info.remainder >= 1 {
                let sub_array = self.add_module(BitcellArrayRecursiveArg::new(row_sub_array_info.remainder, col_sub_array_info.remainder), factory)?;
                let base_row_index = row_sub_array_info.count * row_sub_array_info.size;
                let base_col_index = col_sub_array_info.count * col_sub_array_info.size;

                let mut nets = vec![];

                for i in 0..col_sub_array_info.remainder {
                    nets.push(Self::bitline_pn(base_col_index + i));
                }

                for i in 0..col_sub_array_info.remainder {
                    nets.push(Self::bitline_bar_pn(base_col_index + i));
                }

                for i in 0..row_sub_array_info.remainder {
                    nets.push(Self::wordline_pn(base_row_index + i));
                }

                nets.push(Self::vdd_pn());
                nets.push(Self::gnd_pn());

                let instance = self.add_instance("subarray", sub_array.clone())?;
                self.connect_instance(instance, nets.into_iter())?;
            };
        } else {
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
        }

        Ok(())
    }

}

struct SubArrayInfo {   
    pub size: usize,
    pub count: usize,
    pub remainder: usize,
}

impl SubArrayInfo {
    fn new(size: usize, count: usize, remainder: usize) -> Self {
        Self { size, count, remainder }
    }

    fn from_size(size: usize) -> Self {
        let mut multiple = 1;
        let mut left = size - multiple * multiple;
        loop {
            let next_multiple = multiple + 1;
            let pow_mul2 = next_multiple * next_multiple;
            if pow_mul2 > size { 
                break;
            }
            // Update
            multiple = next_multiple;
            left = size - pow_mul2;
        }

        // Now size = multiple * multiple + remainder
        // But, left may bigger than multiple
        if multiple > left {
            return SubArrayInfo::new(multiple, multiple, left);
        }
        
        let remainder = left % multiple;
        let subsize = multiple + (left / multiple);
        let subcount = multiple;

        SubArrayInfo::new(subsize, subcount, remainder)
    }
}