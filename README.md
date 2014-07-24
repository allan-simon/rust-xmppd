##Rust-xmppd

Tenative to implement a basic XMPP/Jabber server in Rust

## Build instruction

for this you need `cargo` and the last nightly release of Rust
once you got that simply hit

    cargo build


and you're ready to go

## Why yet an other XMPP server ?

First be reassured, I know ejabberd, openfire, prosody works pretty well
I have been working for a company project on both openfire and prosody for
now a year.

Here the motivation are the following:

  * Fun (I want to learn Rust it's enough by itself?)
  * Openfire is nice but does not support several domain (and probably never
    will
  * Openfire is over architectured and though seems well organized, the code
    seems pretty hermetic for people to dive into it
  * Openfire is in Java and I dont really appreciate the language itself
  * Prosody on the other hand is pretty well architectured but single threaded
  * Prosody does not seems to have any "custering" features planned yet
  * Prosody though Lua is pretty "fresh" and nice to work with, compile-time
    error and strong typing system are also great for "not so great"
    programmers

Aware of the advatanges and weak points, both in term of architecture and
features the long term goal is to achieve

  * A server that support multi domains
  * multi threaded because we didnt bought all these CPUs for nothing
  * a plugin system with as much things as possible as external plugin
  * a tiny core
  * fancy stuff to make it scale (because your boss is not going to let
    you use this if you cant deploy in da cloud (tm) and scale it on
    one million node)
  * a rock solid code base in a rock solid language


##What is working yet


You should be able to configurate username and passwords in `data/login.json`
login and exchanging messages should work for `@localhost` account using pidgin

presences are also somewhat exchanged

##How the data flow

TODO

##What is planned

In order of importance for me (which should more or less follow the order of
implementation)

  * use an actual xml parser (rustyxml) to parse the XMPP stream ,currently
    we're doing old school string splitting which was good only for
    "check it works"
  * encapsulate the TCP stream into a XMPPServerStream object, the way
    rust-xmpp project do (TODO: add link to this project), but they do it
    for client side which is not adapted for server side

TODO
