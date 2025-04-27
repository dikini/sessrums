//! Macro implementation for defining global protocols.
//!
//! This crate provides a macro for defining global protocols using a sequence
//! diagram-inspired syntax, making it easier to create and understand complex
//! communication protocols.

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, Ident, Token, parse::{Parse, ParseStream}, Result, Error, braced, bracketed, parenthesized};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{Type, TypePath};
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::collections::HashMap;

/// A macro for defining global protocols using a sequence diagram-inspired syntax.
///
/// This macro allows you to define global protocols in a more intuitive way,
/// similar to how you would draw a sequence diagram. It supports message passing,
/// branching and choice, recursion, and composition (sequential and parallel).
///
/// # Examples
///
/// ## Simple Message Passing
///
/// ```rust
/// global_protocol! {
///     protocol PingPong {
///         Client -> Server: String;
///         Server -> Client: String;
///     }
/// }
/// ```
///
/// This generates the equivalent of:
///
/// ```rust
/// type PingPong = GSend<String, Client, Server, GRecv<String, Server, Client, GEnd>>;
/// ```
///
/// ## Branching and Choice
///
/// ```rust
/// global_protocol! {
///     protocol Authentication {
///         Client -> Server: Credentials;
///         choice at Server {
///             option Success {
///                 Server -> Client: Token;
///                 Client -> Server: Request;
///                 Server -> Client: Response;
///             }
///             option Failure {
///                 Server -> Client: ErrorMessage;
///             }
///         }
///     }
/// }
/// ```
///
/// ## Recursion
///
/// ```rust
/// global_protocol! {
///     protocol ChatSession {
///         rec ChatLoop {
///             choice at Client {
///                 option SendMessage {
///                     Client -> Server: Message;
///                     Server -> Client: Confirmation;
///                     continue ChatLoop;
///                 }
///                 option Quit {
///                     Client -> Server: Disconnect;
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Sequential Composition
///
/// ```rust
/// global_protocol! {
///     protocol Login {
///         Client -> Server: Credentials;
///         Server -> Client: Token;
///     }
///
///     protocol DataExchange {
///         Client -> Server: Request;
///         Server -> Client: Response;
///     }
///
///     protocol ComposedProtocol {
///         seq {
///             include Login;
///             include DataExchange;
///         }
///     }
/// }
/// ```
///
/// ## Parallel Composition
///
/// ```rust
/// global_protocol! {
///     protocol ParallelOperations {
///         par {
///             Client -> Server: Request;
///             Server -> Client: Response;
///         } and {
///             Client -> Logger: LogEntry;
///             Logger -> Monitor: Notification;
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn global_protocol(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProtocolDefinitions);
    let expanded = input.expand();
    TokenStream::from(expanded)
}

/// Represents a set of protocol definitions.
struct ProtocolDefinitions {
    protocols: Vec<ProtocolDefinition>,
}

impl Parse for ProtocolDefinitions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut protocols = Vec::new();
        
        while !input.is_empty() {
            protocols.push(input.parse()?);
        }
        
        Ok(ProtocolDefinitions { protocols })
    }
}

impl ProtocolDefinitions {
    fn expand(&self) -> TokenStream2 {
        let protocol_defs = self.protocols.iter().map(|p| p.expand());
        
        quote! {
            #(#protocol_defs)*
        }
    }
}

/// Represents a single protocol definition.
struct ProtocolDefinition {
    protocol_keyword: Ident,
    name: Ident,
    brace_token: Brace,
    body: ProtocolBody,
}

impl Parse for ProtocolDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let protocol_keyword = input.parse()?;
        let name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let body = content.parse()?;
        
        Ok(ProtocolDefinition {
            protocol_keyword,
            name,
            brace_token,
            body,
        })
    }
}

impl ProtocolDefinition {
    fn expand(&self) -> TokenStream2 {
        let name = &self.name;
        let body = self.body.expand();
        
        quote! {
            type #name = #body;
        }
    }
}

/// Represents the body of a protocol definition.
enum ProtocolBody {
    Interactions(Vec<Interaction>),
    SequentialComposition {
        seq_keyword: Ident,
        brace_token: Brace,
        protocols: Vec<IncludeProtocol>,
    },
    ParallelComposition {
        par_keyword: Ident,
        first_brace_token: Brace,
        first_body: Box<ProtocolBody>,
        and_keyword: Ident,
        second_brace_token: Brace,
        second_body: Box<ProtocolBody>,
    },
}

impl Parse for ProtocolBody {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) && input.peek2(Brace) {
            let keyword: Ident = input.parse()?;
            
            if keyword == "seq" {
                let content;
                let brace_token = braced!(content in input);
                let mut protocols = Vec::new();
                
                while !content.is_empty() {
                    protocols.push(content.parse()?);
                }
                
                return Ok(ProtocolBody::SequentialComposition {
                    seq_keyword: keyword,
                    brace_token,
                    protocols,
                });
            } else if keyword == "par" {
                let first_content;
                let first_brace_token = braced!(first_content in input);
                let first_body = Box::new(first_content.parse()?);
                
                let and_keyword: Ident = input.parse()?;
                if and_keyword != "and" {
                    return Err(Error::new(and_keyword.span(), "Expected 'and' keyword"));
                }
                
                let second_content;
                let second_brace_token = braced!(second_content in input);
                let second_body = Box::new(second_content.parse()?);
                
                return Ok(ProtocolBody::ParallelComposition {
                    par_keyword: keyword,
                    first_brace_token,
                    first_body,
                    and_keyword,
                    second_brace_token,
                    second_body,
                });
            }
        }
        
        let mut interactions = Vec::new();
        
        while !input.is_empty() {
            interactions.push(input.parse()?);
        }
        
        Ok(ProtocolBody::Interactions(interactions))
    }
}

impl ProtocolBody {
    fn expand(&self) -> TokenStream2 {
        match self {
            ProtocolBody::Interactions(interactions) => {
                let mut current = quote! { GEnd };
                
                for interaction in interactions.iter().rev() {
                    current = interaction.expand(current);
                }
                
                current
            },
            ProtocolBody::SequentialComposition { protocols, .. } => {
                if protocols.is_empty() {
                    return quote! { GEnd };
                }
                
                let mut iter = protocols.iter();
                let first = iter.next().unwrap();
                let first_expanded = first.expand();
                
                let mut current = first_expanded;
                
                for protocol in iter {
                    let next = protocol.expand();
                    current = quote! { GSeq<#current, #next> };
                }
                
                current
            },
            ProtocolBody::ParallelComposition { first_body, second_body, .. } => {
                let first_expanded = first_body.expand();
                let second_expanded = second_body.expand();
                
                quote! { GPar<#first_expanded, #second_expanded> }
            },
        }
    }
}

/// Represents an include statement for protocol composition.
struct IncludeProtocol {
    include_keyword: Ident,
    protocol_name: Ident,
    semicolon_token: Token![;],
}

impl Parse for IncludeProtocol {
    fn parse(input: ParseStream) -> Result<Self> {
        let include_keyword: Ident = input.parse()?;
        if include_keyword != "include" {
            return Err(Error::new(include_keyword.span(), "Expected 'include' keyword"));
        }
        
        let protocol_name = input.parse()?;
        let semicolon_token = input.parse()?;
        
        Ok(IncludeProtocol {
            include_keyword,
            protocol_name,
            semicolon_token,
        })
    }
}

impl IncludeProtocol {
    fn expand(&self) -> TokenStream2 {
        let protocol_name = &self.protocol_name;
        
        quote! { #protocol_name }
    }
}

/// Represents an interaction in a protocol.
enum Interaction {
    MessagePassing {
        from: Ident,
        arrow: Token![->],
        to: Ident,
        colon: Token![:],
        message_type: Type,
        semicolon: Token![;],
    },
    Choice {
        choice_keyword: Ident,
        at_keyword: Ident,
        role: Ident,
        brace_token: Brace,
        options: Vec<Option>,
    },
    Recursion {
        rec_keyword: Ident,
        label: Ident,
        brace_token: Brace,
        body: Vec<Interaction>,
    },
    Continue {
        continue_keyword: Ident,
        label: Ident,
        semicolon: Token![;],
    },
}

impl Parse for Interaction {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        
        if lookahead.peek(Ident) && input.peek2(Token![->]) {
            let from = input.parse()?;
            let arrow = input.parse()?;
            let to = input.parse()?;
            let colon = input.parse()?;
            let message_type = input.parse()?;
            let semicolon = input.parse()?;
            
            Ok(Interaction::MessagePassing {
                from,
                arrow,
                to,
                colon,
                message_type,
                semicolon,
            })
        } else if lookahead.peek(Ident) {
            let keyword: Ident = input.parse()?;
            
            if keyword == "choice" {
                let at_keyword: Ident = input.parse()?;
                if at_keyword != "at" {
                    return Err(Error::new(at_keyword.span(), "Expected 'at' keyword"));
                }
                
                let role = input.parse()?;
                let content;
                let brace_token = braced!(content in input);
                let mut options = Vec::new();
                
                while !content.is_empty() {
                    options.push(content.parse()?);
                }
                
                Ok(Interaction::Choice {
                    choice_keyword: keyword,
                    at_keyword,
                    role,
                    brace_token,
                    options,
                })
            } else if keyword == "rec" {
                let label = input.parse()?;
                let content;
                let brace_token = braced!(content in input);
                let mut body = Vec::new();
                
                while !content.is_empty() {
                    body.push(content.parse()?);
                }
                
                Ok(Interaction::Recursion {
                    rec_keyword: keyword,
                    label,
                    brace_token,
                    body,
                })
            } else if keyword == "continue" {
                let label = input.parse()?;
                let semicolon = input.parse()?;
                
                Ok(Interaction::Continue {
                    continue_keyword: keyword,
                    label,
                    semicolon,
                })
            } else {
                Err(Error::new(keyword.span(), "Expected 'choice', 'rec', or 'continue' keyword"))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl Interaction {
    fn expand(&self, continuation: TokenStream2) -> TokenStream2 {
        match self {
            Interaction::MessagePassing { from, to, message_type, .. } => {
                quote! {
                    GSend<#message_type, #from, #to, #continuation>
                }
            },
            Interaction::Choice { role, options, .. } => {
                if options.len() == 1 {
                    let option = &options[0];
                    let option_body = option.expand();
                    
                    quote! {
                        GChoice<#role, (#option_body,)>
                    }
                } else {
                    let option_bodies = options.iter().map(|opt| opt.expand());
                    
                    quote! {
                        GChoice<#role, (#(#option_bodies),*)>
                    }
                }
            },
            Interaction::Recursion { label, body, .. } => {
                let label_type = format_ident!("{}Label", label);
                
                let mut current = quote! { GEnd };
                
                for interaction in body.iter().rev() {
                    current = interaction.expand(current);
                }
                
                quote! {
                    {
                        struct #label_type;
                        GRec<#label_type, #current>
                    }
                }
            },
            Interaction::Continue { label, .. } => {
                let label_type = format_ident!("{}Label", label);
                
                quote! {
                    GVar<#label_type>
                }
            },
        }
    }
}

/// Represents an option in a choice interaction.
struct Option {
    option_keyword: Ident,
    name: Ident,
    brace_token: Brace,
    body: Vec<Interaction>,
}

impl Parse for Option {
    fn parse(input: ParseStream) -> Result<Self> {
        let option_keyword: Ident = input.parse()?;
        if option_keyword != "option" {
            return Err(Error::new(option_keyword.span(), "Expected 'option' keyword"));
        }
        
        let name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let mut body = Vec::new();
        
        while !content.is_empty() {
            body.push(content.parse()?);
        }
        
        Ok(Option {
            option_keyword,
            name,
            brace_token,
            body,
        })
    }
}

impl Option {
    fn expand(&self) -> TokenStream2 {
        let mut current = quote! { GEnd };
        
        for interaction in self.body.iter().rev() {
            current = interaction.expand(current);
        }
        
        current
    }
}