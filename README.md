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
libsecp256k1 Schnorr (pre-standard AFAIK) is about a 16 times faster!

```
$ cargo bench | grep time # sorted manually from faster to slower

BigUint rand            time:   [27.142 ns 27.174 ns 27.208 ns]
BigUint cmp             time:   [36.984 ns 36.997 ns 37.015 ns]
BigUint checked_sub     time:   [59.581 ns 59.591 ns 59.606 ns]
ScalarP mul one         time:   [97.895 ns 97.904 ns 97.913 ns]
BigUint mul assign      time:   [146.50 ns 146.53 ns 146.57 ns]
BigUint mul             time:   [152.18 ns 152.20 ns 152.22 ns]
BigUint rem             time:   [184.59 ns 184.67 ns 184.75 ns]
BigUint modpow 2        time:   [1.7779 us 1.7788 us 1.7797 us]
EC Jac Point double     time:   [8.1594 us 8.1608 us 8.1623 us]
EC mixed Point adding   time:   [12.059 us 12.061 us 12.064 us]
EC Jac Point add        time:   [16.682 us 16.685 us 16.688 us]
ScalarP modpow one      time:   [53.067 us 53.102 us 53.133 us]
Schnorr libsecp sign    time:   [91.489 us 91.501 us 91.513 us]
ScalarP inv             time:   [146.66 us 146.70 us 146.74 us]
EC Point adding         time:   [150.68 us 150.74 us 150.79 us]
BigUint div             time:   [184.67 ns 184.74 ns 184.81 ns]
BigUint modpow          time:   [229.81 us 231.16 us 232.63 us]
G JPoint mul            time:   [368.85 us 368.91 us 368.97 us]
*Schnorr sign            time:   [1.4849 ms 1.4851 ms 1.4852 ms]*
EC Jac P mul 5naf       time:   [2.8732 ms 2.8745 ms 2.8758 ms]
EC Jac P mul 6naf       time:   [2.9053 ms 2.9062 ms 2.9071 ms]
EC Jac P mul 4naf       time:   [2.9478 ms 2.9493 ms 2.9511 ms]
EC Jac P mul 3naf       time:   [3.1364 ms 3.1382 ms 3.1400 ms]
EC Jac P mul 2naf       time:   [3.4741 ms 3.4789 ms 3.4844 ms]
*Schnorr verify          time:   [3.6886 ms 3.6900 ms 3.6914 ms]*
EC Jac Point mul        time:   [4.1507 ms 4.1546 ms 4.1584 ms]
EC JPoint kP+lQ shamir  time:   [5.2714 ms 5.2748 ms 5.2786 ms]
EC JPoint kP+lQ         time:   [8.3215 ms 8.3273 ms 8.3332 ms]
Schnorr 100 B verify    time:   [153.72 ms 153.74 ms 153.76 ms]

```

Run on my threadripper 1950X