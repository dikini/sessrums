use sessrums::proto::{
    Send, Recv, End, Choose, Offer,
};

#[test]
fn test_simple_choice_projection() {
    // Define a global protocol: RoleA chooses between sending a String or an i32 to RoleB
    // Define a global protocol: RoleA chooses between sending a String or an i32 to RoleB
    // Project for RoleA (chooser)
    // Should be Choose<(Send<String, End>, Send<i32, End>)>
    
    // Project for RoleB (receiver)
    // Should be Offer<(Recv<String, End>, Recv<i32, End>)>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Choose<Send<String, End>, Send<i32, End>>>();
    assert_type::<Offer<Recv<String, End>, Recv<i32, End>>>();
}

#[test]
fn test_simple_offer_projection() {
    // Define a global protocol: RoleB offers a choice to RoleA between receiving a String or an i32
    // Define a global protocol: RoleB offers a choice to RoleA between receiving a String or an i32
    // Project for RoleA (sender)
    // Should be Choose<(Send<String, End>, Send<i32, End>)>
    
    // Project for RoleB (offeree)
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
    // Define a complex global protocol:
    // RoleA sends a bool to RoleB, then
    // RoleA chooses between:
    // 1. Sending a String to RoleB, then ending
    // 2. Receiving an i32 from RoleB, then ending
    // Project for RoleA
    // Should be Send<bool, Choose<(Send<String, End>, Recv<i32, End>)>>
    
    // Project for RoleB
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
    // Define a global protocol with three roles:
    // RoleA sends a bool to RoleB, then
    // RoleB chooses between:
    // 1. Sending a String to RoleC, then ending
    // 2. Sending an i32 to RoleA, then ending
    // Project for RoleA
    // Should be Send<bool, Offer<(End, Recv<i32, End>)>>
    
    // Project for RoleB
    // Should be Recv<bool, Choose<(Send<String, End>, Send<i32, End>)>>
    
    // Project for RoleC
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
    // Define a global protocol with nested choices:
    // RoleA chooses between:
    // 1. Sending a String to RoleB, then ending
    // 2. RoleB chooses between:
    //    a. Sending an i32 to RoleA, then ending
    //    b. Sending a bool to RoleA, then ending
    // Project for RoleA
    // Should be Choose<(Send<String, End>, Offer<(Recv<i32, End>, Recv<bool, End>)>)>
    
    // Project for RoleB
    // Should be Offer<(Recv<String, End>, Choose<(Send<i32, End>, Send<bool, End>)>)>
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<Choose<Send<String, End>, Offer<Recv<i32, End>, Recv<bool, End>>>>();
    assert_type::<Offer<Recv<String, End>, Choose<Send<i32, End>, Send<bool, End>>>>();
}