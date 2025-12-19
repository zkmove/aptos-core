
<a id="0x1_zkhash"></a>

# Module `0x1::zkhash`



-  [Function `hash`](#0x1_zkhash_hash)
-  [Function `poseidon_hash`](#0x1_zkhash_poseidon_hash)


<pre><code></code></pre>



<a id="0x1_zkhash_hash"></a>

## Function `hash`

hashes two u128 values using the Poseidon hash function and returns a u256 hash.


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x1_hash">hash</a>(data1: u128, data2: u128): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="hash.md#0x1_hash">hash</a>(data1: u128, data2: u128): u256 {
    <a href="zkhash.md#0x1_zkhash_poseidon_hash">poseidon_hash</a>(data1, data2)
}
</code></pre>



</details>

<a id="0x1_zkhash_poseidon_hash"></a>

## Function `poseidon_hash`

Performs a Poseidon hash on two u128 values and returns a u256 hash.


<pre><code><b>fun</b> <a href="zkhash.md#0x1_zkhash_poseidon_hash">poseidon_hash</a>(data1: u128, data2: u128): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="zkhash.md#0x1_zkhash_poseidon_hash">poseidon_hash</a>(data1: u128, data2: u128): u256;
</code></pre>



</details>


[move-book]: https://aptos.dev/move/book/SUMMARY
