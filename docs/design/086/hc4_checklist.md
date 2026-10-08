# Spec 086 · HC-4 hands-on checklist

Open the editor on the test circuit (it has every kind of item):

```bash
cd /Users/mario.gomez/workspace/games/tdrace/.claude/worktrees/track-editor-inspector-c9c1b9 && TDRACE_GIT_TRACKS_DIR=/private/tmp/claude-501/-Users-mario-gomez-workspace-games-tdrace--claude-worktrees-track-editor-inspector-c9c1b9/d1d22908-f40d-4365-a382-318921529e63/scratchpad/tracks-22c5df5 cargo run -p tdrace-app -- track crates/tdrace-app/tests/fixtures/editor_inspector_fixture.json editor
```

Tick each step that works. Write a note for each step that does not.

1. [ ] **Drag = one undo.** Click one waypoint. Drag the Width bar left and right for a few seconds. Release. Press Ctrl+Z once. The width goes back to the value before the drag.
2. [ ] **Hold = one undo.** Hold `+` on Width for about 1 s. The first step is immediate; repeats start after a short pause. Press Ctrl+Z once. All the steps go back.
3. [ ] **Click to type.** Click the Width bar without dragging. Type `7.5`, press Enter. The width is 7.5 m. Click the bar again, press Esc: nothing changes.
4. [ ] **Banking.** Click the 18° chip, then `±`. Banking shows −18°. Press `]` once: −17°.
5. [ ] **Edges.** Set L curb on, R wall off, Dist L 3 m, Dist R 6 m, Wall type Tyres. Each change shows on the map.
6. [ ] **Surface dropdown.** Open Surface. Scroll the list with the wheel and with the arrow keys. Pick Gravel. Open it again and press Esc: the list closes and the editor stays open.
7. [ ] **Many waypoints.** Box-select 3 or more waypoints. The panel shows the same controls, "N selected", and "Mixed" / "—" where values differ. Click the 10° chip: all get 10°. Press `-` on Width once: each width drops by 0.5 m.
8. [ ] **Zone and ramp.** Select a surface zone: change Material and Layer. Select a jump ramp: change Angle, Length, and click "Fit pitch". The side view updates.
9. [ ] **Mixed selection.** Shift+click a waypoint, a zone and a ramp. The chip row shows the three kinds. Switch chips. A change made for one kind does not change the others.
10. [ ] **Wheel and tooltips.** Click a slider, then click the map, then turn the wheel over the map: only the map zooms. Turn the wheel over the panel: no value changes. Rest the pointer on Banking for half a second: a tooltip shows `[ / ]`, fully on screen.

Extra (optional): with nothing selected, the panel shows the circuit. Open Off-track: Sheet Ice is in the list. Press the 12 grid preset: the grid has 12 slots.
