use crate::dns::Record;

pub fn encode(_r: &Record) -> Option<Vec<u8>> {
    None
}

use crate::dns::registry::Registry;

pub fn register(reg: &mut Registry) {
    reg.register("ANY", encode);
}
