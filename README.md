# cef-sys

A systems level crate for the Chromium Embedded Framework. It exposes the C API only, since bindgen cannot handle the C++ API.

This is not meant to be used by applications due to its unsafe nature, a separate abstraction library should wrap this crate first.

# Usage

Pre-built CEF binaries can be acquired from here: http://opensource.spotify.com/cefbuilds/index.html. 

Valid CEF release binaries must be present in your system's library search path. On Windows, there are a couple places you can put the binaries to accomplish this:
- Your working directory
- A folder in the `LIB` path.

The second option should be easiest to manage, since it doesn't involve cluttering up the source tree with binary files. Extract the files in the `Release` folder of the CEF distribution to whichever folder you choose.

The CEF distribution comes with a `icudtl.dat` file. Extract that file to whichever folder holds the CEF libraries.

# License

This project is licensed under the BSD license, see LICENSE.txt. This is the same as the Chromium Embedded Framework itself.
