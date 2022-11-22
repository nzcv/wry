const HTML: &str = r#"
  <html>
  <head>
      <style>
          html {
            font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
          }
          * {
              padding: 0;
              margin: 0;
              box-sizing: border-box;
          }
          main {
            display: grid;
            place-items: center;
            height: calc(100vh - 30px);
          }
          .titlebar {
              height: 30px;
              padding-left: 5px;
              display: grid;
              grid-auto-flow: column;
              grid-template-columns: 1fr max-content max-content max-content;
              align-items: center;
              background: #1F1F1F;
              color: white;
              user-select: none;
          }
          .titlebar-button {
              display: inline-flex;
              justify-content: center;
              align-items: center;
              width: 30px;
              height: 30px;
          }
          .titlebar-button:hover {
              background: #3b3b3b;
          }
          .titlebar-button#close:hover {
              background: #da3d3d;
          }
          .titlebar-button img {
              filter: invert(100%);
          }
      </style>
  </head>
  <body>
      <div class="titlebar">
          <div class="drag-region">Custom Titlebar</div>
          <div>
              <div class="titlebar-button" onclick="window.rpc.call('minimize', {})">
                  <img src="https://api.iconify.design/codicon:chrome-minimize.svg" />
              </div>
              <div class="titlebar-button" onclick="window.ipc.postMessage('maximize')">
                  <img src="https://api.iconify.design/codicon:chrome-maximize.svg" />
              </div>
              <div class="titlebar-button" id="close" onclick="window.ipc.postMessage('close')">
                  <img src="https://api.iconify.design/codicon:close.svg" />
              </div>
          </div>
      </div>
      <main>
          <h4> WRYYYYYYYYYYYYYYYYYYYYYY! </h4>
      </main>
      <script>
          document.addEventListener('mousedown', (e) => {
              if (e.target.classList.contains('drag-region') && e.buttons === 1) {
                  e.detail === 2
                      ? window.ipc.postMessage('maximize')
                      : window.ipc.postMessage('drag_window');
              }
          })
          document.addEventListener('touchstart', (e) => {
              if (e.target.classList.contains('drag-region')) {
                  window.ipc.postMessage('drag_window');
              }
          })
      </script>
  </body>
  </html>
"#;


fn main() -> wry::Result<()> {
    use wry::{
        application::{
            event::{Event, StartCause, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            window::WindowBuilder,
        },
        webview::WebViewBuilder,
    };
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello World")
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new(window)?
        .with_html(HTML)?
        .with_devtools(true)
        .with_initialization_script(
            r#"
        (function() {
            function Rpc() {
                const self = this;
                this._promises = {};
                this._error = (id, error) => {
                    if(this._promises[id]){
                        this._promises[id].reject(error);
                        delete this._promises[id];
                    }
                }
                this._result = (id, result) => {
                    if(this._promises[id]){
                        if (result.status == 200)
                            this._promises[id].resolve(result.data)
                        else
                            this._promises[id].reject({ code: result.status, data: result.data })
                        delete this._promises[id];
                    }
                }
                this.call = function(cmd, args) {
                    let array = new Uint32Array(1);
                    window.crypto.getRandomValues(array);
                    const id = array[0];
                    const payload = {
                        method_id: id,
                        method: "exec",
                        command: cmd,
                        args,
                    };
                    const promise = new Promise((resolve, reject) => {
                        self._promises[id] = {resolve, reject};
                    });
                    window.ipc.postMessage(JSON.stringify(payload));
                    return promise;
                }
            }
            window.external = window.external || {};
            window.external.rpc = new Rpc();
            window.rpc = window.external.rpc;
        })(); console.log('window.rpc inited');"#,
        )
        .with_ipc_handler(move |_win, msg| {
          println!("{}", msg);
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
