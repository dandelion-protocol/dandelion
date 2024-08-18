# Cryptography in Dandelion

Dandelion relies heavily on cryptography at all levels, so the choice of
cryptographic algorithms has a deep impact.

## Cryptographic hash: BLAKE2s

The history of cryptography is littered with broken hash algorithms.  MD5,
though laughably broken today, was once state-of-the-art.  SHA-1 lasted for a
while after that, but it's now nearly as broken as MD5.  SHA-2 still stands in
2024, but like SHA-1 and MD5 it's built on the Merkle-Damgård construction, so
it may be doomed to follow in their footsteps under future attacks.

The hash algorithms submitted to NIST as candidates for SHA-3 are generally
built on a much more stream-cipher-like foundation, versus the S-box structure
of Merkle-Damgård hashes.  In addition to SHA-3 itself, the following
finalists are generally well-regarded:

* Keccak is neat because it naturally works as an XOF (extensible output
  function), i.e. the hash can be as long as you want it to be.  SHA-3 and
  SHAKE are members of the Keccak family.
* BLAKE and its variants have some similarities to Keccak, but the BLAKE core
  is based on ChaCha instead of inventing a new stream cipher core.
* Skein is a bit different and uses Threefish, a block cipher.  However,
  Threefish itself doesn't use S-boxes and actually has more internal
  similarities to Salsa/ChaCha than to AES.

Any of these would be a fairly solid choice.  However, BLAKE2 is vastly more
popular in real-world use than any of the others: Noise, WireGuard, IPFS, etc
etc etc.

That leaves the choice between BLAKE2s (256-bit), and BLAKE2b (512-bit).  For
cryptographic hashes, 256-bit security is generally considered to be more than
enough.

Decision: BLAKE2s

## Signatures: Ed25519

Even more so than cryptographic hash algorithms, public-key cryptosystems are
a vast field of broken hopes and shattered dreams.  RSA is still standing,
which is honestly kind of surprising at this point, but it's just about
impossible to implement RSA without leaking information like a sieve through
side channels (timing, power consumption) due to its reliance on bigint
implementations.  Aside from RSA, however, the only fruitful area left is
elliptic curve cryptography, and even there, most algorithms are hard to
implement perfectly and many have failed spectacularly.  Those generally fall
into two categories: ECDSA, and EdDSA, both of which use elliptic curves over
Galois fields.

EdDSA is a no-brainer over ECDSA:

* EdDSA is much less vulnerable to timing attacks than ECDSA.
* EdDSA signatures are fully deterministic for a given private key and
  message; ECDSA uses random nonces, and repeating a nonce for two different
  messages allows forged signatures for any message.

Two EdDSA parameterizations are standardized:

* Ed25519, using Curve25519 and SHA2-512
* Ed448, using Curve448 and SHAKE256 (a variant of SHA3-256)

Curve448 is regarded as having 224-bit security, i.e. having security
equivalent to a perfect cipher with a 224-bit key, whereas Curve25519 only
offers 128-bit security.  However, 128-bit security is still regarded as being
sufficient for most uses, especially now that Moore's Law has faltered.

(Both would break almost equally badly in a world with sufficiently large
quantum computers, but there are no post-quantum public-key cryptosystems that
I'd trust with my cat pictures, nevermind anything more important, so it
really comes down to one of these two.)

And then there's the hash.  As I mention above, I think it's long past time to
retire hashes based on Merkle-Damgård.  However, SHA-2 is as-yet unbroken for
preimage attacks, and the SHA2-512 variant of SHA-2 has significantly more
headroom for weathering attacks than SHA2-256, so I think it's fine here.

Ed25519 has a huge amount of prior use in existing real-world protocols:
TLSv1.3, SSH, NaCl, and many many others.  Also, implementations of Ed448 are
a lot harder to find than those of Ed25519.

Decision: Ed25519, with a possible future upgrade to Ed448 if it turns out
that we want to rely on signatures made in the distant past remaining
unforgeable.

## Key exchange: X25519

When it comes to key exchange, Diffie-Hellman is the only game in town.  The
older variety, a cousin to RSA, uses prime numbers and therefore is subject to
the same side-channel leaks.  Modern varieties use elliptic curves.

Again, Diffie-Hellman over Curve25519 ("X25519") is the natural choice.  Even
moreso than for EdDSA, ECDH implementations over Curve448 are not widespread.

Decision: X25519

## Ciphers: ChaCha20

There are two serious choices here: AES, or the Salsa/ChaCha family.

AES is very much a product of its time, an era when block ciphers were
believed to be fundamentally more secure than stream ciphers, and building a
stream cipher out of a block cipher using block cipher modes was considered
more reasonable than just using a native stream cipher.

That era has passed.

It turns out that building a secure block cipher mode is pretty much just as
hard as building a secure stream cipher.  The only block cipher modes left
standing are CTR and GCM, with CTR working in almost exactly the same way
Salsa-family ciphers use their nonces, and GCM functioning as an AEAD
construction with no significant security differences from Poly1305.

AES is more complex, needs hardware acceleration to keep up, and is easier to
fumble with regards to making it constant-time and constant-power.
ChaCha20-Poly1305 was added to SSH and to TLSv1.2 for a reason.

Of the Salsa/ChaCha family, ChaCha20 is an improvement on Salsa20 in pretty
much every way, so there is no compelling reason to choose Salsa20 over
ChaCha20.

And regarding the X variants (extended nonces), they are provably no less
secure than the base algorithms.  Having a longer nonce means more ciphertext
can be transmitted before needing a key rotation, but ChaCha20 and has enough
space in the nonce for 64-bit counters, so it's not that important.

Decision: ChaCha20 or XChaCha20.  No point in using anything else.

## AEAD constructions: Poly1305

AEAD constructions are a critical part of modern cipher security, as most
ciphers are vulnerable to ciphertext malleability, i.e. an attacker can change
the plaintext undetectably by manipulating the ciphertext.

With the advent of the Invisible Salamanders family of attacks, it would be
nice to have a well-tested AEAD construction based on a cryptographic hash,
instead of a flimsy one like GMAC or Poly1305.

Alas, such constructions are still being dreamed up.

Instead, we must use either key commitment or plaintext commitment: in any
situation, there must either be no ambiguity about which key is meant to be
used for a given ciphertext, or else a cryptographic hash of the *intended
plaintext* (**not** the ciphertext!) must be provided securely.

With that caveat out of the way, our choice of ChaCha20 leads us inexorably to
Poly1305, as there are no other AEADs commonly paired with it.

Decision: Poly1305.
