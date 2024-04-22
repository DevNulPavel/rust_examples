
use super::*;

#[test]
fn test_chain() {
    let mut chain = Blockchain::new();
    assert!(chain.chain().is_valid()); // all chains are initially valid
    
    for i in 1..5 {
        // Вкидываем транзакции
        chain.add_transaction(Transaction {
            from: UserId(i),
            to: UserId(i + 1),
            amount: (i + 2) as i128,
        });
        chain.add_transaction(Transaction {
            from: UserId(i + 3),
            to: UserId(i + 4),
            amount: (i + 5) as i128,
        });
        chain.add_transaction(Transaction {
            from: UserId(i + 6),
            to: UserId(i + 7),
            amount: (i + 8) as i128,
        });

        // Майним
        chain.mine();
    }
    assert!(chain.chain().is_valid());
}
