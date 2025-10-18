use super::*;

#[test]
fn test_assert() {
	let signer = Signer::generate();

	let hello = "Hello World!!! 4skjgl;asdfv;dnvoef";
	let encrypted = signer.encrypt_text(hello).expect("failed to encrypt");
	let decrypted = signer.decrypt_text(&encrypted).expect("failed to decrpyt");
	assert_eq!(hello, decrypted);
}
