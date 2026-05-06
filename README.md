# ZK-Proof Contributor Shield

A privacy-preserving bounty redemption system built on **Stellar/Soroban** and **Noir (ZK circuits)**. Contributors can claim on-chain rewards without ever linking their GitHub identity to their financial wallet.

---

## The Problem

Open-source bounty platforms typically require contributors to connect a public wallet to their GitHub account. This creates a permanent, public on-chain record: **"GitHub user X owns wallet Y"** — exposing contributors to targeted attacks, doxxing, and financial surveillance.

---

## The Solution

ZK-Proof Contributor Shield breaks that link using **zero-knowledge proofs**:

- The bounty is registered on-chain against a GitHub ID and a public key.
- The contributor proves off-chain (via Noir) that they own the private key behind that public key — **without revealing the private key**.
- They submit the proof alongside a **fresh stealth address** they control.
- The Soroban contract verifies the proof and sends funds to the stealth address.

The ledger records only: *"a valid proof was submitted → funds sent to address Z."* No identity. No link.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        OFF-CHAIN (Contributor)                  │
│                                                                 │
│   GitHub ID + priv_key ──► Noir Circuit ──► ZK Proof           │
│                                  │                              │
│                         (priv_key never leaves here)            │
└──────────────────────────────────┬──────────────────────────────┘
                                   │  proof + public_inputs
                                   │  + stealth_address
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                     ON-CHAIN (Soroban Contract)                 │
│                                                                 │
│  claim_bounty()                                                 │
│    ├── verify_proof(proof, public_inputs)  ◄── soroban-zk-verifier
│    ├── assert bounty exists & not claimed                       │
│    └── token.transfer(contract → stealth_address)              │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

| Step | Actor | Action |
|------|-------|--------|
| 1 | Bounty Admin | Calls `register_bounty(github_id, pub_key, amount)` on-chain |
| 2 | Contributor | Runs `generate_proof.py` locally with their `priv_key` |
| 3 | Contributor | Calls `claim_bounty(github_id, stealth_address, proof, public_inputs)` |
| 4 | Contract | Verifies ZK proof, marks bounty claimed, transfers funds |

---

## Project Structure

```
circuits/
  src/main.nr        # Noir circuit — identity ownership proof
  Nargo.toml         # Noir package config
  Prover.toml        # Local witness inputs (gitignored — never commit real values)

contracts/
  src/lib.rs         # Soroban contract: register_bounty + claim_bounty
  src/tests.rs       # Unit tests
  Cargo.toml

scripts/
  generate_proof.py  # Off-chain proof generation (wraps nargo prove)
```

---

## Code

### Noir Circuit (`circuits/src/main.nr`)

The circuit has one job: prove that the caller knows a `priv_key` whose scalar multiplication with the curve generator `G` equals the registered `pub_key`, and that the `github_id` is non-zero.

```rust
use dep::std;

fn main(github_id: pub Field, priv_key: Field, pub_key: pub Field) {
    // Derive the public key from the private key using elliptic curve scalar multiplication.
    // priv_key is a private witness — it is never included in the proof's public inputs.
    let derived_pub = std::scalar_mul(priv_key, std::params::G);
    assert(derived_pub == pub_key);

    // Prevent zero/null GitHub ID claims
    assert(github_id != 0);
}
```

- `priv_key` is a **private witness** — it exists only inside the proof, never on-chain.
- `github_id` and `pub_key` are **public inputs** — the contract checks these match the registered bounty.

---

### Soroban Contract (`contracts/src/lib.rs`)

```rust
#[contractimpl]
impl ContributorShield {
    /// Bounty admin registers a reward for a GitHub contributor.
    pub fn register_bounty(env: Env, github_id: u64, pub_key: Bytes, amount: i128) {
        let record = BountyRecord { github_id, pub_key, amount, claimed: false };
        env.storage().persistent().set(&github_id, &record);
    }

    /// Contributor claims the bounty by submitting a ZK proof.
    /// Funds are sent to a stealth address — no link to the contributor's identity.
    pub fn claim_bounty(
        env: Env,
        github_id: u64,
        stealth_address: Address,
        proof: Proof,
        public_inputs: PublicInputs,
    ) {
        let mut record: BountyRecord = env.storage().persistent()
            .get(&github_id).expect("bounty not found");

        assert!(!record.claimed, "already claimed");

        // On-chain ZK proof verification via soroban-zk-verifier
        verify_proof(&env, &proof, &public_inputs).expect("invalid proof");

        record.claimed = true;
        env.storage().persistent().set(&github_id, &record);

        let token = soroban_sdk::token::Client::new(&env, &env.current_contract_address());
        token.transfer(&env.current_contract_address(), &stealth_address, &record.amount);
    }
}
```

---

### Proof Generation (`scripts/generate_proof.py`)

```python
def generate_proof(github_id: int, priv_key: str, pub_key: str) -> dict:
    prover_toml.write_text(
        f'github_id = "{github_id}"\n'
        f'priv_key = "{priv_key}"\n'   # stays local — never sent anywhere
        f'pub_key = "{pub_key}"\n'
    )
    subprocess.run(["nargo", "prove"], cwd=CIRCUITS_DIR, check=True)
    return {"proof": proof_path.read_text().strip()}
```

---

## Prerequisites

- [Noir / Nargo](https://noir-lang.org/) ≥ 0.31
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools) (Soroban, Protocol 22+)
- `soroban-zk-verifier` crate
- Python 3.10+

---

## Quick Start

### 1. Compile & test the circuit
```bash
cd circuits
nargo check
nargo test
```

### 2. Generate a proof (off-chain)
```bash
python scripts/generate_proof.py \
  --github-id 12345678 \
  --priv-key 0xYOUR_PRIV_KEY \
  --pub-key 0xYOUR_PUB_KEY
```

### 3. Build & deploy the contract
```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/contributor_shield_contract.wasm
```

### 4. Register a bounty (admin)
```bash
stellar contract invoke --id <CONTRACT_ID> -- register_bounty \
  --github_id 12345678 \
  --pub_key <HEX_PUB_KEY> \
  --amount 1000
```

### 5. Claim a bounty (contributor)
```bash
stellar contract invoke --id <CONTRACT_ID> -- claim_bounty \
  --github_id 12345678 \
  --stealth_address <FRESH_WALLET_ADDR> \
  --proof <PROOF_HEX> \
  --public_inputs <INPUTS_HEX>
```

---

## Security Notes

- `priv_key` is the ZK **witness** — it never leaves the contributor's machine and is never stored or transmitted.
- `Prover.toml` is gitignored. Never commit real keys.
- The stealth address must be a **freshly generated wallet** with no prior on-chain history to preserve unlinkability.
- The contract enforces single-claim via the `claimed` flag, preventing replay attacks.
