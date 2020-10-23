use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use identity_crypto::{Ed25519, KeyGen, KeyGenerator};
// use multihash::Blake2b256;
use identity_core::{common::OneOrMany, did::DIDDocument, did::DIDDocumentBuilder, did::DID};
#[wasm_bindgen]
#[derive(Serialize)]
pub struct Core {}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen]
    pub fn create_did(id: String) -> Result<String, JsValue> {
        console_error_panic_hook::set_once();

        let did = DID {
            method_name: "iota".into(),
            id_segments: vec![id.into()],
            ..Default::default()
        }
        .init()
        .unwrap();
        Ok(did.to_string())
    }
    // #[wasm_bindgen(js_name = "createDIDDocument")]
    // pub fn create_did_document(did: &str) -> Result<String, JsValue> {
    //     console_error_panic_hook::set_once();

    //     let doc: DIDDocument = DIDDocumentBuilder::default()
    //         .context(OneOrMany::One(DID::BASE_CONTEXT.into()))
    //         .id(DID::parse_from_str(did).unwrap())
    //         .build()
    //         .unwrap();

    //     Ok(doc.to_string())
    // }

    #[wasm_bindgen(js_name = "createGenerateKeypair")]
    pub fn create_generate_keypair() -> Result<String, JsValue> {
        console_error_panic_hook::set_once();
        //     let keypair = Ed25519::generate(&Ed25519, Default::default())?;

        let keypair = KeyGen::generate(&Ed25519, KeyGenerator::default()).unwrap();
        let bs58_auth_key = bs58::encode(keypair.public()).into_string();
        // console.log("bs58_auth_key: {:?}", bs58_auth_key);
        Ok(bs58_auth_key.into())
    }

    // #[wasm_bindgen(js_name = "createDocument")]
    // pub fn create_document(auth_key: String) -> Result<DIDDocument> {
    //     let did: DID = create_method_id(&auth_key, Some("Main"), None)?;
    //     let key: DID = format!("{}#key-1", did).parse()?;
    
    //     let public_key: PublicKey = PublicKeyBuilder::default()
    //         .id(key.clone())
    //         .controller(did.clone())
    //         .key_type(KeyType::Ed25519VerificationKey2018)
    //         .key_data(KeyData::PublicKeyBase58(auth_key))
    //         .build()
    //         .unwrap();
    
    //     let doc: DIDDocument = DIDDocumentBuilder::default()
    //         .context(OneOrMany::One(DID::BASE_CONTEXT.into()))
    //         .id(did)
    //         .public_keys(vec![public_key])
    //         .auth(vec![key.into()])
    //         .build()
    //         .unwrap();
    
    //     Ok(doc)
    // }
}
