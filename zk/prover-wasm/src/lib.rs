use wasm_bindgen::prelude::*;
use halo2_proofs::{
    poly::kzg::{
        commitment::{KZGCommitmentScheme, ParamsKZG},
        multiopen::ProverSHPLONK,
    },
    plonk::{create_proof, keygen_pk, keygen_vk},
    transcript::{Blake2bWrite, Challenge255, TranscriptWriterBuffer},
    circuit::Value,
};
use halo2curves::bn256::{Bn256, Fr, G1Affine};
use bastion_circuit::BastionCircuit;
use rand::rngs::OsRng;

#[wasm_bindgen]
pub struct ZkProver {
    params: ParamsKZG<Bn256>,
}

#[wasm_bindgen]
impl ZkProver {
    #[wasm_bindgen(constructor)]
    pub fn new(k: u32) -> Self {
        let params: ParamsKZG<Bn256> = ParamsKZG::new(k);
        Self { params }
    }

    #[wasm_bindgen]
    pub fn generate_proof(
        &self,
        amount: u64,
        side: u64,
        price: u64,
        nonce: u64,
    ) -> Result<Vec<u8>, JsValue> {
        let amount_fr = Fr::from(amount);
        let side_fr = Fr::from(side);
        let price_fr = Fr::from(price);
        let nonce_fr = Fr::from(nonce);
        let commitment_fr = amount_fr + side_fr + price_fr + nonce_fr;

        let circuit = BastionCircuit {
            amount: Value::known(amount_fr),
            side: Value::known(side_fr),
            price: Value::known(price_fr),
            nonce: Value::known(nonce_fr),
            commitment: Value::known(commitment_fr),
        };

        let vk = keygen_vk(&self.params, &circuit)
            .map_err(|_| JsValue::from_str("Failed to generate vk"))?;
        let pk = keygen_pk(&self.params, vk, &circuit)
            .map_err(|_| JsValue::from_str("Failed to generate pk"))?;

        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
        
        create_proof::<KZGCommitmentScheme<Bn256>, ProverSHPLONK<'_, Bn256>, _, _, _, _>(
            &self.params,
            &pk,
            &[circuit],
            &[&[&[commitment_fr]]],
            OsRng,
            &mut transcript,
        ).map_err(|_| JsValue::from_str("Failed to create proof"))?;

        Ok(transcript.finalize())
    }
}
