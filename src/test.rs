use crate::{AnchornetContract, AnchornetContractClient};
use soroban_sdk::{symbol_short, Env, Symbol, Vec};

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AnchornetContract);
    let client = AnchornetContractClient::new(&env, &contract_id);

    let to = symbol_short!("Anchor");
    let result: Vec<Symbol> = client.hello(&to);

    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap(), symbol_short!("greeting"));
    assert_eq!(result.get(1).unwrap(), symbol_short!("Anchor"));
}
