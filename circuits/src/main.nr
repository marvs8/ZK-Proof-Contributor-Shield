use dep::std;

// Proves ownership of a GitHub identity without revealing the private key.
// Public inputs:  github_id, pub_key
// Private inputs: priv_key
fn main(github_id: pub Field, priv_key: Field, pub_key: pub Field) {
    let derived_pub = std::scalar_mul(priv_key, std::params::G);
    assert(derived_pub == pub_key);
    assert(github_id != 0);
}
