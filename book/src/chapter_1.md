# Chapter 1

## winit
* event loop
* window

## wgpu
* instance
* surface
* needs async (to get adapter)

## pollster
* macro feature for async main

## can't find adapter
* create custom `Instance` and add
  wgpu::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER
* use env to decide if it's set

## expand to all fields from env, as does wgpu-example::framework

## WASM
* add crate type cdylib to cargo.toml
* copy build-wasm.sh
* need lib.rs to compile with build-wasm.sh
* need to pin web-sys at old version for unstable api use in wgpu
* need to connect canvas to wasm before create surface and adapter

## create device and queue
* cfg_if to conditionally let event_loop_fuction

## fix window escape closure by cloning as arc
