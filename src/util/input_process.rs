use signal_hook::consts::signal;

use crate::context::JoshutoContext;
use crate::history::DirectoryHistory;
use crate::io::{FileOp, IOWorkerProgress};
use crate::ui;
use crate::util::event::JoshutoEvent;
use crate::util::format;

pub fn process_noninteractive(event: JoshutoEvent, context: &mut JoshutoContext) {
    match event {
        JoshutoEvent::IOWorkerProgress(res) => process_worker_progress(context, res),
        JoshutoEvent::IOWorkerResult(res) => process_finished_worker(context, res),
        JoshutoEvent::Signal(signal::SIGWINCH) => {}
        _ => {}
    }
}

pub fn process_worker_progress(context: &mut JoshutoContext, res: IOWorkerProgress) {
    context.set_worker_progress(res);
    context.update_worker_msg();
}

pub fn process_finished_worker(
    context: &mut JoshutoContext,
    res: std::io::Result<IOWorkerProgress>,
) {
    let observer = context.remove_job().unwrap();
    let options = context.config_ref().sort_option.clone();
    for tab in context.tab_context_mut().iter_mut() {
        let _ = tab.history_mut().reload(observer.dest_path(), &options);
        let _ = tab.history_mut().reload(observer.src_path(), &options);
    }
    observer.join();
    match res {
        Ok(progress) => {
            let op = match progress.kind() {
                FileOp::Copy => "copied",
                FileOp::Cut => "moved",
            };
            let size_str = format::file_size_to_string(progress.processed());
            let msg = format!(
                "successfully {} {} items ({})",
                op,
                progress.len(),
                size_str
            );
            context.push_msg(msg);
        }
        Err(e) => {
            let msg = format!("{}", e);
            context.push_msg(msg);
        }
    }
}
