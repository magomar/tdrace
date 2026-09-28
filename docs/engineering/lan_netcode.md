---
type: Architecture Spec
title: "LAN Race Netcode"
description: "How a LAN race stays in sync: owner-authoritative cars, the host relay and referee, the shared race clock, interpolation, the reliable control channel, packet formats, tuning constants, tests, and known limits."
status: active
category: engineering
tags: [lan, multiplayer, netcode, cabinet, udp]
---

# LAN Race Netcode 🌐

Contract: [spec 044](../../specs/044_robust_lan_race_synchronization_with_ownerauthoritative_cars.md). Lobby, discovery and direct IP: [spec 036](../../specs/036_local_network_lan_multiplayer_and_cabinet_lobby.md).

## 1. Model

| Who | Does |
|---|---|
| Every machine | Runs physics only for **its own car**. Sends that car's state at 60 Hz. Shows every other car from received states, 100 ms in the past. |
| Host | The same as above, and also: relays the latest state of every car to all clients, fixes the roster, schedules the green light, records the finish order, and sends the final results. |

There is no correction of your own car, so it never jumps back. Two machines can see a contact a little differently. Each machine applies the impulse only to its own car.

## 2. Race flow

1. Host presses START → `launch_race()` freezes the roster → reliable `RaceLaunch` (track, laps, collision mode, roster with grid order).
2. Every machine builds its car list **only** from `RaceLaunch`. Car index `i` = `roster[i]`.
3. Each client sends `Loaded`. When all are loaded (or after 10 s, then late clients are dropped), the host sends `RaceStart { start_at }` = now + 3 s on the host clock.
4. The HUD shows `WAITING FOR RACERS`, then 3-2-1 on the shared race clock. Green light = race clock 0.0 on every machine.
5. Owners send `NetCarState`. The host sends `WorldState` with all cars.
6. An owner that completes the last lap sends `Finished`. The host broadcasts `Standings`. A finisher keeps watching (own car braked, no collisions).
7. When everyone finished, or 30 s after the winner, the host sends `RaceOver`. Every machine shows those results.

## 3. Packets

Header on every datagram: `TDLN` + version (`2`) + kind.

| Kind | Name | Format | Size |
|---|---|---|---|
| 0 | JSON `Packet` | beacon, join request/response, ping/pong, disconnect notice | small |
| 1 | `NetCarState` | slot, time_ms, pos, vel, angle, yaw rate, steer, elevation, vertical velocity, throttle, brake, flags, lap, checkpoint, progress (little-endian) | 57 B |
| 2 | `WorldState` | host time_ms, count, count × car | 11 + 51 per car (8 cars = 419 B) |
| 3 | reliable fragment | seq u16, frag, count, JSON `ControlMessage` bytes | ≤ 1400 B |
| 4 | ack | seq u16 | 8 B |

Reliable messages (`ControlMessage`): `StateSync`, `ClientSlotUpdate`, `RaceLaunch`, `Loaded`, `RaceStart`, `Finished`, `Standings`, `PlayerLeft`, `RaceOver`. They are resent every 150 ms until acked. After 20 tries the peer is lost.

A v2 host answers a v1 join request in v1 format with `RejectedVersionMismatch`.

## 4. Code map

| File | Role |
|---|---|
| `crates/cabinet/src/net/transport.rs` | `Transport` trait, `UdpTransport`, test `SimNetwork` (loss, delay, jitter, drop filter) |
| `crates/cabinet/src/net/wire.rs` | Header, binary `NetCarState` / `WorldState` |
| `crates/cabinet/src/net/reliable.rs` | Seq/ack/resend/fragment channel |
| `crates/cabinet/src/net/clock.rs` | Host clock offset from pings |
| `crates/cabinet/src/net/interp.rs` | `RemoteCarBuffer` |
| `crates/cabinet/src/net/stats.rs` | `NetStats`, counting transport |
| `crates/cabinet/src/net/host.rs`, `client.rs` | Session endpoints, relay, referee |
| `crates/tdrace-app/src/game/lan.rs` | Game glue: `LanRaceState`, `pump_lan`, `lan_race_frame`, `lan_after_frame`, dev net HUD |
| `crates/tdrace-app/src/game/mod.rs` | `physics_step` skips remote cars; collision save/restore; results in host order |

## 5. Tuning constants

| Constant | Value | Where |
|---|---|---|
| State send rate | 60 Hz | `game/lan.rs` `STATE_SEND_INTERVAL_SEC` |
| Interpolation delay | 100 ms | `interp.rs` `INTERP_DELAY_SEC` |
| Max extrapolation | 250 ms | `interp.rs` `MAX_EXTRAPOLATION_SEC` |
| Recovery blend | 100 ms | `interp.rs` `RECOVERY_BLEND_SEC` |
| Reliable resend / tries | 150 ms / 20 | `reliable.rs` |
| Countdown | 3 s | `host.rs` `COUNTDOWN_SEC` |
| Load timeout | 10 s | `host.rs` `LOAD_TIMEOUT_SEC` |
| Finish timeout | 30 s after the winner | `host.rs` `FINISH_TIMEOUT_SEC` |
| Heartbeat timeout | host 3.5 s, client 4.5 s | `host.rs`, `client.rs` |
| Client ping interval | 0.5 s | `client.rs` |

## 6. Debugging

- `TDRACE_DEV=1`, then `F9` in a LAN race: RTT, clock offset, datagrams/s, decode and encode errors, stale drops, reliable backlog, and per remote car: buffer depth, extrapolation, drops.
- Healthy on a LAN: encode errors 0, extrapolation 0 ms most of the time, buffer 6–8 states.

## 7. Tests

```bash
cargo test -p cabinet net::
cargo test -p cabinet --test net_tests
cargo test -p tdrace-app --test lan_integration_tests --test lan_sync_tests
```

`lan_sync_tests` runs real `RaceSession::update()` on a host and 3 clients over `SimNetwork` (5% loss, 0–20 ms jitter), with bots driving. Measured 2026-09-28: remote position error p50 0.010 m, p95 0.026 m, max 0.449 m over 60 s. The in-memory network shares one clock, so this does not include clock-offset error (on a real LAN about speed × offset, e.g. 50 m/s × 5 ms = 0.25 m).

## 8. Known limits

- A modified client can lie about its car. Accepted for a home LAN.
- A braked or parked car cannot be pushed from another machine. Paused cars stay solid; finished and left cars do not collide.
- One LAN session is one race. After the results, players return to the LAN hub.
- No AI bots in LAN races yet. A host-owned bot would fit this model.
