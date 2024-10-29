use std::{cell::RefCell, io::Write};

pub struct Logger {
    file: RefCell<Option<std::fs::File>>,
    log_path: String,
}

impl Logger {
    pub fn new(log_path: &str) -> Logger {
        Self {
            log_path: log_path.to_string(),
            file: RefCell::new(
                std::fs::File::options()
                    .append(true)
                    .create(true)
                    .open(log_path)
                    .ok()
            ),
        }
    }

    pub fn write(&self, data: &str) {
        print!("{}", data);
        _ = std::io::stdout().flush();
        let mut file_opt = self.file.borrow_mut();

        let mut file = file_opt.take();

        if let Some(file) = &mut file {
            _ = write!(file, "{}", data);
        }

        *file_opt = file;
    }

    pub fn flush(&self) {
        _ = std::io::stdout().flush();

        let mut file_opt = self.file.borrow_mut();

        std::mem::drop(file_opt.take());

        *file_opt = std::fs::File::options()
            .append(true)
            .open(&self.log_path)
            .ok();
    }
}

macro_rules! log {
    ($logger: expr, $($format_args: tt)*) => {
        ($logger).write(&format!($($format_args)*));
    };
}
