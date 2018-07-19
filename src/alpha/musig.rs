
// https://blockstream.com/2018/01/23/musig-key-aggregation-schnorr-signatures.html
/*
Call L = H(X1,X2,…)
Call X the sum of all H(L,Xi)Xi
Each signer chooses a random nonce ri, and shares Ri = riG with the other signers
Call R the sum of the Ri points
Each signer computes si = ri + H(X,R,m)H(L,Xi)xi
The final signature is (R,s) where s is the sum of the si values
Verification again satisfies sG = R + H(X,R,m)X
*/

// The following function is for test only,
// obviously the real algo does not require all private keys in the same function
// but it's instead interactive
#[allow(non_snake_case)]
pub fn musig(msg : &Msg, sec_keys:  &Vec<ScalarN>, is_new: bool) -> (Point, Signature, Point, bool) {
    let total_signers = sec_keys.len();
    assert!(total_signers >1);

    let pub_keys : Vec<Point> = sec_keys.iter()
        .map(|sec_key| CONTEXT.G.borrow().mul(sec_key) )
        .collect();

    let pub_keys_bytes : Vec<[u8;33]> = pub_keys.iter()
        .map(|pub_key| pub_key.as_bytes())
        .collect();

    let mut all_pub_keys_bytes = Vec::new();
    pub_keys_bytes.iter()
        .for_each(|el| all_pub_keys_bytes.extend(&el[..]));

    // Call L = H(X1,X2,…)
    let L = concat_and_hash(&all_pub_keys_bytes, &vec![], &vec![]);
    //Call X the sum of all H(L,Xi)Xi
    let mut X : Option<Point>= None;
    for pub_key in pub_keys.iter() {
        X = point_add(
            X,
            Some(pub_key.clone().mul( &concat_and_hash(&L.to_32_bytes(), &pub_key.as_bytes(), &vec![]))));
    }
    let X = X.unwrap();

    let mut ris = Vec::new();
    let mut Ris = Vec::new();
    let mut R : Option<Point> = None;
    // Each signer chooses a random nonce ri, and shares Ri = riG with the other signers
    // Call R the sum of the Ri points
    for _ in 0..total_signers {
        let ri : ScalarN = thread_rng().gen();
        let Ri = CONTEXT.G.clone().mul(&ri);
        R = point_add(R, Some(Ri.clone()) );
        Ris.push(Ri);
        ris.push(ri);
    }
    let R = R.unwrap();


    // Each signer computes si = ri + H(X,R,m)H(L,Xi)xi
    // Let e = int(hash(bytes(r) || bytes(P) || m)) mod n.    // bip schnorr
    let mut s = ScalarN(BigUint::zero());
    let X_bytes = X.as_bytes();
    for i in 0..total_signers {
        let e = match is_new {
            false => concat_and_hash( &X_bytes , &R.as_bytes(), msg ),
            true => concat_and_hash( &R.x.to_32_bytes(), &X_bytes ,  msg ),
        };

        let si = ris[i].clone().add(
            e.mul( &concat_and_hash(&L.to_32_bytes(), &pub_keys[i].as_bytes(), &vec![] ))
                .mul( &sec_keys[i] )
        );
        s = s + si;
    }

    // TODO how to create R with the right convention?

    (X, Signature::new(R.x.clone(), s), R.clone(), R.y.is_jacobi())
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_musig() {
        let mut sec_keys = Vec::new();
        for _ in 0..5 {
            sec_keys.push(thread_rng().gen::<ScalarN>());
        }
        let msg = [0u8; 32];
        let (pub_key, signature, r, _) = musig(&msg, &sec_keys, false);

        //let result = schnorr_verify(&msg, &combined_pub_key, &signature);

        //Verification again satisfies sG = R + H(X,R,m)X
        let left = CONTEXT.G.clone().mul(&signature.s);
        let right = r.clone().add(pub_key.clone().mul(&concat_and_hash(&pub_key.as_bytes(), &r.as_bytes(), &msg)));
        assert_eq!(left, right);

        let (pub_key, signature, _, is_y_jacoby) = musig(&msg, &sec_keys, true);
        if is_y_jacoby {  // until work on musig I can verify onlu if R.y is jacobi
            let result = schnorr_verify(&msg, &pub_key, &signature);
            assert!(result);
        }
    }
}