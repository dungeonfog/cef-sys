# cef-sys

A systems level crate for the Chromium Embedded Framework. It exposes the C API only, since bindgen cannot handle the C++ API.

This is not meant to be used by applications due to its unsafe nature, a separate abstraction library should wrap this crate first.

# License

This project is licensed under the BSD license, see LICENSE.txt. This is the same as the Chromium Embedded Framework itself.
