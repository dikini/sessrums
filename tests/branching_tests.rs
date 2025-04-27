use sessrums::proto::{
    GChoice, GEnd, GOffer, GSend, GRecv, GlobalProtocolBuilder,
    Role, RoleA, RoleB,
    Project, project,
    Protocol, Send, Recv, End, Choose, Offer,
};

// Define a third role for testing multi-party protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct RoleC;

impl Role for RoleC {
    fn name(&self) -> &'static str {
        "RoleC"
    }
}

#[test]
fn test_simple_choice_projection() {
    // Define a global protocol: RoleA chooses between sending a String or an i32 to RoleB
    type Branch1 = GSend<String, RoleA, RoleB, GEnd>;
    type Branch2 = GSend<i32, RoleA, RoleB, GEnd>;
    type GlobalProtocol = GChoice<RoleA, (Branch1, Branch2)>;
    
    // Project for RoleA (chooser)
    type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
    // Should be Choose<(Send<String, End>, Send<i32, End>)>
    
    // Project for RoleB (receiver)
    type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol;
    // Should be Offer<(Recv<String, End>, Recv<i32, End>)>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Choose<Send<String, End>, Send<i32, End>>>();
    assert_type::<Offer<Recv<String, End>, Recv<i32, End>>>();
}

#[test]
fn test_simple_offer_projection() {
    // Define a global protocol: RoleB offers a choice to RoleA between receiving a String or an i32
    type Branch1 = GRecv<String, RoleA, RoleB, GEnd>;
    type Branch2 = GRecv<i32, RoleA, RoleB, GEnd>;
    type GlobalProtocol = GOffer<RoleB, (Branch1, Branch2)>;
    
    // Project for RoleA (sender)
    type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
    // Should be Choose<(Send<String, End>, Send<i32, End>)>
    
    // Project for RoleB (offeree)
    type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol;
    // Should be Offer<(Recv<String, End>, Recv<i32, End>)>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Choose<Send<String, End>, Send<i32, End>>>();
    assert_type::<Offer<Recv<String, End>, Recv<i32, End>>>();
}

#[test]
fn test_complex_branching_protocol() {
    // Define a complex global protocol:
    // RoleA sends a bool to RoleB, then
    // RoleA chooses between:
    // 1. Sending a String to RoleB, then ending
    // 2. Receiving an i32 from RoleB, then ending
    type Branch1 = GSend<String, RoleA, RoleB, GEnd>;
    type Branch2 = GRecv<i32, RoleB, RoleA, GEnd>;
    type GlobalProtocol = GSend<bool, RoleA, RoleB, GChoice<RoleA, (Branch1, Branch2)>>;
    
    // Project for RoleA
    type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
    // Should be Send<bool, Choose<(Send<String, End>, Recv<i32, End>)>>
    
    // Project for RoleB
    type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol;
    // Should be Recv<bool, Offer<(Recv<String, End>, Send<i32, End>)>>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Send<bool, Choose<Send<String, End>, Recv<i32, End>>>>();
    assert_type::<Recv<bool, Offer<Recv<String, End>, Send<i32, End>>>>();
}

#[test]
fn test_multiparty_branching() {
    // Define a global protocol with three roles:
    // RoleA sends a bool to RoleB, then
    // RoleB chooses between:
    // 1. Sending a String to RoleC, then ending
    // 2. Sending an i32 to RoleA, then ending
    type Branch1 = GSend<String, RoleB, RoleC, GEnd>;
    type Branch2 = GSend<i32, RoleB, RoleA, GEnd>;
    type GlobalProtocol = GSend<bool, RoleA, RoleB, GChoice<RoleB, (Branch1, Branch2)>>;
    
    // Project for RoleA
    type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
    // Should be Send<bool, Offer<(End, Recv<i32, End>)>>
    
    // Project for RoleB
    type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol;
    // Should be Recv<bool, Choose<(Send<String, End>, Send<i32, End>)>>
    
    // Project for RoleC
    type RoleCLocal = <GlobalProtocol as Project<RoleC>>::LocalProtocol;
    // Should be Offer<(Recv<String, End>, End)>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Send<bool, Offer<End, Recv<i32, End>>>>();
    assert_type::<Recv<bool, Choose<Send<String, End>, Send<i32, End>>>>();
    assert_type::<Offer<Recv<String, End>, End>>();
}

#[test]
fn test_nested_branching() {
    // Define a global protocol with nested choices:
    // RoleA chooses between:
    // 1. Sending a String to RoleB, then ending
    // 2. RoleB chooses between:
    //    a. Sending an i32 to RoleA, then ending
    //    b. Sending a bool to RoleA, then ending
    type InnerBranch1 = GSend<i32, RoleB, RoleA, GEnd>;
    type InnerBranch2 = GSend<bool, RoleB, RoleA, GEnd>;
    type Branch1 = GSend<String, RoleA, RoleB, GEnd>;
    type Branch2 = GChoice<RoleB, (InnerBranch1, InnerBranch2)>;
    type GlobalProtocol = GChoice<RoleA, (Branch1, Branch2)>;
    
    // Project for RoleA
    type RoleALocal = <GlobalProtocol as Project<RoleA>>::LocalProtocol;
    // Should be Choose<(Send<String, End>, Offer<(Recv<i32, End>, Recv<bool, End>)>)>
    
    // Project for RoleB
    type RoleBLocal = <GlobalProtocol as Project<RoleB>>::LocalProtocol;
    // Should be Offer<(Recv<String, End>, Choose<(Send<i32, End>, Send<bool, End>)>)>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Choose<Send<String, End>, Offer<Recv<i32, End>, Recv<bool, End>>>>();
    assert_type::<Offer<Recv<String, End>, Choose<Send<i32, End>, Send<bool, End>>>>();
}