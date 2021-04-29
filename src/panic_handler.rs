pub(crate) fn install() {
    let next = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        crate::dispatch::HUB
            .read()
            .dispatch_and_block(|| crate::alerts::generate_panic_alert(info));
        next(info);
    }))
}
