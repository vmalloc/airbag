# Airbag

Airbag is a rust library for handling errors and panics using 3rd party services.


## Pager Duty
Airbag supports Pager Duty service. It can be configured by:
```
let pager_duty_token = "token";  // Should be set to your real token
let _guard = airbag::configure_pagerduty(pager_duty_token, None, None, None);
```
