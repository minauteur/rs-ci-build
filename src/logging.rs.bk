//! Logging helpers module.
//!
//! Defines several helper types and functions for using `slog` in our code.

use errors::LoggerError;

use iron::Request;
use iron::typemap::Key;
use plugin::Extensible;

use slog::{Drain, KV, Logger, OwnedKV};
use slog_async::Async;
use slog_term::{FullFormat, TermDecorator};

use std::cell::RefCell;
use std::mem;
use std::ops::DerefMut;
use std::panic::RefUnwindSafe;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

lazy_static! {
    /// This is the primordial logger. All other loggers sould be spawned from this one.
    pub static ref ROOT_LOGGER: Logger = {
        let term_decorator = TermDecorator::new().build();
        let term_drain = FullFormat::new(term_decorator).build().fuse();
        let term_drain = Async::new(term_drain).build().fuse();

        Logger::root(term_drain.fuse(), o!())
    };

    // Used as an ID for each Request that comes in
    static ref REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Debug, Clone)]
struct RequestLogger(RefCell<Rc<Logger>>);

impl Key for RequestLogger {
    type Value = RequestLogger;
}

/// The `HasLogger` trait fetches an `Rc<Logger>`. `HasLogger` guarantees a `Logger` can be found
/// when a mutable borrow is an option, and attempts to find a `Logger` when an immutable borrow is
/// the only option. Because the `get_logger()` functions return `Rc<Logger>`s, any borrows
/// (mutable or immutable) end immediately on returning, since the `Rc` pointer is owned by
/// the caller. The update functions behave similarly, with the mutable borrow guaranteeing success
/// of the update process, while the immutable borrow returns a `Result<()>`, indicating that there
/// was no logger to update. Note that any `Rc<Logger>` pointers returned by `get_logger()` and
/// `try_get_logger()` may not refer to updated `Loggers` after the `Rc` pointer is returned.
pub trait HasLogger {
    fn get_logger(&mut self) -> Rc<Logger>;
    fn try_get_logger(&self) -> Option<Rc<Logger>>;
    fn update_logger<T>(&mut self, values: OwnedKV<T>)
        where T: 'static + Send + Sync + RefUnwindSafe + KV;
    fn try_update_logger<T>(&self, values: OwnedKV<T>) -> Result<(), LoggerError>
        where T: 'static + Send + Sync + RefUnwindSafe + KV;
}

impl<'a, 'b> HasLogger for Request<'a, 'b> {
    fn get_logger(&mut self) -> Rc<Logger> {
        let req_logger = self.extensions
                             .entry::<RequestLogger>()
                             .or_insert_with(|| {
                                                 let logger =
                                                     ROOT_LOGGER.new(o!("request_id" =>
                                       REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed)));
                                                 RequestLogger(RefCell::new(Rc::new(logger)))
                                             });
        match *req_logger {
            RequestLogger(ref inner) => inner.borrow().clone(),
        }
    }

    fn try_get_logger(&self) -> Option<Rc<Logger>> {
        self.extensions()
            .get::<RequestLogger>()
            .map(|logger| match *logger {
                     RequestLogger(ref inner) => inner.borrow().clone(),
                 })
    }

    fn update_logger<T>(&mut self, values: OwnedKV<T>)
        where T: 'static + Send + Sync + RefUnwindSafe + KV
    {
        let new_logger = self.get_logger().new(values);
        self.extensions
            .insert::<RequestLogger>(RequestLogger(RefCell::new(Rc::new(new_logger))));
    }

    fn try_update_logger<T>(&self, values: OwnedKV<T>) -> Result<(), LoggerError>
        where T: 'static + Send + Sync + RefUnwindSafe + KV
    {
        match self.extensions().get::<RequestLogger>() {
            Some(ref_logger) => {
                match *ref_logger {
                    RequestLogger(ref inner) => {
                        let new_logger = inner.borrow().new(values);
                        mem::replace(inner.borrow_mut().deref_mut(), Rc::new(new_logger));
                        Ok(())
                    },
                }
            },
            None => Err(LoggerError::NoLogger),
        }

    }
}
