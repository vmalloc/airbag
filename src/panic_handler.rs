pub(crate) fn install() {
    let next = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("Airbag: Panic caught: {info}");
        log::info!("Sending panic alert via Airbag...");
        crate::trigger(crate::alert::Alert::build_panic_alert(info));
        next(info);
    }))
}
