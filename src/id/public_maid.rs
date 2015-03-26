/*  Copyright 2014 MaidSafe.net limited

    This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
    version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
    licence you accepted on initial access to the Software (the "Licences").

    By contributing code to the MaidSafe Software, or to this project generally, you agree to be
    bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
    directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
    available at: http://www.maidsafe.net/licenses

    Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
    under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
    OF ANY KIND, either express or implied.

    See the Licences for the specific language governing permissions and limitations relating to
    use of the MaidSafe Software.                                                                 */
extern crate "rustc-serialize" as rustc_serialize;
extern crate sodiumoxide;
extern crate cbor;

use cbor::CborTagEncode;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use sodiumoxide::crypto;
use helper::*;
use common::NameType;
use traits::RoutingTrait;

/// PublicMaid
///
/// #Examples
///
/// ```
/// extern crate sodiumoxide;
/// extern crate maidsafe_types;
///
/// // Generating sign and asymmetricbox keypairs,
/// let (pub_sign_key, _) = sodiumoxide::crypto::sign::gen_keypair(); // returns (PublicKey, SecretKey)
/// let (pub_asym_key, _) = sodiumoxide::crypto::asymmetricbox::gen_keypair();
///
/// // Creating new PublicMaid
/// let public_maid  = maidsafe_types::PublicMaid::new((pub_sign_key, pub_asym_key),
///                     sodiumoxide::crypto::sign::Signature([2u8; 64]),
///                     maidsafe_types::NameType([8u8; 64]),
///                     sodiumoxide::crypto::sign::Signature([5u8; 64]),
///                     maidsafe_types::NameType([6u8; 64]));
///
/// // getting PublicMaid::public_keys
/// let &(pub_sign, pub_asym) = public_maid.get_public_keys();
///
/// // getting PublicMaid::mpid_signature
/// let maid_signature: &sodiumoxide::crypto::sign::Signature = public_maid.get_maid_signature();
///
/// // getting PublicMaid::owner
/// let owner: &maidsafe_types::NameType = public_maid.get_owner();
///
/// // getting PublicMaid::signature
/// let signature: &sodiumoxide::crypto::sign::Signature = public_maid.get_signature();
///
/// // getting PublicMaid::name
/// let name: &maidsafe_types::NameType = public_maid.get_name();
/// ```
pub struct PublicMaid {
	public_keys: (crypto::sign::PublicKey, crypto::asymmetricbox::PublicKey),
	maid_signature: crypto::sign::Signature,
	owner: NameType,
	signature: crypto::sign::Signature,
	name: NameType
}

impl RoutingTrait for PublicMaid {
    fn get_name(&self) -> NameType {
        let sign_arr = &(&self.public_keys.0).0;
        let asym_arr = &(&self.public_keys.1).0;

        let mut arr_combined = [0u8; 64 * 2];

        for i in 0..sign_arr.len() {
            arr_combined[i] = sign_arr[i];
        }
        for i in 0..asym_arr.len() {
            arr_combined[64 + i] = asym_arr[i];
        }

        let digest = crypto::hash::sha512::hash(&arr_combined);

        NameType(digest.0)
    }

    fn get_owner(&self) -> Option<Vec<u8>> {
        Some(array_as_vector(&self.owner.0))
    }
}

impl PublicMaid {
	pub fn new(public_keys: (crypto::sign::PublicKey, crypto::asymmetricbox::PublicKey),
						 maid_signature: crypto::sign::Signature,
						 owner: NameType,
						 signature: crypto::sign::Signature,
						 name: NameType) -> PublicMaid {
		PublicMaid {
		public_keys: public_keys,
		maid_signature: maid_signature,
		owner: owner,
		signature: signature,
		name: name,
		}
	}

	pub fn get_public_keys(&self) -> &(crypto::sign::PublicKey, crypto::asymmetricbox::PublicKey) {
		&self.public_keys
	}

	pub fn get_maid_signature(&self) -> &crypto::sign::Signature {
		&self.maid_signature
	}

	pub fn get_owner(&self) -> &NameType {
		&self.owner
	}

	pub fn get_signature(&self) -> &crypto::sign::Signature {
		&self.signature
	}

	pub fn get_name(&self) -> &NameType {
		&self.name
	}
}

impl Encodable for PublicMaid {
	fn encode<E: Encoder>(&self, e: &mut E)->Result<(), E::Error> {
		let (crypto::sign::PublicKey(pub_sign_vec), crypto::asymmetricbox::PublicKey(pub_asym_vec)) = self.public_keys;
		let crypto::sign::Signature(maid_signature) = self.maid_signature;
		let crypto::sign::Signature(signature) = self.signature;
		CborTagEncode::new(5483_001, &(
				array_as_vector(&pub_sign_vec),
					array_as_vector(&pub_asym_vec),
					array_as_vector(&maid_signature),
				&self.owner,
					array_as_vector(&signature),
				&self.name)).encode(e)
	}
}

impl Decodable for PublicMaid {
	fn decode<D: Decoder>(d: &mut D)-> Result<PublicMaid, D::Error> {
		try!(d.read_u64());
		let(pub_sign_vec, pub_asym_vec, maid_signature, owner, signature, name) = try!(Decodable::decode(d));
		let pub_keys = (crypto::sign::PublicKey(vector_as_u8_32_array(pub_sign_vec)),
				crypto::asymmetricbox::PublicKey(vector_as_u8_32_array(pub_asym_vec)));
		let parsed_maid_signature = crypto::sign::Signature(vector_as_u8_64_array(maid_signature));
		let parsed_signature = crypto::sign::Signature(vector_as_u8_64_array(signature));

		Ok(PublicMaid::new(pub_keys, parsed_maid_signature, owner, parsed_signature, name))
	}
}

#[test]
fn serialisation_public_maid() {
	let (pub_sign_key, _) = crypto::sign::gen_keypair();
	let (pub_asym_key, _) = crypto::asymmetricbox::gen_keypair();

	let obj_before = PublicMaid::new((pub_sign_key, pub_asym_key), crypto::sign::Signature([5u8; 64]),
	NameType([5u8; 64]), crypto::sign::Signature([5u8; 64]), NameType([3u8; 64]));

	let mut e = cbor::Encoder::from_memory();
	e.encode(&[&obj_before]).unwrap();

	let mut d = cbor::Decoder::from_bytes(e.as_bytes());
	let obj_after: PublicMaid = d.decode().next().unwrap().unwrap();

	let &(crypto::sign::PublicKey(pub_sign_arr_before), crypto::asymmetricbox::PublicKey(pub_asym_arr_before)) = obj_before.get_public_keys();
	let &(crypto::sign::PublicKey(pub_sign_arr_after), crypto::asymmetricbox::PublicKey(pub_asym_arr_after)) = obj_after.get_public_keys();
	let (&crypto::sign::Signature(maid_signature_before), &crypto::sign::Signature(maid_signature_after)) = (obj_before.get_maid_signature(), obj_after.get_maid_signature());
	let (&crypto::sign::Signature(signature_before), &crypto::sign::Signature(signature_after)) = (obj_before.get_signature(), obj_after.get_signature());
	let (&NameType(name_before), &NameType(name_after)) = (obj_before.get_name(), obj_after.get_name());
	let (&NameType(owner_before), &NameType(owner_after)) = (obj_after.get_owner(), obj_after.get_owner());

	assert!(compare_u8_array(&name_before, &name_after));
	assert!(compare_u8_array(&owner_before, &owner_after));
	assert_eq!(pub_sign_arr_before, pub_sign_arr_after);
	assert_eq!(pub_asym_arr_before, pub_asym_arr_after);
	assert!(compare_u8_array(&maid_signature_before, &maid_signature_after));
	assert!(compare_u8_array(&signature_before, &signature_after));
}
