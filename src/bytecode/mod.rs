pub mod encoder;
pub mod decoder;

pub use encoder::encode;
pub use decoder::decode;

pub const MAGIC: u32 = 0x56424300;
pub const VERSION: u32 = 1;
