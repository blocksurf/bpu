use crate::components::*;
use bsv::{OpCodes, P2PKHAddress, PublicKey, ScriptBit, Transaction, TxIn, TxOut};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
/// Bitcoin Processing Unit
pub struct BPU {
    pub r#out: Vec<IO>,
    pub r#in: Vec<IO>,
    pub tx: Tx,
    pub lock: Option<u32>,
}

impl std::fmt::Display for BPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub fn maybe_transform(
    cell: Cell,
    script_bit: &ScriptBit,
    transform: Option<fn(Cell, &ScriptBit) -> Cell>,
) -> Cell {
    match transform {
        Some(f) => f(cell, script_bit),
        None => cell,
    }
}

impl BPU {
    pub fn from_raw_tx(raw_tx: &str, parse_config: ParseConfig) -> Result<BPU> {
        let gene = Transaction::from_hex(raw_tx)?;
        Self::collect(&gene, parse_config) //settings, transform)
    }

    /// Collects script bits from each Input/Output script
    pub fn collect(tx: &Transaction, parse_config: ParseConfig) -> Result<BPU> {
        let mut results = BPU {
            tx: Tx {
                h: tx.get_id_hex().ok(),
                ..Default::default()
            },
            r#in: vec![],
            r#out: vec![],
            lock: Some(tx.get_n_locktime()),
        };

        let inputs: Vec<TxIn> = (0..tx.get_ninputs())
            .map(|e| tx.get_input(e).unwrap())
            .collect();

        let outputs: Vec<TxOut> = (0..tx.get_noutputs())
            .map(|e| tx.get_output(e).unwrap())
            .collect();

        let settings = parse_config.split;

        for (i, input) in inputs.iter().enumerate() {
            let mut limb = IO::new(i);
            let mut counter = IndexCounter::new(i);
            let mut cell: Vec<Cell> = vec![];
            let mut address: Option<String> = None;

            let script = if let Ok(s) = input.get_finalised_script() {
                s
            } else {
                input.get_unlocking_script()
            };

            if let Some(ScriptBit::Push(buf)) = script.get_script_bit(1) {
                // Public Key
                if buf.len() == 33 && (buf[0] == 2 || buf[0] == 3) {
                    if let Ok(v) = PublicKey::from_bytes(&buf) {
                        if let Ok(v) = v.to_p2pkh_address() {
                            if let Ok(v) = v.to_string() {
                                address = Some(v)
                            }
                        }
                    }
                } else if buf.len() == 20 {
                    // P2PKH
                    if let Ok(v) = P2PKHAddress::from_pubkey_hash(&buf) {
                        if let Ok(v) = v.to_string() {
                            address = Some(v)
                        }
                    }
                }
            }

            let bits: Vec<ScriptBit> = script
                .into_iter()
                .flat_map(|e| flatten_script_bits(&[e]))
                .collect();

            for (i, chunk) in bits.iter().enumerate() {
                counter.chunk_index = i;

                Self::extract_cells(
                    &mut limb,
                    &mut cell,
                    &mut counter,
                    chunk,
                    &settings,
                    parse_config.transform,
                );
            }

            if !cell.is_empty() {
                limb.tape.push(Tape {
                    cell,
                    i: counter.tape_index,
                });
            }

            let sender = SendRecv {
                h: Some(input.get_prev_tx_id_hex(None)),
                i: input.get_vout(),
                a: Some(address.unwrap_or("false".to_string())),
                v: None,
            };

            limb.e = Some(sender);

            limb.seq = Some(input.get_sequence());

            results.r#in.push(limb);
        }

        for (i, output) in outputs.iter().enumerate() {
            let mut limb = IO::new(i);
            let mut counter = IndexCounter::new(i);
            let mut cell: Vec<Cell> = vec![];
            let mut address: Option<String> = None;

            let script = output.get_script_pub_key();

            if let Some(ScriptBit::Push(buf)) = script.get_script_bit(2) {
                // P2PKH Script
                if let Ok(v) = P2PKHAddress::from_pubkey_hash(&buf) {
                    if let Ok(v) = v.to_string() {
                        address = Some(v)
                    }
                }
            }

            let bits: Vec<ScriptBit> = script
                .into_iter()
                .flat_map(|e| flatten_script_bits(&[e]))
                .collect();

            for (i, chunk) in bits.iter().enumerate() {
                counter.chunk_index = i;

                Self::extract_cells(
                    &mut limb,
                    &mut cell,
                    &mut counter,
                    chunk,
                    &settings,
                    parse_config.transform,
                );
            }
            if !cell.is_empty() {
                limb.tape.push(Tape {
                    cell,
                    i: counter.tape_index,
                });
            }

            let sender = SendRecv {
                h: None,
                i: i as u32,
                a: Some(address.unwrap_or("false".to_string())),
                v: Some(output.get_satoshis()),
            };

            limb.e = Some(sender);

            results.r#out.push(limb);
        }

        Ok(results)
    }

    /// Recursively maps each script chunk (Cell) to the Input/Output Tape
    pub fn extract_cells(
        xput: &mut IO,
        cell: &mut Vec<Cell>,
        counter: &mut IndexCounter,
        chunk: &ScriptBit,
        settings: &[SplitConfig],
        _transform: Option<fn(Cell, &ScriptBit) -> Cell>,
    ) {
        let mut is_splitter = false;
        let mut splitter: Option<Include> = None;

        match chunk {
            ScriptBit::OpCode(op_code) => {
                let op = Some(*op_code as u8);
                let ops = Some(op_code.to_string());

                for setting in settings {
                    if setting.token.op == op || setting.token.ops == ops {
                        splitter = setting.include.to_owned();
                        is_splitter = true;
                    }
                }

                if is_splitter {
                    let splitter = splitter.unwrap();

                    match splitter {
                        Include::Left => {
                            let item: Cell = maybe_transform(
                                Cell {
                                    op,
                                    ops,
                                    ii: counter.chunk_index,
                                    i: counter.cell_index,
                                    ..Default::default()
                                },
                                chunk,
                                _transform,
                            );
                            counter.cell_index += 1;

                            cell.push(item);

                            xput.tape.push(Tape {
                                cell: cell.to_owned(),
                                i: counter.tape_index,
                            });

                            counter.tape_index += 1;

                            cell.clear();
                            counter.cell_index = 0;
                        }
                        Include::Right => {
                            xput.tape.push(Tape {
                                cell: cell.to_vec(),
                                i: counter.tape_index,
                            });
                            counter.tape_index += 1;

                            let item: Cell = maybe_transform(
                                Cell {
                                    op,
                                    ops,
                                    ii: counter.chunk_index,
                                    i: counter.cell_index,
                                    ..Default::default()
                                },
                                chunk,
                                _transform,
                            );

                            *cell = vec![item];
                            counter.cell_index = 1;
                        }
                        Include::Center => {
                            xput.tape.push(Tape {
                                cell: cell.to_vec(),
                                i: counter.tape_index,
                            });
                            counter.tape_index += 1;

                            let item: Cell = maybe_transform(
                                Cell {
                                    op,
                                    ops,
                                    ii: counter.chunk_index,
                                    i: 0,
                                    ..Default::default()
                                },
                                chunk,
                                _transform,
                            );

                            xput.tape.push(Tape {
                                cell: vec![item],
                                i: counter.tape_index,
                            });

                            cell.clear();
                            counter.cell_index = 0;
                        }
                    }
                } else {
                    let item: Cell = maybe_transform(
                        Cell {
                            op,
                            ops,
                            ii: counter.chunk_index,
                            i: counter.cell_index,
                            ..Default::default()
                        },
                        chunk,
                        _transform,
                    );

                    cell.push(item);
                    counter.cell_index += 1;
                }
            }
            ScriptBit::If { code, pass, fail } => {
                Self::extract_cells(
                    xput,
                    cell,
                    counter,
                    &ScriptBit::OpCode(code.to_owned()),
                    settings,
                    _transform,
                );

                for bit in pass {
                    Self::extract_cells(xput, cell, counter, bit, settings, _transform)
                }

                if let Some(fail_script) = fail {
                    for bit in fail_script {
                        Self::extract_cells(xput, cell, counter, bit, settings, _transform)
                    }
                }
            }
            _ => {
                if let Some(bytes) = chunk.inner() {
                    let string = String::from_utf8_lossy(&bytes).to_string();

                    for setting in settings {
                        if setting.token.b.as_ref() == Some(&bytes)
                            || setting.token.s.as_ref() == Some(&string)
                        {
                            splitter = setting.include.to_owned();
                            is_splitter = true;
                        }
                    }

                    if is_splitter {
                        let splitter = splitter.unwrap();
                        match splitter {
                            Include::Left => {
                                let item: Cell = maybe_transform(
                                    Cell {
                                        b: Some(bytes),
                                        s: Some(string),
                                        ii: counter.chunk_index,
                                        i: counter.cell_index,
                                        ..Default::default()
                                    },
                                    chunk,
                                    _transform,
                                );

                                cell.push(item);
                                counter.cell_index += 1;

                                xput.tape.push(Tape {
                                    cell: cell.to_vec(),
                                    i: counter.tape_index,
                                });

                                counter.tape_index += 1;

                                cell.clear();
                                counter.cell_index = 0;
                            }
                            Include::Right => {
                                xput.tape.push(Tape {
                                    cell: cell.to_vec(),
                                    i: counter.tape_index,
                                });
                                counter.tape_index += 1;

                                let item: Cell = maybe_transform(
                                    Cell {
                                        b: Some(bytes),
                                        s: Some(string),
                                        ii: counter.chunk_index,
                                        i: counter.cell_index,
                                        ..Default::default()
                                    },
                                    chunk,
                                    _transform,
                                );

                                *cell = vec![item];
                                counter.cell_index = 1;
                            }
                            Include::Center => {
                                xput.tape.push(Tape {
                                    cell: cell.to_vec(),
                                    i: counter.tape_index,
                                });
                                counter.tape_index += 1;

                                let item: Cell = maybe_transform(
                                    Cell {
                                        b: Some(bytes),
                                        s: Some(string),
                                        ii: counter.chunk_index,
                                        i: 0,
                                        ..Default::default()
                                    },
                                    chunk,
                                    _transform,
                                );

                                xput.tape.push(Tape {
                                    cell: vec![item],
                                    i: counter.tape_index,
                                });

                                cell.clear();
                                counter.cell_index = 0;
                            }
                        }
                    } else {
                        let item: Cell = maybe_transform(
                            Cell {
                                b: Some(bytes.to_vec()),
                                s: Some(string.to_string()),
                                ii: counter.chunk_index,
                                i: counter.cell_index,
                                ..Default::default()
                            },
                            chunk,
                            _transform,
                        );
                        cell.push(item);
                        counter.cell_index += 1;
                    }
                }
            }
        }
    }
}

/// Returns a flattened Vec<ScriptBit>
pub fn flatten_script_bits(script_bits: &[ScriptBit]) -> Vec<ScriptBit> {
    let mut flat_map = Vec::new();

    for bit in script_bits {
        match bit {
            ScriptBit::If { code, pass, fail } => {
                flat_map.push(ScriptBit::OpCode(code.to_owned()));

                let passed = flatten_script_bits(pass);
                flat_map.extend(passed);

                if let Some(fail_bits) = fail {
                    flat_map.push(ScriptBit::OpCode(OpCodes::OP_ELSE));
                    let failed = flatten_script_bits(fail_bits);
                    flat_map.extend(failed);
                }
                flat_map.push(ScriptBit::OpCode(OpCodes::OP_ENDIF));
            }
            _ => {
                flat_map.push(bit.to_owned());
            }
        }
    }

    flat_map
}
