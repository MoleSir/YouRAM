use crate::YouRAMResult;
use super::{FunctionCharzPolicy, FunctionTransactionGenerator};

pub struct MatSPolicy;

impl FunctionCharzPolicy for MatSPolicy {
    fn generate_transactions(&self, charz: &mut FunctionTransactionGenerator) -> YouRAMResult<()> {
        let max_address = charz.transactions.max_address();
        let full_word = charz.transactions.max_word();
    
        // Write 0 from low address to high address
        for address in 0..=max_address {
            charz.add_write_transaction(address, 0);
        }
    
        // Read 0 and write 1 from low address to high address
        for address in 0..=max_address {
            charz.add_read_transaction(address);
            charz.add_write_transaction(address, full_word);
        }
    
        // Read 1 from high address to low address
        for address in (0..=max_address).rev() {
            charz.add_read_transaction(address);
        }
    
        Ok(())
    }
}