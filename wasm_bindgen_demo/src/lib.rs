use js_sys::{Function, Object, Reflect, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};

// lifted from the `console_log` example
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

const WASM: &[u8] = include_bytes!("add_bg.wasm");

async fn run_async() -> Result<(), JsValue> {
    console_log!("instantiating a new wasm module directly");

    let a = JsFuture::from(WebAssembly::instantiate_buffer(WASM, &Object::new())).await?;
    let b: WebAssembly::Instance = Reflect::get(&a, &"instance".into())?.dyn_into()?;

    let c = b.exports();

    let add = Reflect::get(c.as_ref(), &"add".into())?
        .dyn_into::<Function>()
        .expect("add export wasn't a function");

    let three = add.call2(&JsValue::undefined(), &1.into(), &2.into())?;
    console_log!("1 + 2 = {:?}", three);


    let return_string = Reflect::get(c.as_ref(), &"return_string".into())?
        .dyn_into::<Function>()
        .expect("return string export wasn't a function");

    let result = return_string.call0(&JsValue::undefined());
    console_log!("returned string: {:?}", result);

    let concat_string = Reflect::get(c.as_ref(), &"concat_string".into())?
        .dyn_into::<Function>()
        .expect("concat string export wasn't a function");

    let concat_result = concat_string.call1(&JsValue::undefined(), &JsValue::from(" name"));
    console_log!("concatenated string: {:?}", concat_result);

    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() {
    spawn_local(async {
        run_async().await.unwrap_throw();
    });
}
