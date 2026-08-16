<script lang="ts">
  // A one-line marquee that moves only while its row is hovered: if the text fits
  // it sits still and truncates like any label; if it overflows it holds still and
  // truncated until the pointer is over the row, then scrolls its content in a slow,
  // seamless loop so the tail can be read — and slides back to its starting position
  // the moment the pointer leaves, so a row at rest always reads from the beginning
  // of its title rather than freezing wherever the pointer happened to go. Motion is
  // dropped entirely when the operator has asked for less of it — the text then
  // truncates rather than scrolls, so nothing is ever hidden, only still.
  //
  // The hover trigger is the ancestor `.group` (the session row) — so hovering
  // anywhere on the row, not just the title, starts it. (Keyboard focus in the row
  // turns the trailing fade on, matching the controls that surface on focus-within,
  // but does not start the scroll — see below.)
  //
  // It measures itself: the container's width against one copy of the text. Two
  // copies ride the track with a gap between them, and the track slides by exactly
  // one copy-plus-gap, so the second copy lands where the first began and the loop
  // has no seam.
  //
  // The scroll is a Web Animations API animation rather than a CSS one, because the
  // return leg needs the *current* offset: pausing a CSS animation leaves the track
  // parked mid-title, and there is no reliable way to transition out of an animated
  // transform back to the base one. Here the loop is cancelled, its offset read off
  // the element, and a short animation walks it home — and re-entering mid-return
  // resumes the loop from where the return had got to, so flicking the pointer
  // across a row never snaps.

  let {
    text,
    class: klass = "",
    fadeTail = false,
  }: {
    text: string;
    // Styling from the parent — the marquee is monochrome and muted by default, but
    // the caller sets size and colour so it reads as a subtitle beside its label.
    class?: string;
    // Hold the trailing fade on even when the row is neither hovered nor focused:
    // the row's own trailing controls (the close ✕) are showing anyway, and the
    // title runs underneath them. Hover and focus turn the fade on by themselves.
    fadeTail?: boolean;
  } = $props();

  // Constant scroll speed in px/s, so a long title doesn't whip past faster than a
  // short one — the duration is derived from the distance, not fixed.
  const SPEED = 32;
  // The gap between the two copies, in px, so the loop breathes rather than butting
  // the text against its own tail.
  const GAP = 32;
  // How long the slide back to the start takes. Short and eased: it is an undo of
  // the scroll, not a scroll of its own.
  const RETURN_MS = 240;

  let root = $state<HTMLDivElement | null>(null);
  let track = $state<HTMLDivElement | null>(null);
  let containerWidth = $state(0);
  let textWidth = $state(0);
  let hovered = $state(false);
  let focused = $state(false);

  const reduceMotion =
    typeof matchMedia !== "undefined" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches;

  // Scroll only when the text genuinely overflows its container and motion is
  // allowed. A one-pixel slack avoids a jittery scroll on a title that only just
  // fits.
  const scrolling = $derived(
    !reduceMotion && textWidth > 0 && textWidth > containerWidth + 1,
  );
  const shift = $derived(textWidth + GAP);
  const durationMs = $derived((shift / SPEED) * 1000);
  // The trailing fade tracks exactly what the row's close ✕ does: shown on the
  // active row, and on whichever row the pointer or the keyboard is in.
  const fading = $derived(fadeTail || hovered || focused);

  // Hover and focus come from the ancestor row rather than this element, so the
  // whole row is the trigger — the same contract the CSS group-hover version had.
  $effect(() => {
    const el = root;
    if (!el) return;
    const row = el.closest(".group") ?? el;
    const enter = () => (hovered = true);
    const leave = () => (hovered = false);
    const focus = () => (focused = true);
    const blur = () => (focused = false);
    row.addEventListener("pointerenter", enter);
    row.addEventListener("pointerleave", leave);
    row.addEventListener("focusin", focus);
    row.addEventListener("focusout", blur);
    return () => {
      row.removeEventListener("pointerenter", enter);
      row.removeEventListener("pointerleave", leave);
      row.removeEventListener("focusin", focus);
      row.removeEventListener("focusout", blur);
    };
  });

  // The running loop and the slide-back, held so each can cancel the other; and the
  // distance the live loop was built for, so a title that changes under it restarts
  // rather than looping over a stale seam.
  let loop: Animation | null = null;
  let loopSpan = 0;
  let ret: Animation | null = null;

  // Where the track currently sits, in px (0 at the start, negative once it has
  // scrolled) — read off the element so it is the animated value, not the base one.
  function offsetOf(el: HTMLElement): number {
    const t = getComputedStyle(el).transform;
    if (!t || t === "none") return 0;
    try {
      return new DOMMatrixReadOnly(t).m41;
    } catch {
      return 0;
    }
  }

  $effect(() => {
    const el = track;
    const span = shift;
    const ms = durationMs;
    // Hover alone drives the scroll — not focus. A click on a row leaves it
    // focused, and a row that scrolled for as long as it held focus would never
    // settle. Focus still turns the *fade* on, because it turns the ✕ on.
    const run = scrolling && hovered;
    if (!el) {
      // The track left the DOM (the text now fits, or reduced motion): its
      // animations went with it.
      loop = null;
      ret = null;
      return;
    }
    if (run) {
      if (loop && loopSpan === span) return;
      ret?.cancel();
      ret = null;
      // Pick up wherever the track is — mid slide-back, or mid loop under a title
      // that just changed length.
      const from = offsetOf(el);
      loop?.cancel();
      loop = el.animate(
        [
          { transform: "translateX(0)" },
          { transform: `translateX(${-span}px)` },
        ],
        { duration: ms, iterations: Infinity, easing: "linear" },
      );
      loopSpan = span;
      loop.currentTime = span > 0 ? (Math.min(-from, span) / span) * ms : 0;
    } else if (loop) {
      const from = offsetOf(el);
      loop.cancel();
      loop = null;
      // Already home (it had barely started): nothing to walk back.
      if (from > -0.5) return;
      ret?.cancel();
      ret = el.animate(
        [
          { transform: `translateX(${from}px)` },
          { transform: "translateX(0)" },
        ],
        { duration: RETURN_MS, easing: "ease-out" },
      );
    }
  });

  // Nothing outlives the component: a cancelled animation leaves the base transform
  // (the start) behind, which is where a torn-down marquee belongs anyway.
  $effect(() => () => {
    loop?.cancel();
    ret?.cancel();
  });
</script>

<div
  bind:this={root}
  class={["marquee", fading && "is-fading", klass]}
  bind:clientWidth={containerWidth}
  title={text}
  aria-label={text}
>
  {#if scrolling}
    <!-- The moving track: two copies a gap apart, sliding by one copy-plus-gap so
       the second copy seamlessly takes the first's place. The transform is driven
       from the script above, not from CSS. -->
    <div
      bind:this={track}
      class="marquee-track"
      style="gap: {GAP}px;"
    >
      <span class="marquee-copy">{text}</span>
      <span class="marquee-copy" aria-hidden="true">{text}</span>
    </div>
  {:else}
    <!-- Fits (or reduced motion): a plain, still, truncating label. A hidden
       measuring copy keeps textWidth up to date so a later resize can start it
       scrolling without a mount. -->
    <span class="marquee-static">{text}</span>
    <span class="marquee-measure" aria-hidden="true" bind:clientWidth={textWidth}
      >{text}</span
    >
  {/if}
</div>

<style>
  /* The fade's reach, as a registered length so it can be transitioned: an
     unregistered custom property is an untyped token and would snap. */
  @property --marquee-fade {
    syntax: "<length>";
    inherits: false;
    initial-value: 0px;
  }

  .marquee {
    position: relative;
    overflow: hidden;
    white-space: nowrap;
    min-width: 0;
    /* The title runs the full width of its row, underneath the trailing controls,
       so the tail is masked away rather than colliding with them. Masking the text
       instead of laying a background-coloured gradient over it means the fade is
       right on every row state — active fill, hover fill, plain plate — without
       having to name which fill is underneath. At 0 length every stop collapses to
       the right edge and the mask is a no-op, so a row with nothing to hide behind
       pays nothing. The middle stop keeps the text nearly clear across the control
       itself, then ramps back to solid over the rest of the reach. */
    --marquee-fade: 0px;
    transition: --marquee-fade 120ms ease-out;
    -webkit-mask-image: linear-gradient(
      to left,
      transparent 0px,
      rgb(0 0 0 / 0.15) calc(var(--marquee-fade) * 0.45),
      #000 var(--marquee-fade)
    );
    mask-image: linear-gradient(
      to left,
      transparent 0px,
      rgb(0 0 0 / 0.15) calc(var(--marquee-fade) * 0.45),
      #000 var(--marquee-fade)
    );
  }

  .marquee.is-fading {
    --marquee-fade: 3rem;
  }

  .marquee-track {
    display: inline-flex;
    width: max-content;
    will-change: transform;
  }

  .marquee-copy {
    flex: 0 0 auto;
  }

  .marquee-static {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* The measuring copy is laid out (so it has a width) but never seen: off the
     flow, zero-height, non-interactive. */
  .marquee-measure {
    position: absolute;
    visibility: hidden;
    height: 0;
    white-space: nowrap;
    pointer-events: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .marquee {
      transition: none;
    }
  }
</style>
