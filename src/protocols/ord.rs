use crate::BMapManager;
use crate::BpuError;
use crate::Include;
use crate::OrdData;
use crate::ParseConfig;
use crate::Result;
use crate::SplitConfig;
use crate::BPU;
use bsv::Script;
use bsv::{OpCodes, ScriptBit, Transaction};

pub struct Ord;

pub struct HandlerProps {
    pub data_obj: BPU,
    pub cell: Vec<crate::Cell>,
    pub tape: Option<crate::Tape>,
    pub tx: Option<Transaction>,
}

impl Ord {
    pub fn from_raw_tx(hex: &str) -> Result<BPU> {
        let config = ParseConfig {
            split: vec![
                SplitConfig {
                    include: Some(Include::Left),
                    token: crate::Token {
                        op: None,
                        ops: None,
                        b: None,
                        s: Some("|".to_string()),
                    },
                },
                SplitConfig {
                    include: Some(Include::Left),
                    token: crate::Token {
                        op: Some(106),
                        ops: None,
                        b: None,
                        s: None,
                    },
                },
            ],
            transform: None,
        };

        BPU::from_raw_tx(hex, config)
    }

    pub fn handler(tx: &Transaction, bmap: &mut BMapManager) -> Result<()> {
        let (_, ret) = Self::get_ord_script(tx)?;
        let op_if = ret.get_script_bit(0).unwrap();

        if let ScriptBit::If {
            code: _,
            pass,
            fail: _,
        } = op_if
        {
            let mut content_type = None;
            let mut data = None;

            let mut iter = pass.iter().enumerate().peekable();
            while let Some((_, bit)) = iter.next() {
                if bit.eq(&ScriptBit::OpCode(OpCodes::OP_1)) {
                    if let Some((_, next)) = iter.peek() {
                        let bytes = next.inner().unwrap();
                        let mime = String::from_utf8_lossy(&bytes).to_string();
                        content_type = Some(mime);
                        if data.is_some() {
                            break;
                        }
                    }
                } else if bit.eq(&ScriptBit::OpCode(OpCodes::OP_0)) {
                    if let Some((_, next)) = iter.peek() {
                        data = Some(next.to_vec());
                        if content_type.is_some() {
                            break;
                        }
                    }
                }
            }

            if let (Some(data), Some(content_type)) = (data, content_type) {
                bmap.ord.push(OrdData { data, content_type });
            }
        }

        Ok(())
    }

    pub fn get_ord_script(tx: &Transaction) -> Result<(usize, Script)> {
        let script = (0..tx.get_noutputs()).find_map(|e| {
            let out = tx.get_output(e).unwrap();
            let script = out.get_script_pub_key();

            if Self::script_checker(&script) {
                Some((
                    e,
                    Script::from_script_bits(vec![script.get_script_bit(6).unwrap()]),
                ))
            } else {
                None
            }
        });

        match script.is_some() {
            true => Ok(script.unwrap()),
            false => Err(BpuError::CustomError(
                "Invalid Ord tx. Script not found.".to_string(),
            )),
        }
    }

    pub fn script_checker(script: &Script) -> bool {
        if let Some((i, ord_script)) = script.iter().enumerate().find(|(_, e)| {
            matches!(
                e,
                ScriptBit::If {
                    code: _,
                    pass: _,
                    fail: _,
                }
            )
        }) {
            let prev_cell = script.get_script_bit(i - 1).unwrap();
            let buffer = ord_script.to_vec();

            prev_cell.eq(&ScriptBit::OpCode(OpCodes::OP_0))
                && buffer[1] == 3
                && &buffer[2..5] == b"ord"
        } else {
            false
        }
    }
}
