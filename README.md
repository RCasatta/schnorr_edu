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
Schnorr sign            time:   [2.6110 ms 2.6584 ms 2.6702 ms]
                        change: [+0.1630% +1.4299% +2.7001%] (p = 0.24 > 0.05)
                        No change in performance detected.

Schnorr verify          time:   [9.3913 ms 9.3982 ms 9.4261 ms]
                        change: [-3.8001% -2.4223% -1.0098%] (p = 0.12 > 0.05)
                        No change in performance detected.

Schnorr 100 Batch verify
                        time:   [332.42 ms 332.49 ms 332.78 ms]
                        change: [-1.8245% -0.9584% -0.0778%] (p = 0.17 > 0.05)
                        No change in performance detected.
...   
```

On My mid 2014 Mac Book Pro