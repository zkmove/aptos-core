module std::proofs {
    use std::option::{Self, Option};

    public fun verify_proof(
        params: vector<u8>,
        vk_bytes: vector<u8>,
        circuit_info: vector<u8>,
        public_inputs: vector<u8>,
        proof: vector<u8>,
        kzg: u8,
        k_opt: Option<u32>,
    ): bool {
        let k_present = option::is_some(&k_opt);
        let k_value = if (k_present) {
            *option::borrow(&k_opt)
        } else {
            0u32
        };

        verify_halo2_proof(
            params,
            vk_bytes,
            circuit_info,
            public_inputs,
            proof,
            kzg,
            k_present,
            k_value
        )
    }

    native fun verify_halo2_proof(
        params: vector<u8>,
        vk_bytes: vector<u8>,
        circuit_info: vector<u8>,
        public_inputs: vector<u8>,
        proof: vector<u8>,
        kzg: u8,
        k_present: bool,
        k: u32,
    ): bool;
}
