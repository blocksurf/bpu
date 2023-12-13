use crate::Include;
use crate::ParseConfig;
use crate::Result;
use crate::SplitConfig;
use crate::BPU;

pub struct Bitcom;

// const opReturnSchema = [
//     {
//         su: [
//             { pubkey: 'string' },
//             { sign_position: 'string' },
//             { signature: 'string' },
//         ],
//         echo: [{ data: 'string' }, { to: 'string' }, { filename: 'string' }],
//         route: [
//             [
//                 {
//                     add: [
//                         { bitcom_address: 'string' },
//                         { route_matcher: 'string' },
//                         { endpoint_template: 'string' },
//                     ],
//                 },
//                 {
//                     enable: [{ path: 'string' }],
//                 },
//             ],
//         ],
//         useradd: [{ address: 'string' }],
//     },
// ]

impl Bitcom {
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
}
