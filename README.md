# pron

`pron` is a tiny command line tool that allows encoding data from JSON to protobuf and vice versa.

## Install

`protoc` is required, which can be installed by following [these instructions](https://grpc.io/docs/protoc-installation/). Verify that `protoc` is available by running:

```sh
protoc --version
```

No pre-compiled binaries are offered at the moment, so you will have to compile `pron` locally. A Rust toolchain is required which can be installed by following [these instructions](https://www.rust-lang.org/tools/install). If a Rust toolchain is available, you can install `pron` by running:

```sh
cargo install --git https://github.com/mrzenioszeniou/pron
```

## Usage

If you have a `foobar.proto` file that looks like this:

```proto
syntax = "proto3";

package foobar;

message Foobar {
    string foo = 1;
    int64 bar = 2;
}
```

You can encode a JSON representation of `Foobar` using its protobuf definition, like so:

```sh
echo '{ "foo":"42","bar":42 } ' | pron  --proto foobar.proto --message foobar.Foobar encode
```

You can decode a `Foobar` message that is protobuf-encoded, like so:

```sh
echo CgI0MhAq | base64 -d | pron --proto foobar.proto --message foobar.Foobar decode
```

## Contributing

All contributions are welcome.

## License

MIT © [Zenios Zeniou](LICENSE)
