// This code runs inside the Ownable iframe. It's not part of the React app.
// See `ownable-js.webpack.js`

// @ts-ignore
import Listener from "simple-iframe-rpc/listener";

type Dict = {[prop: string]: any}
type StateDump = Array<[ArrayLike<number>, ArrayLike<number>]>

interface MsgInfo {
  sender: string;
  funds: Array<never>;
}

const listener = new Listener({
  init,
  instantiate,
  execute,
  externalEvent,
  query,
  refresh,
});
listener.listen(window, "*");

let ownableId: string;
let worker: Worker;

function init(id: string, javascript: string, wasm: Uint8Array): Promise<any> {
  ownableId = id;

  return new Promise(async (resolve, reject) => {
    const blob = new Blob([javascript], {type: `application/javascript`});
    const blobURL = URL.createObjectURL(blob);
    worker = new Worker(blobURL, { type: "module" });

    worker.onmessage = (event) => {
      resolve(event.data);
    };
    worker.onerror = (err) => reject(err);
    worker.onmessageerror = (err) => reject(err);

    const buffer = wasm.buffer;
    worker.postMessage(buffer, [buffer]);
  });
}

function workerCall<T extends string|Dict>(
  type: string,
  ownableId: string,
  msg: Dict,
  info: Dict,
  state?: StateDump,
): Promise<{result: T, state: StateDump}> {
  return new Promise((resolve, reject) => {
    if (!worker) {
      reject("Unable to execute: not initialized");
      return;
    }

    worker.addEventListener('message', (event: MessageEvent<Map<string, any>|{err: any}>) => {
      if ('err' in event.data) {
        reject(event.data.err);
        return;
      }

      const result = JSON.parse(event.data.get('state'));
      const nextState: StateDump = event.data.has('mem') ? JSON.parse(event.data.get('mem')).state_dump : state;

      resolve({result, state: nextState});
    }, { once: true });

    worker.postMessage({type, ownable_id: ownableId, msg, info, mem: {state_dump: state}});
  });
}

async function instantiate(msg: Dict, info: Dict) {
  const {result: resultMap, state} = await workerCall<{attributes: [{key: string, value: any}]}>(
    "instantiate",
    ownableId,
    msg,
    info
  );
  const result = Object.fromEntries(resultMap.attributes.map(a => [a.key, a.value]));

  return {result, state};
}

function execute(
  ownableId: string,
  msg: Dict,
  info: MsgInfo,
  state: StateDump
): Promise<{result: Dict, state: StateDump}> {
  return workerCall<Dict>("execute", ownableId, msg, info, state);
}

function externalEvent(
  ownableId: string,
  msg: Dict,
  info: MsgInfo,
  state: StateDump
): Promise<{result: Dict, state: StateDump}> {
  return workerCall<Dict>("external_event", ownableId, msg, info, state);
}

async function query(msg: Dict, state: StateDump): Promise<Dict> {
  const {result: resultB64} = await workerCall<string>("query", ownableId, msg, {}, state);
  return JSON.parse(atob(resultB64)) as Dict;
}

async function refresh(state: StateDump): Promise<void> {
  const widgetState = await query({get_ownable_config: {}}, state);

  const iframe = document.getElementsByTagName('iframe')[0];
  iframe.contentWindow!.postMessage({ownableId, state: widgetState}, "*");
}