// Starts the corpus worker (crates/rgworker), which Trunk builds as a no-modules script. The query
// string (?v=<build>) is passed on so a new deploy never mixes with cached files of an older one.
importScripts("./rgworker.js" + self.location.search);
wasm_bindgen("./rgworker_bg.wasm" + self.location.search);
