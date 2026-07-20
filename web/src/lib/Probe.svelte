<script lang="ts">
  // Throwaway ticket-01 verification surface — NOT part of the cockpit. Reached
  // only via the #probe URL hash (see main.ts), so the real chrome is untouched.
  // It proves the design-system plumbing renders: token utilities resolve
  // (bg-background / text-foreground / bg-primary), the radius scale applies
  // (rounded-md), IBM Plex Sans loads (font-sans), and a Phosphor icon mounts.
  // Deleted once the reskin proper (tickets 02–04) exercises these for real.
  import { Compass } from 'phosphor-svelte'
</script>

<div class="min-h-screen bg-background text-foreground font-sans flex items-center justify-center">
  <div class="flex flex-col gap-4 rounded-md border border-border bg-card p-8 text-card-foreground">
    <div class="flex items-center gap-3">
      <Compass size={28} weight="bold" data-testid="probe-icon" />
      <span class="text-lg font-semibold">design-system probe</span>
    </div>
    <p class="text-sm text-muted-foreground">
      bg-background · text-foreground · rounded-md · IBM Plex Sans · Phosphor
    </p>
    <button class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground">
      primary button
    </button>
  </div>
</div>
