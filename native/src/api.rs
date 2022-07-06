#![allow(unused_imports)]
extern crate bellman_ce;
extern crate rand;

mod bit_iterator;
mod hasher;
use crate::api::hasher::BabyPedersenHasher;

use bellman_ce::groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof, Proof,
};
use bellman_ce::pairing::bn256::{Bn256, Fr};
use bellman_ce::pairing::ff::{Field, PrimeField, PrimeFieldRepr};
use bellman_ce::{Circuit, ConstraintSystem, SynthesisError};
use rand::{thread_rng, Rng};
use sapling_crypto_ce::alt_babyjubjub::AltJubjubBn256;
use sapling_crypto_ce::circuit::num::{AllocatedNum, Num};
use sapling_crypto_ce::circuit::test::TestConstraintSystem;
use sapling_crypto_ce::circuit::{
    baby_eddsa, blake2s, boolean, ecc, float_point, multipack, num, pedersen_hash, sha256,
    Assignment,
};
use sapling_crypto_ce::jubjub::{JubjubEngine, JubjubParams};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct PedersenDemo<'a, E: JubjubEngine> {
    pub params: &'a E::Params,
    pub hash: Option<E::Fr>,
    pub preimage: Option<E::Fr>,
}
impl<'a, E: JubjubEngine> Circuit<E> for PedersenDemo<'a, E> {
    fn synthesize<CS: ConstraintSystem<E>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let hash = AllocatedNum::alloc(cs.namespace(|| "hash"), || {
            let hash_value = self.hash;
            Ok(*hash_value.get()?)
        })?;
        hash.inputize(cs.namespace(|| "hash input"))?;

        let mut hash_calculated = AllocatedNum::alloc(cs.namespace(|| "preimage"), || {
            let preimage_value = self.preimage;
            Ok(*preimage_value.get()?)
        })?;

        for i in 0..5 {
            let preimage_bits = hash_calculated
                .into_bits_le(cs.namespace(|| format!("preimage into bits {}", i)))?;

            hash_calculated = pedersen_hash::pedersen_hash(
                cs.namespace(|| format!("hash calculated {}", i)),
                pedersen_hash::Personalization::NoteCommitment,
                &preimage_bits,
                self.params,
            )?
            .get_x()
            .clone();
        }

        cs.enforce(
            || "add constraint between input and pedersen hash output",
            |lc| lc + hash_calculated.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + hash.get_variable(),
        );
        Ok(())
    }
}

// #[test]
pub fn test_pedersen_proof() -> bool {
    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut thread_rng();
    let pedersen_params = &AltJubjubBn256::new();

    let preimage = rng.gen();
    let hasher = BabyPedersenHasher::default();
    let mut hash = preimage;
    for _ in 0..5 {
        hash = hasher.hash(hash);
    }
    println!("Preimage: {}", preimage.clone());
    println!("Hash: {}", hash.clone());

    println!("Creating parameters...");
    let params = {
        let c = PedersenDemo::<Bn256> {
            params: pedersen_params,
            hash: None,
            preimage: None,
        };
        generate_random_parameters(c, rng).unwrap()
    };

    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);

    println!("Checking constraints...");
    let c = PedersenDemo::<Bn256> {
        params: pedersen_params,
        hash: Some(hash.clone()),
        preimage: Some(preimage.clone()),
    };
    let mut cs = TestConstraintSystem::<Bn256>::new();
    c.synthesize(&mut cs).unwrap();
    println!("Unconstrained: {}", cs.find_unconstrained());
    let err = cs.which_is_unsatisfied();
    if err.is_some() {
        panic!("ERROR satisfying in: {}", err.unwrap());
    }

    println!("Creating proofs...");
    let c = PedersenDemo::<Bn256> {
        params: pedersen_params,
        hash: Some(hash.clone()),
        preimage: Some(preimage.clone()),
    };
    let stopwatch = std::time::Instant::now();
    let proof = create_random_proof(c, &params, rng).unwrap();
    println!("Proof time: {}ms", stopwatch.elapsed().as_millis());

    let result = verify_proof(&pvk, &proof, &[hash]).unwrap();

    return result;
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn get_counter() -> u64 {
    COUNTER.load(Ordering::SeqCst)
}

pub fn increment() -> u64 {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    COUNTER.load(Ordering::SeqCst)
}

pub fn decrement() -> u64 {
    COUNTER.fetch_sub(1, Ordering::SeqCst);
    COUNTER.load(Ordering::SeqCst)
}
