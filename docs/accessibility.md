# Accessibility review

Target: WCAG 2.2 AA principles on Windows 11/WebView2. This is a focused product
review, not a third-party certification.

## Implemented and reviewed

- Native buttons, inputs, selects, headings, lists, status regions, alerts, and dialogs are used.
- Sidebar navigation exposes current page and names icon-only compact navigation.
- A keyboard-visible skip link moves focus to the main content.
- `:focus-visible` covers links, buttons, inputs, selects, textareas, and programmatic focus targets.
- Release-critical dialogs trap Tab/Shift+Tab, close with Escape, and restore trigger focus.
- Placeholder-only project/session/version controls have persistent accessible names.
- Loading uses polite status; errors use alert; index removal explains that originals remain.
- Reduced-motion preference collapses transitions and animations.
- Real 1182×791 inspection caught and fixed a Timeline filter-grid overflow;
  the final screenshot shows all controls without horizontal clipping.

## Contrast review

Core light theme pairs meet 4.5:1 for normal text: `#17201d` on white,
`#64706b` on white, and `#28624f` on white. Core dark pairs meet 4.5:1:
`#eef4f1` and `#a8b3ae` on `#222925`, and `#72b49a` on `#222925`.
Subtle `#87918d` text is supplemental only and should not carry the sole meaning.
Focus rings use a distinct two-pixel outline.

## Honest gaps

- No independent screen-reader certification has been completed.
- High-contrast/forced-colors mode needs a dedicated manual pass.
- Dense version/project/session editing needs more usability testing at 200% zoom.
- Localization can change control widths; both languages need visual regression review.

These gaps are documented rather than presented as conformance claims.
