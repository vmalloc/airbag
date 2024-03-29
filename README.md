# Airbag

Airbag is a Rust library that aims to simplify incident reporting to various 3rd party services. Airbag exposes a simple interface to report incidents with various fields and metadata, as well as catch and report Rust panics. These get reported to a preconfigured backend that takes care of the actual alert/incident sending.

## Features

* Support for multiple configurable backends
* Middleware support, allowing applications to customize emitted alerts before they are being sent
* Supports shortcuts for handling `Result`s with propagation to alerts
* Catches and reports panics (only when configured globally)

## Getting Started

You can configure Airbag on either a global scope (whole application), in which case it will also catch and report panics, or on a thread-level scope (in which case panics will not get automatically reported). This is done via the `airbag::configure` and `airbag::configure_thread_local`:

```
let _guard = airbag::configure(airbag::backends::SquadCast::builder().region("eu").token("token here").build());
```

Or
```
let _guard = airbag::configure_thread_local(airbag::backends::SquadCast::builder().region("eu").token("token here").build());
```

After configuring Airbag, you can emit an alert by:
```
airbag::alert::Alert::builder()
    .title("Alert title")
    .dedup_key("alert-dedup-key")
    .trigger();
```

## Documentation

Head over to [the full documentation hosted on docs.rs](https://docs.rs/airbag/latest/airbag/) to find out more about Airbag's usage and API
