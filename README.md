# WARNING

This is an educational-only implementation of the proposed [Schnorr BIP](https://github.com/sipa/bips/blob/bip-schnorr/bip-schnorr.mediawiki) with the following features:

* Non efficient
* Non secure
* Non constant time
* A lot of heap allocation
* A lot of avoidable copy of memory
* Poor understanding of underlying math
* Lot of TODOs 

If you want to sign and verify with schnorr use [libsecp256k1]()


## Testing

That said, it looks it currently pass the [test vectors](https://github.com/sipa/bips/blob/bip-schnorr/bip-schnorr.mediawiki#test-vectors) of the Schnorr BIP

```
cargo test --release
```

## Benchmark

When I say non efficient I really mean it!
libsecp256k1 ECDSA which has comparable complexity AFAIK is about a 20 times faster!

```
$ cargo bench
...
Schnorr sign            time:   [1.6801 ms 1.6937 ms 1.7061 ms]

Schnorr verify          time:   [5.3759 ms 5.3898 ms 5.4065 ms]

Schnorr 100 Batch verify
                        time:   [189.05 ms 189.56 ms 190.34 ms]
...   
```

On My mid 2014 Mac Book Pro