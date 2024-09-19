use crate::Graph;

type UserId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
struct User {
    id: UserId,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relationship {
    Follows,
    Blocks,
}

impl std::borrow::Borrow<UserId> for User {
    fn borrow(&self) -> &UserId {
        &self.id
    }
}
type UserGraph = Graph<UserId, User, Relationship>;

#[test]
fn test_add_node_to_empty() {
    let graph: UserGraph = Graph::default();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);

    let alice = graph.get_node(&1);
    assert!(alice.is_none());

    let alice = User {
        id: 1,
        name: "Alice".to_string(),
    };
    graph.insert(alice.id, alice.clone());
    assert!(!graph.is_empty());
    assert_eq!(graph.len(), 1);

    let found_alice = graph.get_node(&1).unwrap();
    assert_eq!(found_alice.value(), &alice);
}

#[test]
fn test_from_iter() {
    let alice = User {
        id: 1,
        name: "Alice".to_string(),
    };
    let bob = User {
        id: 2,
        name: "Bob".to_string(),
    };
    let charlie = User {
        id: 3,
        name: "Charlie".to_string(),
    };
    let users: Graph<UserId, User, Relationship> = [alice.clone(), bob.clone(), charlie.clone()]
        .into_iter()
        .collect();

    assert_eq!(users.get_node(&1).unwrap().value().name, "Alice");

    // Alice follows Bob, and Bob blocks Charlie
    users.add_edge(1, 2, Relationship::Follows);
    users.add_edge(2, 3, Relationship::Blocks);

    // Test that edges from Alice are correct
    let alice_relations = users.edges_from(&alice.id).unwrap();
    assert_eq!(alice_relations.len(), 1);

    let alice_to_bob = alice_relations[0];
    assert_eq!(alice_to_bob.1, bob.id);
    assert_eq!(
        *users.get_edge(alice_to_bob.0).unwrap(),
        Relationship::Follows
    );

    // Test that edges to Bob are correct
    let who_follows_bob = users.edges_to(&bob.id).unwrap();
    assert_eq!(who_follows_bob.len(), 1);
    let bob_from_alice = who_follows_bob[0];
    // This is the same edge
    assert_eq!(bob_from_alice.0, alice_to_bob.0);
}
