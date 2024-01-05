# Airbag

Airbag is a rust library for handling errors and panics using 3rd party services.


## Pager Duty
Airbag supports Pager Duty service. It can be configured by:
```
let pager_duty_token = "token";  // Should be set to your real token
 let _pd = airbag::configure_thread_local(airbag::backends::PagerDuty::builder().token("token").build());

```
