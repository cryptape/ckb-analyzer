pub fn channel<T: Send + 'static>(
    cap: usize,
) -> (
    crossbeam::channel::Sender<T>,
    ckb_async_runtime::tokio::sync::mpsc::Receiver<T>,
) {
    let (crossbeam_sender, crossbeam_receiver) = crossbeam::channel::bounded(cap);
    let (tokio_sender, tokio_receiver) = ckb_async_runtime::tokio::sync::mpsc::channel(cap);

    ::std::thread::spawn(move || {
        while let Ok(data) = crossbeam_receiver.recv() {
            if let Err(_err) = tokio_sender.blocking_send(data) {
                break;
            }
        }
    });

    (crossbeam_sender, tokio_receiver)
}
