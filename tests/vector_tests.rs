#![allow(non_snake_case)]
mod common;

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        str::FromStr,
    };

    use secp256k1::{Secp256k1, SecretKey, Scalar};

    #[cfg(feature = "receiving")]
    use silentpayments::receiving::Receiver;

    #[cfg(feature = "sending")]
    use silentpayments::sending::generate_multiple_recipient_pubkeys;

    use crate::common::{
        structs::TestData,
        utils::{
            self, calculate_tweak_data_for_recipient, decode_input_pub_keys, decode_outpoints,
            decode_outputs_to_check, decode_priv_keys, decode_recipients, get_a_sum_secret_keys,
            hash_outpoints, sender_calculate_partial_secret, verify_and_calculate_signatures, receiver_calculate_shared_secret
        },
    };

    const IS_TESTNET: bool = false;

    #[test]
    fn test_with_test_vectors() {
        let testdata = utils::read_file();

        for test in testdata {
            process_test_case(test);
        }
    }

    fn process_test_case(test_case: TestData) {
        let mut sending_outputs: HashSet<String> = HashSet::new();
        eprintln!("test.comment = {:?}", test_case.comment);

        #[cfg(feature = "sending")]
        for sendingtest in test_case.sending {
            let given = sendingtest.given;

            let expected = sendingtest.expected.outputs;

            let expected_output_addresses: HashSet<String> =
                expected.iter().map(|(x, _)| x.into()).collect();

            let input_priv_keys = decode_priv_keys(&given.input_priv_keys);

            let outpoints = decode_outpoints(&given.outpoints);

            let outpoints_hash = hash_outpoints(&outpoints);

            let silent_addresses = decode_recipients(&given.recipients);

            let a_sum = get_a_sum_secret_keys(&input_priv_keys);

            let partial_secret =
                sender_calculate_partial_secret(a_sum, outpoints_hash);

            let outputs =
                generate_multiple_recipient_pubkeys(silent_addresses, partial_secret).unwrap();

            for output_pubkeys in &outputs {
                for pubkey in output_pubkeys.1 {
                    // TODO check if this is always true
                    sending_outputs.insert(hex::encode(pubkey.serialize()));
                }
            }

            assert_eq!(sending_outputs, expected_output_addresses);
        }

        #[cfg(feature = "receiving")]
        for receivingtest in test_case.receiving {
            let given = receivingtest.given;
            let mut expected = receivingtest.expected;

            let receiving_outputs: HashSet<String> = given.outputs.iter().cloned().collect();

            #[cfg(feature = "sending")]
            // assert that the generated sending outputs are a subset
            // of the expected receiving outputs
            // i.e. all the generated outputs are present
            assert!(sending_outputs.is_subset(&receiving_outputs));

            let b_scan = SecretKey::from_str(&given.scan_priv_key).unwrap();
            let b_spend = SecretKey::from_str(&given.spend_priv_key).unwrap();
            let secp = Secp256k1::new();
            let B_spend = b_spend.public_key(&secp);
            let B_scan = b_scan.public_key(&secp);

            let mut sp_receiver = Receiver::new(0, B_scan, B_spend, IS_TESTNET).unwrap();

            let outputs_to_check = decode_outputs_to_check(&given.outputs);

            let outpoints = decode_outpoints(&given.outpoints);

            let input_pub_keys = decode_input_pub_keys(&given.input_pub_keys);

            for (_, label) in &given.labels {
                let label = label[..].try_into().unwrap();
                sp_receiver.add_label(label).unwrap();
            }

            let mut receiving_addresses: HashSet<String> = HashSet::new();
            // get receiving address for no label
            receiving_addresses.insert(sp_receiver.get_receiving_address());

            // get receiving addresses for every label
            let labels = sp_receiver.list_labels();
            for label in &labels {
                receiving_addresses
                    .insert(sp_receiver.get_receiving_address_for_label(label).unwrap());
            }

            let set1: HashSet<_> = receiving_addresses.iter().collect();
            let set2: HashSet<_> = expected.addresses.iter().collect();

            // check that the receiving addresses generated are equal
            // to the expected addresses
            assert_eq!(set1, set2);

            let tweak_data = calculate_tweak_data_for_recipient(&input_pub_keys, &outpoints);
            let shared_secret = receiver_calculate_shared_secret(tweak_data, b_scan);

            let scanned_outputs_received = sp_receiver
                .scan_transaction_with_labels(&shared_secret, outputs_to_check)
                .unwrap();

            let key_tweaks: Vec<Scalar> = scanned_outputs_received
                .into_iter()
                .flat_map(|(_, map)| {
                    let mut ret: Vec<Scalar> = vec![];
                    for l in map.into_values() {
                        ret.push(l);
                    }
                    ret
                })
                .collect();

            let mut res = verify_and_calculate_signatures(key_tweaks, b_spend).unwrap();

            res.sort_by_key(|output| output.pub_key.clone());
            expected
                .outputs
                .sort_by_key(|output| output.pub_key.clone());

            assert_eq!(res, expected.outputs);
        }
    }
}
