# Simple Network Protocols

Implementations of some simple network protocols/services:

|            Protocol | Transport |  Port |   Standard |     Feature |
| ------------------- | --------- | ----- | ---------- | ----------- |
|                Echo |   TCP/UDP |     7 |  [RFC 862] |      `echo` |
|             Discard |   TCP/UDP |     9 |  [RFC 863] |   `discard` |
|        Active Users |   TCP/UDP |    11 |  [RFC 866] |    `active` |
|             Daytime |   TCP/UDP |    13 |  [RFC 867] |   `daytime` |
| Character Generator |   TCP/UDP |    19 |  [RFC 864] |   `chargen` |
|                Time |   TCP/UDP |    37 |  [RFC 868] |      `time` |

[RFC 862]: https://datatracker.ietf.org/doc/html/rfc862
[RFC 863]: https://datatracker.ietf.org/doc/html/rfc863
[RFC 866]: https://datatracker.ietf.org/doc/html/rfc866
[RFC 867]: https://datatracker.ietf.org/doc/html/rfc867
[RFC 864]: https://datatracker.ietf.org/doc/html/rfc864
[RFC 868]: https://datatracker.ietf.org/doc/html/rfc868

All features are enabled by default.

## Implementation notes

There is a "fake" filesystem embedded into the binary by the build script, which is used for protocols that require a file system or similar as data.
The file system is read-only, and because it is embedded into the server binary, does not require runtime file system access.
Using the real file system of the host computer is not supported.

Active Users sends a list of random, fictitious users.

## Tests

Run all tests with `cargo test` (or [`cargo nextest run`](https://nexte.st/)) while the server is running.
If you get an error about file removal failure when starting the test, try running either the server or the tests with `--release`.

The code inside `/src` contains unit tests where appropriate, and all protocols have integration tests in `/tests`.
Generic integration tests *should* work for all RFC-compliant servers, though where the relevent standard is ambiguous, the tests often use a strict interpretation.
Also keep in mind that the implementations and tests here are of early version of basic protocols, without any modern updates.
Integration tests in files ending with `-spspecific` contain simple-protocols-specific assertions that enforce stricter-than-standardized or nonstandardized behaviour that may only be applicable to this project.

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) (SPDX `Apache-2.0`) or the [MIT license](./LICENSE-MIT) (SPDX `MIT`) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
