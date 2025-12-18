#[test_only]
module std::zkhash_tests {
    use std::zkhash;

    #[test]
    fun test_poseidon_hash() {
        let arg1 = 123u128;
        let arg2 = 45u128;
        let expected_output = 5396936627018144388256392133700981730161373533767880136248396757995540825894u256;
        let poseidon_result = zkhash::hash(arg1, arg2);
        assert!(poseidon_result == expected_output, 0);
    }
}
