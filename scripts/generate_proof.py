"""
Off-chain helper: generate a ZK proof for GitHub identity ownership.
Requires: nargo (Noir toolchain) installed and in PATH.
"""
import subprocess
import json
import sys
from pathlib import Path

CIRCUITS_DIR = Path(__file__).parent.parent / "circuits"


def generate_proof(github_id: int, priv_key: str, pub_key: str) -> dict:
    """Write Prover.toml, run nargo prove, return proof + public inputs."""
    prover_toml = CIRCUITS_DIR / "Prover.toml"
    prover_toml.write_text(
        f'github_id = "{github_id}"\n'
        f'priv_key = "{priv_key}"\n'
        f'pub_key = "{pub_key}"\n'
    )

    result = subprocess.run(
        ["nargo", "prove"],
        cwd=CIRCUITS_DIR,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        raise RuntimeError("nargo prove failed")

    proof_path = CIRCUITS_DIR / "proofs" / "contributor_shield.proof"
    return {"proof": proof_path.read_text().strip()}


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Generate contributor ZK proof")
    parser.add_argument("--github-id", required=True, type=int)
    parser.add_argument("--priv-key", required=True)
    parser.add_argument("--pub-key", required=True)
    args = parser.parse_args()

    result = generate_proof(args.github_id, args.priv_key, args.pub_key)
    print(json.dumps(result, indent=2))
