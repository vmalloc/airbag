# Changelog

## v4.0 
* Refactored middleware installation entry points, making it easier to conditionally install middleware
* Expose AirbagResult as `airbag::AirbagResult`

## v3.0
* Added mandatory region to SquadCast backends

## v2.0
* Major API overhaul, move to configurable backends (currently supporting PagerDuty and SquadCast)
* Add middleware support (currently for prefixing summaries and dedup keys)
* Add integration tests
* Update dependencies

## v1.0
* Compatibility with PagerDuty Events v2 API