//! Network built-ins: `fetch`, `serve`.

use crate::context::{Callable, NativeFn};
use crate::heap::ManagedObject;
use crate::vm::execute_bytecode;
use crate::value_fmt::stringify_value;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use ys_core::compiler::Value;
use ys_core::error::JitError;

pub fn register(fns: &mut FxHashMap<String, NativeFn>) {
    register_fetch(fns);
    register_serve(fns);
}

fn register_fetch(fns: &mut FxHashMap<String, NativeFn>) {
    fns.insert("fetch".into(), Arc::new(|ctx, args, loc| {
        Box::pin(async move {
            let url = args.first()
                .and_then(|v| ctx.value_as_string(*v))
                .ok_or_else(|| JitError::runtime(
                    "fetch requires a string URL as its first argument",
                    loc.line as usize,
                    loc.col as usize,
                ))?;

            match reqwest::get(&url).await {
                Ok(resp) => {
                    let status = resp.status();
                    let body   = resp.text().await.unwrap_or_else(|_| "Error reading body".into());
                    println!("Fetch {}: {} - {}", url, status, body);
                    Ok(Value::from_bits(0))
                }
                Err(e) => {
                    println!("Fetch {} failed: {}", url, e);
                    Ok(Value::from_bits(0))
                }
            }
        })
    }));
}

fn register_serve(fns: &mut FxHashMap<String, NativeFn>) {
    fns.insert("serve".into(), Arc::new(|ctx, args, loc| {
        Box::pin(async move {
            let (port_val, handler_val) = match args.as_slice() {
                [p, h] => (*p, *h),
                _ => return Err(JitError::runtime(
                    "serve(port, handler) expects 2 arguments",
                    loc.line as usize,
                    loc.col as usize,
                )),
            };

            let port = port_val.as_number().ok_or_else(|| JitError::runtime(
                "serve: port must be a number",
                loc.line as usize,
                loc.col as usize,
            ))? as u16;

            let handler_name = ctx.value_as_string(handler_val).ok_or_else(|| JitError::runtime(
                "serve: handler must be a function name string",
                loc.line as usize,
                loc.col as usize,
            ))?;

            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .await
                .map_err(|e| JitError::runtime(
                    format!("Failed to bind to port {}: {}", port, e),
                    loc.line as usize,
                    loc.col as usize,
                ))?;

            println!("Web server listening on port {}", port);

            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        let (mut socket, _) = accept_res.map_err(|e| JitError::runtime(
                            format!("Accept error: {}", e),
                            loc.line as usize,
                            loc.col as usize,
                        ))?;

                        let ctx          = ctx.clone();
                        let handler_name = handler_name.clone();

                        tokio::spawn(async move {
                            let mut buf = [0u8; 1024];
                            let n = match socket.read(&mut buf).await {
                                Ok(n) if n > 0 => n,
                                _ => return,
                            };
                            let req_data = String::from_utf8_lossy(&buf[..n]).to_string();

                            let name_id = ctx.string_pool
                                .iter()
                                .position(|s| s.as_ref() == handler_name)
                                .map(|i| i as u32);
                            let callable = name_id.and_then(|id| ctx.get_callable(id));

                            let Some(Callable::User(f)) = callable else {
                                let _ = socket
                                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\nHandler not found")
                                    .await;
                                return;
                            };

                            let registers = make_registers(f.locals_count);
                            if f.locals_count > 0 {
                                let val = if let Some(sso) = Value::sso(&req_data) {
                                    sso
                                } else {
                                    let temp = AtomicU64::new(0);
                                    ctx.alloc(ManagedObject::String(Arc::from(req_data.clone())), &temp);
                                    Value::from_bits(temp.load(Ordering::Relaxed))
                                };
                                registers[0].store(val.to_bits(), Ordering::Relaxed);
                            }

                            let mut js     = JoinSet::new();
                            let t_roots    = Arc::new(Mutex::new(Vec::with_capacity(16)));
                            ctx.active_registers.lock().push(t_roots.clone());

                            match execute_bytecode(&f.instructions, ctx.clone(), &mut js, registers, None, t_roots).await {
                                Ok(res) => {
                                    let body = stringify_value(&ctx, res);
                                    let resp = if body.starts_with("HTTP/") {
                                        body
                                    } else {
                                        format!(
                                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                                            body.len(), body
                                        )
                                    };
                                    let _ = socket.write_all(resp.as_bytes()).await;
                                }
                                Err(e) => {
                                    let err = format!("HTTP/1.1 500 Internal Server Error\r\n\r\nError: {:?}", e);
                                    let _ = socket.write_all(err.as_bytes()).await;
                                }
                            }
                        });
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutting down web server gracefully...");
                        break;
                    }
                }
            }
            Ok(Value::from_bits(0))
        })
    }));
}

fn make_registers(count: usize) -> Arc<[AtomicU64]> {
    let v: Vec<AtomicU64> = (0..count).map(|_| AtomicU64::new(0)).collect();
    Arc::from(v)
}
