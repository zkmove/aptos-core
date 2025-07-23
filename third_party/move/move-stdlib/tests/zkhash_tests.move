#[test_only]
module std::zkhash_tests {
    use std::zkhash;

    #[test]
    fun test_fake_hash() {
        let arg1 = 123u128;
        let arg2 = 45u128;
        let expected_output = 15312706511442230855851857334429569515643u256;
        let fake_result = zkhash::fake_hash(arg1, arg2);
        assert!(fake_result == expected_output, 0);
    }
}
