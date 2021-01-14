use jsonrpc_server_utils::tokio::prelude::*;

// Just transform tokio 1.0 channel to crossbeam channel
// I don't know how to transform into tokio 2.0 channel
pub fn forward_tokio1_channel<T>(
    tokio1_receiver: jsonrpc_server_utils::tokio::sync::mpsc::Receiver<T>,
) -> crossbeam::channel::Receiver<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = crossbeam::channel::bounded(100);
    ::std::thread::spawn(move || {
        tokio1_receiver
            .for_each(|item| Ok(sender.send(item).unwrap()))
            .wait()
            .unwrap()
    });
    receiver
}
