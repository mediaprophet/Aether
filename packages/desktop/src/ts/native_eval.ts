import {
  Channel,
  AetherChannel,
  WeakAetherChannel,
} from "../../../document/src/ts/eval";

// In aether desktop, eval needs to use the window object to store global state because we evaluate separate snippets of javascript in the browser
declare global {
  interface Window {
    __msg_queues: WeakAetherChannel[];
    finalizationRegistry: FinalizationRegistry<{ id: number }>;

    getQuery(request_id: number): WeakAetherChannel;

    createQuery(request_id: number): NativeAetherChannel;
  }
}

// A message that can be sent to the desktop renderer about a query
class QueryParams {
  id: number;
  data: { method: "drop" | "send"; data?: any };

  constructor(id: number, method: "drop" | "send", data?: any) {
    this.id = id;
    this.data = { method, data };
  }
}

window.__msg_queues = window.__msg_queues || [];
// In aether desktop, eval is copy so we cannot run a drop handler. Instead, the drop handler is run after the channel is garbage collected in the javascript side
window.finalizationRegistry =
  window.finalizationRegistry ||
  new FinalizationRegistry(({ id }) => {
    // @ts-ignore - wry gives us this
    window.ipc.postMessage(
      JSON.stringify({
        method: "query",
        params: new QueryParams(id, "drop"),
      })
    );
  });

// Get a query from the global state
window.getQuery = function (request_id: number): WeakAetherChannel {
  return window.__msg_queues[request_id];
};

// Create a new query (and insert it into the global state)
window.createQuery = function (request_id: number): NativeAetherChannel {
  return new NativeAetherChannel(request_id);
};

export class NativeAetherChannel extends AetherChannel {
  rust_to_js: Channel;
  request_id: number;

  constructor(request_id: number) {
    super();
    this.rust_to_js = new Channel();
    this.request_id = request_id;

    window.__msg_queues[request_id] = this;
    window.finalizationRegistry.register(this, { id: request_id });
  }

  // Receive message from Rust
  async recv() {
    return await this.rust_to_js.recv();
  }

  // Send message to rust.
  send(data: any) {
    // @ts-ignore - wry gives us this
    window.ipc.postMessage(
      JSON.stringify({
        method: "query",
        params: new QueryParams(this.request_id, "send", data),
      })
    );
  }

  // Send data from rust to javascript
  rustSend(data: any) {
    this.rust_to_js.send(data);
  }

  // Receive data sent from javascript in rust. This is a no-op in the native interpreter because the rust code runs remotely
  async rustRecv(): Promise<any> { }

  // Close the channel, dropping it.
  close(): void {
    window.__msg_queues[this.request_id] = null;
  }
}
