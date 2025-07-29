module std::zkhash {
    public fun hash(data1: u128, data2: u128): u256 {
        poseidon_hash(data1, data2)
    }

    native fun poseidon_hash(data1: u128, data2: u128): u256;
}
