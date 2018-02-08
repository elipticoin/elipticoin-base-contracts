use base_token::*;
use test::fake_blockchain::FakeBlockChain;
use test::fake_blockchain::SENDER;

#[test]
fn balance_of() {
    let fake_blockchain =  FakeBlockChain {..Default::default()};
    let base_token =  BaseToken { blockchain: fake_blockchain };
    base_token._initialize();
    let balance = base_token.balance_of(SENDER.to_vec());
    assert_eq!(balance, 100);
}
