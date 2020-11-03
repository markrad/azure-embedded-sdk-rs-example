# azure-embedded-sdk-rs-example
Example using the azure-embedded-sdk-rs crate.

This is currently a very basic sample. It decodes a connection string, generates the MQTT parameters, connects to the IoT hub and sends 30 messages to it. 

To run this you will need the OpenSSL libraries installed and you will need to pass their location to the build step. This can be accomplished in one of two manners:
1) Add a RUSTFLAGS environment variable that passes a -L flag that identifies the location of the OpenSSL libriaies. However, this may cause problems on Windows.
2) If the path to your OpenSSL libraries contains a blank character such as "C:\Program Files\..." then option 1) will not work. This is due to bug in cargo's parsing of the variable. You will need to create a file at ".\\.cargo\\config.toml" to indicate where the OpenSSL libraries are. For example:
```
[build]
rustflags = ["-L", "C:\\Program Files\\OpenSSL-Win64\\lib" ]
```
