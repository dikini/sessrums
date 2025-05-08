# MPST DSL Syntax and Grammar

This document formally defines the syntax and grammar for the Multiparty Session Types (MPST) Domain-Specific Language (DSL). The DSL provides a Mermaid-like syntax for defining multiparty session type protocols, making them more intuitive and readable while being precise enough to be parsed by a procedural macro.

## 1. Introduction

The MPST DSL allows users to define multiparty session type protocols in a concise, readable format. The DSL is designed to be:

- **Intuitive**: The syntax resembles sequence diagrams, making it easy to understand the flow of communication
- **Precise**: The grammar is well-defined and unambiguous, enabling accurate parsing
- **Expressive**: The DSL supports all the features of multiparty session types, including message passing, choice, and recursion
- **Readable**: The syntax is designed to be easily read and written by humans

The DSL is processed at compile time by a procedural macro, which transforms the textual protocol definitions into Rust code that constructs the `GlobalInteraction` enum structure.

## 2. Lexical Elements

### 2.1 Identifiers

Identifiers are used for role names, message types, and recursion labels.

```ebnf
Identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
```

Identifiers must start with a letter or underscore, followed by any number of letters, digits, or underscores.

### 2.2 Keywords

The following keywords are reserved and cannot be used as identifiers:

```
protocol, participant, as, choice, at, option, or, rec, continue, end
```

### 2.3 Symbols

The following symbols have special meaning in the DSL:

```
{ } ; : -> ,
```

### 2.4 Whitespace and Comments

Whitespace (spaces, tabs, newlines) is ignored except as a separator between tokens.

Comments can be:
- Line comments: `// comment text`
- Block comments: `/* comment text */`

Comments are treated as whitespace and ignored by the parser.

## 3. Grammar

The grammar is defined in Extended Backus-Naur Form (EBNF).

### 3.1 Protocol Definition

```ebnf
Protocol ::= 'protocol' Identifier '{' ParticipantList InteractionList '}'

ParticipantList ::= Participant*

Participant ::= 'participant' Identifier ('as' Identifier)? ';'

InteractionList ::= Interaction*

Interaction ::= MessageInteraction
              | ChoiceInteraction
              | RecursionInteraction
              | ContinueInteraction
              | EndInteraction
```

### 3.2 Message Interaction

```ebnf
MessageInteraction ::= Identifier '->' Identifier ':' MessageType ';'

MessageType ::= Identifier ('::' Identifier)* ('<' GenericParams '>')?

GenericParams ::= MessageType (',' MessageType)*
```

### 3.3 Choice Interaction

```ebnf
ChoiceInteraction ::= 'choice' 'at' Identifier '{' BranchList '}'

BranchList ::= Branch ('or' Branch)*

Branch ::= ('option' Identifier)? '{' InteractionList '}'
```

### 3.4 Recursion Interaction

```ebnf
RecursionInteraction ::= 'rec' Identifier '{' InteractionList '}'

ContinueInteraction ::= 'continue' Identifier ';'
```

### 3.5 End Interaction

```ebnf
EndInteraction ::= 'end' ';'
```

## 4. Syntactic Restrictions

### 4.1 Well-Formedness

1. All roles referenced in interactions must be declared as participants.
2. All recursion labels referenced in `continue` statements must be defined with a corresponding `rec` block.
3. Recursion labels must be unique within their scope.
4. Message types must be valid Rust types.
5. In a choice interaction, the first message in each branch must be sent by the deciding role.

### 4.2 Nesting and Scoping

1. Choice branches can contain any interaction, including nested choices and recursion.
2. Recursion blocks can contain any interaction, including nested recursion.
3. Recursion labels are scoped to their containing block and any nested blocks.
4. A `continue` statement can only reference a recursion label that is in scope.

## 5. Examples

### 5.1 Simple Protocol

```
protocol PingPong {
    participant Client;
    participant Server;
    
    Client -> Server: String;
    Server -> Client: String;
    end;
}
```

### 5.2 Protocol with Choice

```
protocol LoginProtocol {
    participant Client;
    participant Server;
    
    Client -> Server: Credentials;
    
    choice at Server {
        option Success {
            Server -> Client: LoginSuccess;
            end;
        }
        or {
            Server -> Client: LoginFailure;
            end;
        }
    }
}
```

### 5.3 Protocol with Recursion

```
protocol ChatProtocol {
    participant Client;
    participant Server;
    
    rec ChatLoop {
        choice at Client {
            option SendMessage {
                Client -> Server: ChatMessage;
                Server -> Client: Acknowledgment;
                continue ChatLoop;
            }
            or {
                Client -> Server: Disconnect;
                end;
            }
        }
    }
}
```

### 5.4 Complex Protocol with Multiple Participants

```
protocol OnlineStore {
    participant Customer;
    participant Store;
    participant Warehouse;
    participant ShippingService;
    
    // Browse products
    rec BrowseLoop {
        Customer -> Store: BrowseRequest;
        Store -> Customer: ProductList;
        
        choice at Customer {
            option ContinueBrowsing {
                continue BrowseLoop;
            }
            or {
                // Add to cart
                rec CartLoop {
                    Customer -> Store: AddToCart;
                    Store -> Customer: CartUpdated;
                    
                    choice at Customer {
                        option ContinueShopping {
                            continue BrowseLoop;
                        }
                        or {
                            // Checkout process
                            Customer -> Store: Checkout;
                            Store -> Customer: OrderSummary;
                            
                            choice at Customer {
                                option Confirm {
                                    Customer -> Store: PaymentInfo;
                                    Store -> Customer: PaymentConfirmation;
                                    
                                    // Order fulfillment
                                    Store -> Warehouse: OrderDetails;
                                    Warehouse -> Store: InventoryConfirmation;
                                    Warehouse -> ShippingService: ShipmentRequest;
                                    ShippingService -> Customer: TrackingInfo;
                                    ShippingService -> Store: ShipmentConfirmation;
                                    
                                    end;
                                }
                                or {
                                    Customer -> Store: CancelOrder;
                                    continue BrowseLoop;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## 6. Mapping to Rust Types

The DSL is transformed into Rust code that constructs the `GlobalInteraction` enum structure. The mapping is as follows:

| DSL Construct | Rust Type |
|---------------|-----------|
| `A -> B: T;` | `GlobalInteraction::Message { from: "A", to: "B", msg: PhantomData<T>, cont: ... }` |
| `choice at A { ... }` | `GlobalInteraction::Choice { decider: "A", branches: [...] }` |
| `rec X { ... }` | `GlobalInteraction::Rec { label: "X", body: ... }` |
| `continue X;` | `GlobalInteraction::Var { label: "X" }` |
| `end;` | `GlobalInteraction::End` |

## 7. Error Handling

The DSL parser provides detailed error messages for syntax and semantic errors, including:

1. Syntax errors (e.g., missing semicolons, unmatched braces)
2. Undefined participants
3. Undefined recursion labels
4. Duplicate recursion labels
5. Invalid message types
6. Invalid choice syntax (e.g., first message not sent by the deciding role)

Error messages include the location of the error and suggestions for fixing it.

## 8. Conclusion

This document has defined the formal syntax and grammar for the MPST DSL. The DSL provides a concise, readable way to define multiparty session type protocols, which are then transformed into Rust code at compile time.

The syntax is designed to be intuitive and resemble sequence diagrams, making it easy to understand the flow of communication. At the same time, the grammar is well-defined and unambiguous, enabling accurate parsing and transformation.

The DSL supports all the features of multiparty session types, including message passing, choice, and recursion, and provides detailed error messages for syntax and semantic errors.