# Airbag

Airbag is a Rust library that aims to simplify incident reporting to various 3rd party services. Airbag exposes a simple interface to report incidents with various fields and metadata, as well as catch and report Rust panics. These get reported to a preconfigured backend that takes care of the actual alert/incident sending.

## Features

Airbag currently supports reporting to:

* PagerDuty (via "Events v2" API integration)
* SquadCast (via Webhook integration)


## Getting Started

You can configure Airbag on either a global scope (whole application), in which case it will also catch and report panics, or on a thread-level scope (in which case panics will not get automatically reported). This is done via the `airbag::c

```
let _guard = airbag::configure(airbag::backends::SquadCast::builder().token("token here").build());
```

Or 
```
let _guard = airbag::configure_thread_local(airbag::backends::SquadCast::builder().token("token here").build());
```


## Documentation

Head over to [the full documentation hosted on docs.rs](https://docs.rs/airbag/latest/airbag/) to find out more about Airbag's usage and API
