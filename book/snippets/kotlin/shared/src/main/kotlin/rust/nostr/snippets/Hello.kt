package rust.nostr.snippets

// ANCHOR: full
import rust.nostr.sdk.*

suspend fun hello() {
    // ANCHOR: client
    //val keys = Keys.generate()
    //val client = Client(signer = keys) // TODO: uncomment when fixed
    // ANCHOR_END: client

    // ANCHOR: connect
    //client.addRelay("wss://relay.damus.io")

    //client.connect()
    // ANCHOR_END: connect

    // ANCHOR: publish
    //val builder = EventBuilder.textNote("Hello, rust-nostr!", listOf())
    //client.sendEventBuilder(builder)
    // ANCHOR_END: publish
}
// ANCHOR_END: full
