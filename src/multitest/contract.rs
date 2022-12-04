use cosmwasm_std::{Addr};

pub struct BiddingContract(Addr);

impl BiddingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }
}

impl From<BiddingContract> for Addr {
    fn from(contract: BiddingContract) -> Self {
        contract.0
    }
}
