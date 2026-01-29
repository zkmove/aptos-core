
<a id="0x1_proofs"></a>

# Module `0x1::proofs`



-  [Function `verify_proof`](#0x1_proofs_verify_proof)
-  [Function `verify_halo2_proof`](#0x1_proofs_verify_halo2_proof)


<pre><code><b>use</b> <a href="option.md#0x1_option">0x1::option</a>;
</code></pre>



<a id="0x1_proofs_verify_proof"></a>

## Function `verify_proof`



<pre><code><b>public</b> <b>fun</b> <a href="proofs.md#0x1_proofs_verify_proof">verify_proof</a>(params: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, vk_bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, circuit_info: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, public_inputs: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, proof: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, kzg: u8, k_opt: <a href="option.md#0x1_option_Option">option::Option</a>&lt;u32&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="proofs.md#0x1_proofs_verify_proof">verify_proof</a>(
    params: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    vk_bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    circuit_info: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    public_inputs: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proof: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    kzg: u8,
    k_opt: Option&lt;u32&gt;,
): bool {
    <b>let</b> k_present = <a href="option.md#0x1_option_is_some">option::is_some</a>(&k_opt);
    <b>let</b> k_value = <b>if</b> (k_present) {
        *<a href="option.md#0x1_option_borrow">option::borrow</a>(&k_opt)
    } <b>else</b> {
        0u32
    };

    <a href="proofs.md#0x1_proofs_verify_halo2_proof">verify_halo2_proof</a>(
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
</code></pre>



</details>

<a id="0x1_proofs_verify_halo2_proof"></a>

## Function `verify_halo2_proof`



<pre><code><b>fun</b> <a href="proofs.md#0x1_proofs_verify_halo2_proof">verify_halo2_proof</a>(params: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, vk_bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, circuit_info: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, public_inputs: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, proof: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;, kzg: u8, k_present: bool, k: u32): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="proofs.md#0x1_proofs_verify_halo2_proof">verify_halo2_proof</a>(
    params: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    vk_bytes: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    circuit_info: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    public_inputs: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proof: <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    kzg: u8,
    k_present: bool,
    k: u32,
): bool;
</code></pre>



</details>


[move-book]: https://aptos.dev/move/book/SUMMARY
