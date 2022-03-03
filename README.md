# azure-embedded-sdk-rs-example
This example uses a Rust friendly interface to the [Embedded SDK for C](https://github.com/Azure/azure-sdk-for-c). It consist of three crates with this being the topmost level. The other two are:
* [azure-embedded-sdk-rs](https://github.com/markrad/azure-embedded-sdk-rs) which implements the Rust API on top of
* [azure-embedded-sdk-sys](https://github.com/markrad/azure-embedded-sdk-rs) which will build the Embedded SDK for C and generate a bindgen interface to it.

This example uses the "high level" Rust functions interface. These functions will use Result<> return values where applicable and the requested data will be returned from the functions in a buffer allocated by the function. 

The alternative set of functions, prefixed with ll_, are more inline with the goals of the Embedded SDK for C. The caller is required to pass a buffer to the function thus these functions will not perform any allocations on the heap. Return values will be those returned from the lower level crate. the Result<> construct is not used.
## Building
In order for the Rust utility bindgen to work you will need to install the Clang libraries. These are used to parse the C headers in order to convert them into Rust. 

**Note:** I encountered issues using Clang 6.0. It was unable to resolve the location of *stdbool.h*. Rather than debug that, I switched to Clang 10.0 which resolved the problem.

This sample uses the paho-mqtt crate for MQTT services which, in turn, will use the OpenSSL libraries for TLS negotiation. To run the example you will need to have OpenSSL installed on your host. In some instances, typically Windows, you will also need to pass the location of your OpenSSL librarys to the build step.
This can be accomplished in one of two manners:
1) Add a RUSTFLAGS environment variable that passes a -L flag that identifies the location of the OpenSSL libriaies. However, this may cause problems on Windows.
2) If the path to your OpenSSL libraries contains a blank character such as "C:\Program Files\..." then option 1) will not work. This is due to bug in cargo's parsing of the variable. You will need to create a file at ".\\.cargo\\config.toml" to indicate where the OpenSSL libraries are. For example:
```
[build]
rustflags = ["-L", "C:\\Program Files\\OpenSSL-Win64\\lib" ]
```
## Running
Running requires two environment variables. These are:
* AZ_IOT_CONNECTION_STRING which should be set to the connection string of the device you are simulating
* AZ_IOT_ROOT_CERTIFICATE which should be set to the full or relative path of the (currently) Baltimore CyberTrust root certificate. This file is included in the repository.

Logging is performed by env_logger. You might need a RUST_LOG environment variable to see the output.

With no changes, the example will run forever.

## Implemented
* Connect to an IoT hub using SAS authentication
* Send telemetry to the connected IoT hub
* Receive cloud to device messages from the connected IoT hub
* Receive and respond to direct methods from the connected IoT hub
* Refresh of SAS token and reconnection upon SAS token's expiry
## Not tested but may work
* Authentication using X.509. This sample does not demonstrate that but OpenSSL will support it hence it should work
## Not Implemented
* Connection retry timing
* Use of device provisioning service
## Known Issues
* As noted above, some versions of Clang may cause the build of the azure-embedded-sdk-sys to fail. Try installing a later version of Clang.
