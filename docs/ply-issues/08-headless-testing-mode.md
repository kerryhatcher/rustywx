# Feature request: headless testing mode

**Type:** Feature request
**Severity:** Medium — enables CI-verifiable UI testing

## Summary

Ply has no headless rendering mode, making automated UI testing
impossible. The only verification method is manual visual inspection.
This is a significant limitation for any project with CI requirements.

## Impact

Our port plan explicitly states:

> No new integration tests are planned for the Ply UI layer — Ply does
> not have a headless testing mode. Manual validation checklists (in each
> stage) are the primary verification method.

This means UI regressions can only be caught by a human running the app.
For a data-visualization tool with 8 stages of development, this is a
real risk — layout breaks, element visibility issues, and interaction
bugs can slip through CI undetected.

## What we need

Even a limited headless mode would be valuable:

### Minimal: layout validation
- Run the game loop without a GPU context (software rasterizer or no
  rendering at all)
- Assert element tree structure: "element with id X exists," "element Y
  has width Z," "element A is a child of element B"
- Assert element visibility: "element X is not hidden"

### Moderate: interaction simulation
- Simulate clicks (`ply.simulate_click("my-button")`)
- Simulate key presses
- Assert state changes after interactions
- Run in CI without a display server

### Full: pixel comparison
- Render to an offscreen buffer
- Compare against reference screenshots with a tolerance
- Detect visual regressions automatically

### Precedent

egui provides `egui::Context` testing without a window. Bevy has
`bevy_render` headless modes. A similar capability for Ply would
dramatically improve the testing story for the ecosystem.
