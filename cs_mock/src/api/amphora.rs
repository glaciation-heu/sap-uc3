use once_cell::sync::Lazy;
use std::{collections::HashMap, str::FromStr, sync::Mutex};
use tracing::{event, Level};

use base64::{prelude::BASE64_STANDARD, Engine};
use num_bigint::{BigInt, RandBigInt};
use num_integer::Integer;
use poem::Result;
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    ApiResponse, Object, OpenApi,
};
use std::ops::{Add, Mul};

use super::utils::{to_bigint_arr, to_gfp};

pub struct AmphoraApi;
#[OpenApi]
impl AmphoraApi {
    /// Retrieve a set of party individual InputMask shares. The InputMaks can be used to secret share confidential data (compute a MaskedInput).
    #[oai(path = "/:vcp_nr/amphora/input-masks", method = "get")]
    async fn get_input_masks(
        &self,
        vcp_nr: Path<i32>,
        #[oai(name = "requestId")] request_id: Query<String>,
        count: Query<i32>
    ) -> Result<GetInputMasksResponses> {
        event!(Level::INFO, "Request to get input-masks from vcp {}", vcp_nr.0);
        Ok(GetInputMasksResponses::OK(Json(generate_input_masks(
            vcp_nr.0,
            request_id.0,
            count.0,
        ))))
    }

    /// A MaskedInput is used to securely secret share values with the participating parties. Amphora will then use this MaskedInput to compute its individual SecretShare together with the provided tags.
    #[oai(path = "/:vcp_nr/amphora/masked-inputs", method = "post")]
    async fn post_masked_input(
        &self,
        vcp_nr: Path<i32>,
        body: Json<UploadMaskedInputObject>,
    ) -> Result<PostMaskedInputResponse> {
        event!(
            Level::DEBUG,
            "Adding masked input. Data lenght: {}",
            body.0.data.len()
        );
        add_secrets(vcp_nr.0, &body.0.secret_id, &body.0.data);
        Ok(PostMaskedInputResponse::OK(Json(body.0.secret_id)))
    }

    #[oai(path = "/:vcp_nr/amphora/secret-shares/:secretId", method = "get")]
    async fn get_secret_share(
        &self,
        vcp_nr: Path<i32>,
        #[oai(name = "secretId")] secret_id: Path<String>,
        #[oai(name = "requestId")] request_id: Query<String>,
    ) -> Result<GetSecretShareResponse> {
        let id = secret_id.0.clone();
        let odo = get_secret_share(vcp_nr.0, &secret_id.0, &request_id.0);
        Ok(GetSecretShareResponse::OK(Json(SecretShareResponse {
            secret_id: id,
            tags: Vec::new(),
            secret_shares: odo.secret_shares.clone(),
            v_shares: odo.v_shares,
            w_shares: odo.w_shares,
            r_shares: odo.r_shares,
            u_shares: odo.u_shares,
            data: odo.secret_shares,
        })))
    }
}

#[derive(ApiResponse)]
enum GetInputMasksResponses {
    #[oai(status = 200)]
    OK(Json<OutputDeliveryObject>),
}

#[derive(ApiResponse)]
enum PostMaskedInputResponse {
    #[oai(status = 201)]
    OK(Json<String>),
}

#[derive(ApiResponse)]
enum GetSecretShareResponse {
    #[oai(status = 200)]
    OK(Json<SecretShareResponse>),
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
pub struct OutputDeliveryObject {
    pub secret_shares: String,
    pub r_shares: String,
    pub v_shares: String,
    pub w_shares: String,
    pub u_shares: String,
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
struct DataObject {
    value: String,
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
struct UploadMaskedInputObject {
    secret_id: String,
    data: Vec<DataObject>,
    tags: Option<Vec<Tag>>,
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
struct Tag {
    key: String,
    value: String,
    value_type: Option<String>,
}

#[derive(Object)]
#[oai(rename_all = "camelCase")]
struct SecretShareResponse {
    secret_id: String,
    tags: Vec<Tag>,
    secret_shares: String,
    r_shares: String,
    v_shares: String,
    w_shares: String,
    u_shares: String,
    data: String,
}

struct InputMasksObject {
    secret: (BigInt, BigInt),
    r: (BigInt, BigInt),
    v: (BigInt, BigInt),
    u: (BigInt, BigInt),
    w: (BigInt, BigInt),
}
pub static P: Lazy<BigInt> =
    Lazy::new(|| BigInt::from_str("198766463529478683931867765928436695041").unwrap());
pub static R: Lazy<BigInt> =
    Lazy::new(|| BigInt::from_str("141515903391459779531506841503331516415").unwrap());
pub static R_INV: Lazy<BigInt> =
    Lazy::new(|| BigInt::from_str("133854242216446749056083838363708373830").unwrap());

impl InputMasksObject {
    fn generate() -> InputMasksObject {
        let mut rng = rand::thread_rng();
        // Generate a random pair
        let s = (
            rng.gen_bigint(256).mod_floor(&P),
            rng.gen_bigint(256).mod_floor(&P),
            //BigInt::from_str("12345").unwrap(),
            //BigInt::from_str("67890").unwrap(),
        );
        InputMasksObject::generate_for(s)
    }

    fn generate_for(s: (BigInt, BigInt)) -> InputMasksObject {
        let mut rng = rand::thread_rng();
        // Generate r-values
        let r = (
            rng.gen_bigint(256).mod_floor(&P),
            rng.gen_bigint(256).mod_floor(&P),
            //BigInt::from_str("23451").unwrap(),
            //BigInt::from_str("78906").unwrap(),
        );
        // Generate v-values
        let v = (
            rng.gen_bigint(256).mod_floor(&P),
            rng.gen_bigint(256).mod_floor(&P),
            //BigInt::from_str("34512").unwrap(),
            //BigInt::from_str("89067").unwrap(),
        );
        let w = (
            s.0.clone()
                .mul(&r.0)
                .add(s.0.clone().mul(&r.1))
                .mod_floor(&P), //w_0 = s_0 * r_0 + s_0 * r_1
            s.1.clone()
                .mul(&r.1)
                .add(s.1.clone().mul(&r.0))
                .mod_floor(&P), //w_1 = s_1 * r_1 + s_1 * r_0
        );
        let u = (
            v.0.clone()
                .mul(&r.0)
                .add(v.0.clone().mul(&r.1))
                .mod_floor(&P), // u_0 = v_0 * r_0 + v_0 * r_1
            v.1.clone()
                .mul(&r.1)
                .add(v.1.clone().mul(&r.0))
                .mod_floor(&P), // u_1 = v_1 * r_1 + v_1 * r_0
        );
        InputMasksObject {
            secret: s,
            r,
            v,
            u,
            w,
        }
    }
}

// Use global varriable to save "randomness"
static GLOBAL_RANDOMNESS: Lazy<Mutex<HashMap<String, Vec<InputMasksObject>>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

static GLOBAL_SECRETS: Lazy<Mutex<HashMap<String, Vec<(BigInt, BigInt)>>>> = Lazy::new(|| {
    let m = HashMap::new();
    Mutex::new(m)
});

fn generate_input_masks(vcp: i32, id: String, count: i32) -> OutputDeliveryObject {
    // Try to get previously generated randomness
    let mut random_db = GLOBAL_RANDOMNESS.lock().unwrap();
    if !random_db.contains_key(&id) {
        let mut new_input_mask: Vec<InputMasksObject> = Vec::new();
        for _ in 0..count {
            new_input_mask.push(InputMasksObject::generate());
        }
        random_db.insert(id.clone(), new_input_mask);
    }

    // Get input-masks from random_db.
    let input_masks = random_db.get(&id).unwrap();
    let bin_output_delivery;
    // crate tuple of (s_share, r_share, v_share, u_share, w_share) for VCP0 or VCP1
    if vcp > 0 {
        bin_output_delivery = input_masks.iter().fold(
            (
                Vec::<u8>::new(),
                Vec::<u8>::new(),
                Vec::<u8>::new(),
                Vec::<u8>::new(),
                Vec::<u8>::new(),
            ),
            |mut v, m| {
                v.0.append(&mut to_gfp(&m.secret.1, &R, &P));
                v.1.append(&mut to_gfp(&m.r.1, &R, &P));
                v.2.append(&mut to_gfp(&m.v.1, &R, &P));
                v.3.append(&mut to_gfp(&m.u.1, &R, &P));
                v.4.append(&mut to_gfp(&m.w.1, &R, &P));
                v
            },
        );
    } else {
        bin_output_delivery = input_masks.iter().fold(
            (
                Vec::<u8>::new(),
                Vec::<u8>::new(),
                Vec::<u8>::new(),
                Vec::<u8>::new(),
                Vec::<u8>::new(),
            ),
            |mut v, m| {
                v.0.append(&mut to_gfp(&m.secret.0, &R, &P));
                v.1.append(&mut to_gfp(&m.r.0, &R, &P));
                v.2.append(&mut to_gfp(&m.v.0, &R, &P));
                v.3.append(&mut to_gfp(&m.u.0, &R, &P));
                v.4.append(&mut to_gfp(&m.w.0, &R, &P));
                v
            },
        );
    }
    OutputDeliveryObject {
        secret_shares: BASE64_STANDARD.encode(bin_output_delivery.0),
        r_shares: BASE64_STANDARD.encode(bin_output_delivery.1),
        v_shares: BASE64_STANDARD.encode(bin_output_delivery.2),
        u_shares: BASE64_STANDARD.encode(bin_output_delivery.3),
        w_shares: BASE64_STANDARD.encode(bin_output_delivery.4),
    }
}

pub fn delete_secret(secret_id: &String) {
    let mut secrets = GLOBAL_SECRETS.lock().unwrap();
    secrets.remove(secret_id);
}

pub fn get_secrets_internal(secret_id: &String) -> Vec<(BigInt, BigInt)> {
    let secrets = GLOBAL_SECRETS.lock().unwrap();
    if let Some(s) = secrets.get(secret_id) {
        event!(Level::INFO, "Found secrets for id {}", secret_id);
        s.clone()
    } else {
        event!(Level::WARN, "Did not found secrets for id {}", secret_id);
        Vec::new()
    }
}

pub fn get_secret_share(vcp: i32, secret_id: &String, request_id: &String) -> OutputDeliveryObject {
    let secrets = GLOBAL_SECRETS.lock().unwrap();
    if let Some(secret) = secrets.get(secret_id) {
        event!(
            Level::INFO,
            "get shares vcp {}, s1: {}, s2: {}",
            vcp,
            secret[0].0,
            secret[0].1
        );
        let mut random_db = GLOBAL_RANDOMNESS.lock().unwrap();
        if !random_db.contains_key(request_id) {
            let mut new_input_mask: Vec<InputMasksObject> = Vec::new();
            for s in secret {
                new_input_mask.push(InputMasksObject::generate_for(s.clone()));
            }
            random_db.insert(request_id.clone(), new_input_mask);
        }
        let input_masks: &Vec<InputMasksObject> = random_db.get(request_id).unwrap();

        // Create output delivery object for VCP0 or VCP1
        let bin_output_delivery = secret
            .iter()
            .map(|v| if vcp > 0 { &v.1 } else { &v.0 })
            .zip(input_masks)
            .fold(
                (
                    Vec::<u8>::new(),
                    Vec::<u8>::new(),
                    Vec::<u8>::new(),
                    Vec::<u8>::new(),
                    Vec::<u8>::new(),
                ),
                |mut v, m| {
                    v.0.append(&mut to_gfp(&m.0, &R, &P));
                    if vcp > 0 {
                        v.1.append(&mut to_gfp(&m.1.r.1, &R, &P));
                        v.2.append(&mut to_gfp(&m.1.v.1, &R, &P));
                        v.3.append(&mut to_gfp(&m.1.u.1, &R, &P));
                        v.4.append(&mut to_gfp(&m.1.w.1, &R, &P));
                    } else {
                        v.1.append(&mut to_gfp(&m.1.r.0, &R, &P));
                        v.2.append(&mut to_gfp(&m.1.v.0, &R, &P));
                        v.3.append(&mut to_gfp(&m.1.u.0, &R, &P));
                        v.4.append(&mut to_gfp(&m.1.w.0, &R, &P));
                    }
                    v
                },
            );
        return OutputDeliveryObject {
            secret_shares: BASE64_STANDARD.encode(bin_output_delivery.0),
            r_shares: BASE64_STANDARD.encode(bin_output_delivery.1),
            v_shares: BASE64_STANDARD.encode(bin_output_delivery.2),
            u_shares: BASE64_STANDARD.encode(bin_output_delivery.3),
            w_shares: BASE64_STANDARD.encode(bin_output_delivery.4),
        };
    } else {
        return OutputDeliveryObject {
            secret_shares: "".to_string(),
            r_shares: "".to_string(),
            v_shares: "".to_string(),
            w_shares: "".to_string(),
            u_shares: "".to_string(),
        };
    }
}

fn add_secrets(vcp: i32, secret_id: &String, data: &Vec<DataObject>) {
    event!(Level::INFO, "From {} with data len {} and id {}", vcp, data.len(), secret_id);
    let random_db = GLOBAL_RANDOMNESS.lock().unwrap();
    let input_masks = random_db.get(secret_id).expect("Secret ID not found");
    // let decoded_data = BASE64_STANDARD.decode(&data).expect("Unable to decode base64 encoded data");
    // let decoded_data = to_bigint_arr(&decoded_data, &R_INV, &P);
    let decoded_data = data
        .iter()
        .map(|d| {
            BASE64_STANDARD
                .decode(&d.value)
                .expect("Unable to decode base64 encoded data")
        })
        .map(|decoded| to_bigint_arr(&decoded, &R_INV, &P))
        .flatten()
        .collect::<Vec<BigInt>>();
    assert!(input_masks.len() == decoded_data.len());

    // Calculate shares by adding the decoded data to the input masks of the VCP mod P
    let shares: Vec<BigInt> = decoded_data
        .iter()
        .zip(input_masks)
        .map(|z| {
            //z.0.add(
            //    match vcp {
            //        0 => z.1.secret.0.clone(),
            //        _ => z.1.secret.1.clone()
            //    }
            //).mod_floor(&P)
            let secret_to_add: BigInt = if vcp > 0 {
                z.1.secret.1.clone()
            } else {
                z.1.secret.0.clone()
            };
            //event!(
            //    Level::INFO,
            //    "vcp {} Adding {} to {}",
            //    vcp,
            //    z.0,
            //    secret_to_add
            //);
            let mut val =
                z.0.add(secret_to_add.mul(BigInt::from_str("2").unwrap()))
                    .div_floor(&BigInt::from_str("2").unwrap());
            if vcp > 0 {
                val = val.add(z.0.mod_floor(&BigInt::from_str("2").unwrap()));
            }
            val.mod_floor(&P)
        })
        .collect();
    event!(Level::INFO, "secret: {}", shares[0]);
    // Save share to GLOBAL_SECRETS
    let mut secrets = GLOBAL_SECRETS.lock().unwrap();
    if secrets.contains_key(secret_id) {
        secrets
            .get_mut(secret_id)
            .unwrap()
            .iter_mut()
            .zip(shares)
            .for_each(|(s, v)| {
                if vcp > 0 {
                    s.1 = v;
                } else {
                    s.0 = v;
                }
            });
    } else {
        secrets.insert(
            secret_id.clone(),
            shares
                .iter()
                .map(|s| {
                    if vcp > 0 {
                        (BigInt::from_str("0").unwrap(), s.clone())
                    } else {
                        (s.clone(), BigInt::from_str("0").unwrap())
                    }
                })
                .collect(),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::utils::zipp_shares;

    use std::ops::Sub;

    #[test]
    fn test_secret() {
        let uuid = "b3bde039-d497-4b71-9956-db12f2dddbde".to_string();
        let share = DataObject {
            value: "jhXaWICsBMZadvGKldslPg==".to_string(),
        };
        let share = vec![share];
        add_secrets(0, &uuid, &share);
        add_secrets(1, &uuid, &share);
        let odo_0 = get_secret_share(0, &uuid, &uuid);
        let odo_1 = get_secret_share(0, &uuid, &uuid);
        let s_0 = to_bigint_arr(
            &BASE64_STANDARD.decode(odo_0.secret_shares).unwrap(),
            &R_INV,
            &P,
        );
        let s_1 = to_bigint_arr(
            &BASE64_STANDARD.decode(odo_1.secret_shares).unwrap(),
            &R_INV,
            &P,
        );
        s_0.iter().zip(s_1).for_each(|(l, r)| {
            let res = l.add(r).mod_floor(&P);
            assert_eq!(res.to_string(), "450".to_string());
        });
    }

    #[test]
    fn complete_test() {
        let uuid = "b3bde039-d497-4b71-9956-db12f2dddbee".to_string();
        let secret = BigInt::from_str("500000").unwrap();
        // generate input masks from both providers
        let m_0 = generate_input_masks(0, uuid.clone(), 1);
        let m_1 = generate_input_masks(1, uuid.clone(), 1);

        // encode secret with the help of input masks
        let zipped = zipp_shares(&vec![m_0, m_1], &R_INV, &P);
        // share = secret - (r1 + r2) where r1, r2 are from the OutputDeliveryObjects
        let secret_shared = secret.clone().sub(zipped.secrets[0].clone()).mod_floor(&P);
        let encoded_share = BASE64_STANDARD.encode(to_gfp(&secret_shared, &R, &P));
        let share_list = vec![DataObject {
            value: encoded_share,
        }];
        // Add secrets
        add_secrets(0, &uuid, &share_list);
        add_secrets(1, &uuid, &share_list);

        //let uuid = "b3bde039-d497-4b71-9956-db12f2dddbeb".to_string();

        let odo_0 = get_secret_share(0, &uuid, &uuid);
        let odo_1 = get_secret_share(1, &uuid, &uuid);
        let s_0 = to_bigint_arr(
            &BASE64_STANDARD.decode(odo_0.secret_shares).unwrap(),
            &R_INV,
            &P,
        );
        let s_1 = to_bigint_arr(
            &BASE64_STANDARD.decode(odo_1.secret_shares).unwrap(),
            &R_INV,
            &P,
        );

        s_0.iter().zip(s_1).for_each(|(l, r)| {
            let res = l.add(r).mod_floor(&P);
            assert_eq!(res, secret.clone());
        });
    }
    #[test]
    fn test_multiple_shares() {
        let uuid = "b3bde039-d497-4b71-9956-db12f2dddbec".to_string();
        let secrets = vec![BigInt::from_str("1").unwrap(),BigInt::from_str("2").unwrap(),BigInt::from_str("3").unwrap()];
        // generate input masks from both providers
        let m_0 = generate_input_masks(0, uuid.clone(), secrets.len() as i32);
        let m_1 = generate_input_masks(1, uuid.clone(), secrets.len() as i32);

        // encode secret with the help of input masks
        let zipped = zipp_shares(&vec![m_0, m_1], &R_INV, &P);
        // share = secret - (r1 + r2) where r1, r2 are from the OutputDeliveryObjects
        //let secret_shared = secret.clone().sub(zipped.secrets[0].clone()).mod_floor(&P);
        let secret_shares = secrets.iter()
            .cloned()
            .zip(zipped.secrets)
            .map(|(s,z)| s.sub(z).mod_floor(&P))
            .map(        |secret_share| BASE64_STANDARD.encode(to_gfp(&secret_share, &R, &P)))
            .map(|encoded  | DataObject{value: encoded})
            .collect::<Vec<DataObject>>();
        //let encoded_share = BASE64_STANDARD.encode(to_gfp(&secret_shared, &R, &P));
        //let share_list = vec![DataObject {
        //    value: encoded_share,
        //}];
        // Add secrets
        add_secrets(0, &uuid, &secret_shares);
        add_secrets(1, &uuid, &secret_shares);

        //let uuid = "b3bde039-d497-4b71-9956-db12f2dddbeb".to_string();

        let odo_0 = get_secret_share(0, &uuid, &uuid);
        let odo_1 = get_secret_share(1, &uuid, &uuid);
        let s_0 = to_bigint_arr(
            &BASE64_STANDARD.decode(odo_0.secret_shares).unwrap(),
            &R_INV,
            &P,
        );
        let s_1 = to_bigint_arr(
            &BASE64_STANDARD.decode(odo_1.secret_shares).unwrap(),
            &R_INV,
            &P,
        );

        s_0.iter().zip(s_1).enumerate().for_each(|(i,(l, r))| {
            let res = l.add(r).mod_floor(&P);
            assert_eq!(res, secrets[i].clone());
        });
    }
}
