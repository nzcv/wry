use serde::{Deserialize, Serialize};
use serde_json::Value;
use wry::{
    application::window::Window,
    webview::{self, WebViewBuilder},
};
use std::marker::PhantomData;

const RPC_VERSION: &str = "2.0";

/// RPC request message.
///
/// This usually passes to the [`RpcHandler`] or [`WindowRpcHandler`](crate::WindowRpcHandler) as
/// the parameter. You don't create this by yourself.
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// RPC response message which being sent back to the Javascript side.
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    jsonrpc: String,
    pub(crate) id: Option<Value>,
    pub(crate) result: Option<Value>,
    pub(crate) error: Option<Value>,
}

struct JsonRpc {
    pub rpc_handler: Option<Box<dyn Fn(&Window, RpcRequest) -> Option<RpcResponse>>>,
}


// impl Default for JsonRpc<'a> {
//     fn default() -> Self {
//         Self { rpc_handler: None }
//     }
// }

impl<'a> JsonRpc<'a> {
    pub fn new() -> Self {
        JsonRpc::default()
    }

    /// Set the RPC handler to Communicate between the host Rust code and Javascript on webview.
    ///
    /// The communication is done via [JSON-RPC](https://www.jsonrpc.org). Users can use this to register an incoming
    /// request handler and reply with responses that are passed back to Javascript. On the Javascript
    /// side the client is exposed via `window.rpc` with two public methods:
    ///
    /// 1. The `call()` function accepts a method name and parameters and expects a reply.
    /// 2. The `notify()` function accepts a method name and parameters but does not expect a reply.
    ///
    /// Both functions return promises but `notify()` resolves immediately.
    pub fn with_rpc_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Window, RpcRequest) -> Option<RpcResponse> + 'static,
    {
        self.rpc_handler = Some(Box::new(handler));
        self
    }

    pub fn build(mut self, webview: &mut WebViewBuilder<'a>) {
        if self.rpc_handler.is_some() {
            let js = r#"
            (function() {
                function Rpc() {
                    const self = this;
                    this._promises = {};

                    // Private internal function called on error
                    this._error = (id, error) => {
                        if(this._promises[id]){
                            this._promises[id].reject(error);
                            delete this._promises[id];
                        }
                    }

                    // Private internal function called on result
                    this._result = (id, result) => {
                        if(this._promises[id]){
                            this._promises[id].resolve(result);
                            delete this._promises[id];
                        }
                    }

                    // Call remote method and expect a reply from the handler
                    this.call = function(method) {
                        let array = new Uint32Array(1);
                        window.crypto.getRandomValues(array);
                        const id = array[0];
                        const params = Array.prototype.slice.call(arguments, 1);
                        const payload = {jsonrpc: "2.0", id, method, params};
                        const promise = new Promise((resolve, reject) => {
                            self._promises[id] = {resolve, reject};
                        });
                        window.external.invoke(JSON.stringify(payload));
                        return promise;
                    }

                    // Send a notification without an `id` so no reply is expected.
                    this.notify = function(method) {
                        const params = Array.prototype.slice.call(arguments, 1);
                        const payload = {jsonrpc: "2.0", method, params};
                        window.external.invoke(JSON.stringify(payload));
                        return Promise.resolve();
                    }
                }
                window.external = window.external || {};
                window.external.rpc = new Rpc();
                window.rpc = window.external.rpc;
            })();
            "#;
            webview.with_initialization_script(js);
        }
    }
}
