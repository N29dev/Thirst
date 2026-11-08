# Thirst visual design & motion

Thirst is a **native-feeling desktop launcher**, not a game client or a web dashboard that runs constantly in the background. Its visual system is based on quiet contrast, crisp typography, a unified icon language and short, purposeful transitions.

## Implemented in this iteration

- **Navigation consistency.** Overview, Library, Server Browser, Fast Flags and Settings now use one lightweight SVG icon component, with consistent 19px sizing, active-page semantics, and tooltips when the sidebar collapses.
- **Page transitions.** Navigating to a new tab creates a keyed screen with a small upward motion and fade (260ms). Switching tabs resets an old screen's scroll position, preventing a previous deep scroll from opening the next screen halfway down.
- **Subtle motion.** Hero mock launcher panels and hover cards use short, non-looping reveal animations, while controls use restrained press feedback and focus rings. No video background, GPU-heavy idle blur loop, music or unnecessary asset loading.
- **Useful loading state.** The server list shows five stable placeholders during an initial refresh. The loading shimmer only runs while a request is pending and is disabled with Lite Mode/reduced motion.
- **Accessible controls.** Active sidebar items advertise the current page, compact navigation has hover titles, and the Fast Flags JSON import dropzone exposes a keyboard focus indicator. Decorative hover lighting is not treated as an interactive control.
- **Responsive handling.** Navigation collapses below a compact breakpoint; grids, server-loading placeholders and overlays have narrow-window treatments without losing access to main actions.

## UX invariants

1. **Lite Mode** disables transitions/animations, visual filters and expensive shadows. It keeps every important interaction, button and page accessible.
2. `prefers-reduced-motion` disables the page transition and decorative animations even if Lite Mode is off.
3. Visual changes do **not** add Roblox network requests, cookies, analytics or third-party font/CDN dependencies.
4. Player names, server metadata, saved library files and Fast Flags retain their existing state/behavior when moving between screens.
5. Loading placeholders are **not** fake server results or availability claims.
6. Explicit user actions remain available even if animations fail, are disabled or the GPU is slow.

## Next visual QA

- Test the actual bundled app on Apple Silicon and Windows at 740px, 820px, 1024px, 1280px, 1600px widths and with increased OS text scaling.
- Test navigation from a scrolled-down server list to Settings and from Fast Flags back to Overview.
- Confirm focus visibility and keyboard operation in the collapsed sidebar, file import controls and login consent dialog.
- Check Svelte transitions against Lite Mode and OS reduced-motion, and verify no layout shift or focus loss after repeated screen switching.
- Capture screenshots from both light/dark macOS appearance to verify contrast and native window inset.
- Profile idle CPU/GPU and memory after 30 minutes of use, with and without Lite Mode. Tune animation duration rather than introducing a perpetual render loop.

This visual system is original code. It does not import AppleBlox UI source or theme assets.
