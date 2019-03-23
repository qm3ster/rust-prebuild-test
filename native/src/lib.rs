#![feature(futures_api, async_await, await_macro)]

#[macro_use]
extern crate neon;

#[macro_use]
extern crate tokio;

use neon::prelude::*;
use std::cell::Cell;
use tokio::prelude::*;
use tokio::sync::{mpsc, oneshot};
use tokio_serial::{Serial, SerialPort};

struct BackroundTask {
    rx: Cell<Option<mpsc::Receiver<Command>>>,
    path: String,
}
impl Task for BackroundTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        println!("PERFORM");
        let rx = self.rx.take().unwrap();
        let path = self.path.clone();
        tokio::run_async(
            async {
                let mut rx = rx;
                println!("INNER");
                println!("Gonna open {}", path);
                let sp = Serial::from_path(path, &Default::default()).unwrap();
                println!("Opened {:?}", sp.name());
                let (srx, mut stx) = sp.split();
                let (ctx, crx) = oneshot::channel();
                tokio::spawn_async(
                    async {
                        await!(tokio::io::copy(srx, tokio::io::stdout()).select2(crx));
                        println!("stopped copying");
                    },
                );
                // tokio::spawn(
                //     tokio::io::copy(srx, tokio::io::stdout())
                //         .map_err(|e| ())
                //         .map(|_| ()),
                // );
                while let Some(Ok(cmd)) = await!(rx.next()) {
                    match cmd {
                        Command::Write(s) => await!(stx.write_all_async(s.as_bytes())).unwrap(),
                        Command::Close => rx.close(),
                    };
                }
                ctx.send(()).unwrap();
                println!("/INNER");
            },
        );
        println!("/PERFORM");
        Ok(())
    }
    fn complete(
        self,
        _cx: TaskContext,
        _result: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        Ok(JsUndefined::new())
    }
}

enum Command {
    Write(String),
    Close,
}

pub struct MySerialPort {
    tx: mpsc::Sender<Command>,
}

impl Drop for MySerialPort {
    fn drop(&mut self) {
        println!("Dropping!");
    }
}

struct FutureTask<T>(Cell<Option<T>>)
where
    T: 'static + Send + Sized + Future;

impl<T> FutureTask<T>
where T : 'static + Send + Sized + Future {
    fn new(future: T) -> FutureTask<T> {
        FutureTask(Cell::new(Some(future)))
    }
}

impl<T> Task for FutureTask<T>
where
    T: 'static + Send + Sized + Future,
{
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        tokio::run(self.0.take().unwrap().then(|_| future::ok(())));
        Ok(())
    }
    fn complete(
        self,
        _cx: TaskContext,
        _result: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        Ok(JsUndefined::new())
    }
}

declare_types! {
    pub class JsSerialPort for MySerialPort {
        init(mut cx) {
            let path: String = cx.argument::<JsString>(0)?.value();
            let cb = cx.argument::<JsFunction>(1)?;
            let (tx, rx) = mpsc::channel(1);
            let rx = Cell::new(Some(rx));
            BackroundTask{rx, path}.schedule(cb);
            Ok(MySerialPort {tx})
        }

        method write(mut cx) {
            {
                let mut this = cx.this();
                let text: String = cx.argument::<JsString>(0)?.value();
                let cb = cx.argument::<JsFunction>(1)?;
                let guard = cx.lock();
                let this = &mut this.borrow_mut(&guard);
                let tx = this.tx.clone();
                FutureTask::new(tx.send(Command::Write(text))).schedule(cb);
            }
            Ok(cx.undefined().upcast())
        }

        method close(mut cx) {
            {
                let mut this = cx.this();
                let cb = cx.argument::<JsFunction>(0)?;
                let guard = cx.lock();
                let this = &mut this.borrow_mut(&guard);
                let tx = this.tx.clone();
                FutureTask::new(tx.send(Command::Close)).schedule(cb);
            }
            Ok(cx.undefined().upcast())
        }
    }
}

// Export the class
register_module!(mut m, {
    // <JsSerialPort> tells neon what class we are exporting
    // "SerialPort" is the name of the export that the class is exported as
    m.export_class::<JsSerialPort>("SerialPort")?;
    Ok(())
});
