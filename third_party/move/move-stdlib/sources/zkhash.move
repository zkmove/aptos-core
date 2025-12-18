module std::zkhash {
    /// hashes two u128 values using the Poseidon hash function and returns a u256 hash.
    public fun hash(data1: u128, data2: u128): u256 {
        poseidon_hash(data1, data2)
    }

    /// Performs a Poseidon hash on two u128 values and returns a u256 hash.
    native fun poseidon_hash(data1: u128, data2: u128): u256;
}
