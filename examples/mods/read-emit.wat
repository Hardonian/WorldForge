;; Minimal core-Wasm example for the v0 host ABI. The production Component
;; Model contract is described by wit/worldforge/*.wit.
(module
  (import "worldforge" "read_resource" (func $read-resource (result i64)))
  (import "worldforge" "emit_event" (func $emit-event (param i64)))
  (func (export "init"))
  (func (export "on_tick") (param i64)
    call $read-resource
    call $emit-event)
  (func (export "on_event") (param i64)))
