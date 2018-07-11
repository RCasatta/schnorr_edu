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
libsecp256k1 ECDSA which has comparable complexity AFAIK is about a thousand times faster!

```
$ cargo bench
... 
Schnorr verify          time:   [82.144 ms 84.999 ms 85.713 ms]                        
Schnorr sign            time:   [43.060 ms 43.153 ms 43.176 ms]                       
...   
```

On My mid 2014 Mac Book Pro