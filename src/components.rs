use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BpuError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("BSV lib error: {0}")]
    BsvError(#[from] bsv::BSVErrors),

    #[error("Custom error: {0}")]
    CustomError(String),
}

pub type Result<T> = std::result::Result<T, BpuError>;

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Tx {
    pub h: Option<String>, // txid
    pub r: Option<String>, // raw hex
}

#[serde_as]
#[derive(Debug, Serialize, Default)]
/// Extends BPU
pub struct BobTx {
    /// Block Info (index, hash, timestamp)
    pub blk: Option<Block>,
    pub mem: u64,
    pub fields: HashMap<String, String>,
    pub r#out: Vec<IO>,
    pub r#in: Vec<IO>,
    pub tx: Tx,
    pub lock: Option<u32>,
}
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Clone, Default)]
pub struct OrdData {
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>, // base64
    pub content_type: String,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Clone, Default)]
pub struct BMapManager {
    pub timestamp: u64,
    pub ord: Vec<OrdData>,
    // pub B: Option<Vec<B>>,
    // pub AIP: Option<Vec<AIP>>,
    // pub MAP: Option<Vec<MAP>>,
    // pub BAP: Option<Vec<BAP>>,
    // pub PSP: Option<Vec<PSP>>,
    // pub _21E8: Option<Vec<_21E8>>,
    // pub BOOST: Option<Vec<BOOST>>,
    // pub BITCOM: Option<Vec<BITCOM>>,
    // pub BITPIC: Option<Vec<BITPIC>>,
    // pub BITKEY: Option<Vec<BITKEY>>,
    // pub METANET: Option<Vec<MetaNet>>,
    // pub SYMRE: Option<Vec<SYMRE>>,
    // pub RON: Option<Vec<RON>>,
    // pub HAIP: Option<Vec<HAIP>>,
    // pub BITCOM_HASHED: Option<Vec<BITCOM_HASHED>>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Default)]
pub struct Block {
    i: u32,            // index
    t: u32,            // time
    h: Option<String>, // hash
}

#[derive(Debug, Serialize, Clone)]
/// - With `Include::Left`, you can merge the delimiter to the left side of the split arrays
/// - With `Include::Right`, you can merge the delimiter to the left side of the split arrays
/// - With `Include::Center`,  you can create a new standalone cell which contains the delimiter
pub enum Include {
    Left,
    Right,
    Center,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Clone, Default)]
/// The Cell we want to split (w/o index)
pub struct Token {
    pub op: Option<u8>,
    pub ops: Option<String>,
    #[serde_as(as = "Option<Base64>")]
    pub b: Option<Vec<u8>>,
    pub s: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SplitConfig {
    /// - With `Include::Left`, you can merge the delimiter to the left side of the split arrays
    /// - With `Include::Right`, you can merge the delimiter to the left side of the split arrays
    /// - With `Include::Center`,  you can create a new standalone cell which contains the delimiter
    pub include: Option<Include>,
    /// The tokens we wish to split off from the main sequence of ScriptBits
    pub token: Token,
}

#[derive(Debug, Clone, Default)]
pub struct ParseConfig {
    pub split: Vec<SplitConfig>,
    pub transform: Option<fn(Cell, &bsv::ScriptBit) -> Cell>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tape {
    pub cell: Vec<Cell>,
    pub i: usize,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Cell {
    pub op: Option<u8>,
    pub ops: Option<String>,
    #[serde_as(as = "Option<Base64>")]
    pub b: Option<Vec<u8>>,
    pub s: Option<String>,
    pub ii: usize,
    pub i: usize,
    pub h: Option<String>,
    pub f: Option<String>,
    pub ls: Option<String>,
    pub lh: Option<String>,
    pub lf: Option<String>,
    pub lb: Option<String>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SendRecv {
    pub h: Option<String>,
    pub i: u32,
    pub v: Option<u64>,
    pub a: Option<String>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
/// Transaction Input/Output Object
pub struct IO {
    pub i: usize,
    pub tape: Vec<Tape>,
    pub e: Option<SendRecv>,
    pub seq: Option<u32>,
}

impl IO {
    pub fn new(tx_index: usize) -> Self {
        Self {
            i: tx_index,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct IndexCounter {
    pub outer_index: usize,
    pub tape_index: usize,
    pub cell_index: usize,
    pub chunk_index: usize,
}

impl IndexCounter {
    pub fn new(tx_index: usize) -> Self {
        Self {
            outer_index: tx_index,
            ..Default::default()
        }
    }
}
