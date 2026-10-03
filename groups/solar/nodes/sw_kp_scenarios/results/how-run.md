These are the node's fixtures in the design (`/home/user/VLEO_SIMULATOR/crates/vleo-mod-solar/nodes/sw_kp_scenarios/fixtures.toml`): values worked out outside the code that computes this node — by hand from the cited record, or read from a published source — each with the tolerance the design holds it to. `origin` says which. They are the reference the developer's code is tested against.

Two rows have origin `code`: the answers this node's own code gave, which is the code its pseudocode was copied from. They were run on 2026-10-03 at commit ff63c5a9 of this repository, rustc 1.94.1, on the developer's Linux machine, by calling the function below directly from a Rust test with the inputs in SI (`cargo test -p vleo-mod-solar`). The code is in `code/model.rs`, copied unchanged from `crates/vleo-mod-solar/nodes/sw_kp_scenarios/model.rs`.

**Entry:** `evaluate`
